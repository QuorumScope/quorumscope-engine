use anyhow::Context;
use clap::{Parser, Subcommand};
use quorumscope_api::ApiConfig;
use quorumscope_indexer::sync::SyncTask;
use quorumscope_report::builder::ReportBuilder;
use quorumscope_report::data::{ReportData, ReportEvent, ReportKey, ReportSnapshot};
use quorumscope_rpc::client::RpcClient;
use quorumscope_rpc::config::RpcConfig;
use quorumscope_storage::indexer::IndexerRepository;
use quorumscope_storage::network::ensure_network;
use quorumscope_storage::pool::{StorageConfig, connect};
use sqlx::PgPool;
use std::path::PathBuf;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "quorumscope")]
#[command(about = "QuorumScope CAP-77 engine", long_about = None)]
struct Cli {
    /// PostgreSQL connection URL. Include credentials in the URL; there is no default password.
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://localhost/quorumscope"
    )]
    db_url: String,

    /// Stellar RPC endpoint used by the indexer.
    #[arg(
        long,
        env = "STELLAR_RPC_URL",
        default_value = "https://soroban-testnet.stellar.org"
    )]
    rpc_url: String,

    /// Name under which the network is stored. State is kept per network name.
    #[arg(long, env = "NETWORK_NAME", default_value = "testnet")]
    network_name: String,

    /// Network passphrase. It is part of every transaction hash.
    #[arg(
        long,
        env = "NETWORK_PASSPHRASE",
        default_value = "Test SDF Network ; September 2015"
    )]
    network_passphrase: String,

    /// Seconds between indexer polls in watch mode.
    #[arg(long, env = "POLLING_INTERVAL_SEC", default_value = "5")]
    polling_interval: u64,

    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply database migrations.
    Init,
    /// Serve the HTTP API.
    Serve {
        #[arg(long, env = "API_BIND", default_value = "127.0.0.1:8080")]
        bind: std::net::SocketAddr,
        /// Highest protocol version this release is verified against. Compatibility
        /// is reported as unknown when unset.
        #[arg(long, env = "VERIFIED_PROTOCOL_MAX")]
        verified_protocol_max: Option<i32>,
        /// Seconds without a network observation before state is reported as stale.
        #[arg(long, env = "STALE_AFTER_SEC", default_value = "300")]
        stale_after_sec: i64,
        /// Ledgers the indexer may trail the network before it is reported as behind.
        #[arg(long, env = "MAX_LAG_LEDGERS", default_value = "10")]
        max_lag_ledgers: i64,
        /// Comma-separated browser origins allowed to call the API, for example
        /// `https://console.example.org`. No CORS headers are sent when unset.
        #[arg(long, env = "ALLOWED_ORIGINS", value_delimiter = ',')]
        allowed_origins: Vec<String>,
    },
    /// Read freeze state from the network into the database.
    Index {
        #[command(subcommand)]
        mode: IndexMode,
    },
    /// Write a PDF report for one freeze episode. Needs the `typst` binary,
    /// or set QUORUMSCOPE_TYPST to its path.
    Report {
        /// Episode (incident) UUID.
        #[arg(short, long)]
        incident: Uuid,
        /// Output PDF path.
        #[arg(short, long, default_value = "report.pdf")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum IndexMode {
    /// Read once and exit.
    Once,
    /// Poll until interrupted.
    Watch,
}

async fn load_report(pool: &PgPool, incident: Uuid) -> anyhow::Result<ReportData> {
    let row: Option<(String, String, String, i64, Option<i64>, Uuid)> = sqlx::query_as(
        "SELECT n.name, i.basis, i.status, i.opened_ledger, i.closed_ledger, i.network_id \
         FROM incidents i JOIN networks n ON n.id = i.network_id WHERE i.id = $1",
    )
    .bind(incident)
    .fetch_optional(pool)
    .await?;
    let (network, basis, status, opened, closed, network_id) =
        row.with_context(|| format!("no episode with id {incident}"))?;
    let upper = closed.unwrap_or(i64::MAX);
    let keys: Vec<(Vec<u8>, Option<String>)> = sqlx::query_as(
        "SELECT DISTINCT f.key_hash, k.key_kind FROM freeze_changes f \
         LEFT JOIN ledger_keys k ON k.network_id = f.network_id AND k.key_hash = f.key_hash \
         WHERE f.network_id = $1 AND f.ledger_sequence BETWEEN $2 AND $3 ORDER BY f.key_hash",
    )
    .bind(network_id)
    .bind(opened)
    .bind(upper)
    .fetch_all(pool)
    .await?;
    let events: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT ledger_sequence, kind, evidence_ref FROM incident_events WHERE incident_id = $1 ORDER BY ledger_sequence, id",
    )
    .bind(incident)
    .fetch_all(pool)
    .await?;
    let snapshots: Vec<(i64, i64, i64, i64)> = sqlx::query_as(
        "SELECT ledger_sequence, total_frozen_accounts, total_frozen_trustlines, total_bypassed_txs \
         FROM impact_snapshots WHERE incident_id = $1 ORDER BY ledger_sequence, id",
    )
    .bind(incident)
    .fetch_all(pool)
    .await?;
    Ok(ReportData {
        incident_id: incident.to_string(),
        network,
        basis,
        status,
        opened_ledger: opened,
        closed_ledger: closed,
        keys: keys
            .into_iter()
            .map(|(hash, kind)| ReportKey {
                key_id: hex::encode(hash),
                kind,
            })
            .collect(),
        events: events
            .into_iter()
            .map(|(ledger, kind, evidence_ref)| ReportEvent {
                ledger,
                kind,
                evidence_ref,
            })
            .collect(),
        snapshots: snapshots
            .into_iter()
            .map(|(ledger, a, t, b)| ReportSnapshot {
                ledger,
                frozen_accounts: a,
                frozen_trustlines: t,
                bypassed_transactions: b,
            })
            .collect(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.log_level.to_lowercase().as_str() {
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    match &cli.command {
        Commands::Init => {
            let pool = connect(&StorageConfig::new(cli.db_url.clone())).await?;
            quorumscope_storage::pool::run_migrations(&pool).await?;
            tracing::info!("Migrations applied");
        }
        Commands::Serve {
            bind,
            verified_protocol_max,
            stale_after_sec,
            max_lag_ledgers,
            allowed_origins,
        } => {
            let pool = connect(&StorageConfig::new(cli.db_url.clone())).await?;
            quorumscope_storage::pool::run_migrations(&pool).await?;
            let config = ApiConfig {
                verified_protocol_max: *verified_protocol_max,
                stale_after_secs: *stale_after_sec,
                max_lag_ledgers: *max_lag_ledgers,
                allowed_origins: allowed_origins.clone(),
            };
            let listener = tokio::net::TcpListener::bind(bind).await?;
            tracing::info!(address = %bind, "Serving QuorumScope API");
            axum::serve(listener, quorumscope_api::app_with_config(pool, config))
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
        }
        Commands::Index { mode } => {
            let pool = connect(&StorageConfig::new(cli.db_url.clone())).await?;
            quorumscope_storage::pool::run_migrations(&pool).await?;
            let network_id =
                ensure_network(&pool, &cli.network_name, &cli.network_passphrase).await?;
            tracing::info!(network = %cli.network_name, rpc = %cli.rpc_url, "Starting indexer");
            let client = RpcClient::new(RpcConfig::new(cli.rpc_url.as_str())?)?;
            let task = SyncTask::new(client, IndexerRepository::new(pool), network_id);
            match mode {
                IndexMode::Once => {
                    task.run_once().await?;
                    tracing::info!("One-shot index complete");
                }
                IndexMode::Watch => {
                    task.run_watch(Duration::from_secs(cli.polling_interval))
                        .await?;
                }
            }
        }
        Commands::Report { incident, output } => {
            let pool = connect(&StorageConfig::new(cli.db_url.clone())).await?;
            let data = load_report(&pool, *incident).await?;
            let size = ReportBuilder::new().generate_pdf(&data, output)?;
            tracing::info!(path = %output.display(), bytes = size, "Report written");
        }
    }

    Ok(())
}
