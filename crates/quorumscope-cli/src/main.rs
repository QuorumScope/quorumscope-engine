use clap::{Parser, Subcommand};
use quorumscope_domain::network::NetworkId;
use quorumscope_indexer::sync::SyncTask;
use quorumscope_rpc::client::RpcClient;
use quorumscope_rpc::config::RpcConfig;
use quorumscope_storage::indexer::IndexerRepository;
use quorumscope_storage::pool::{StorageConfig, connect};
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "quorumscope")]
#[command(about = "QuorumScope CAP-77 engine", long_about = None)]
struct Cli {
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://localhost/quorumscope"
    )]
    db_url: String,

    #[arg(
        long,
        env = "STELLAR_RPC_URL",
        default_value = "https://soroban-testnet.stellar.org"
    )]
    rpc_url: String,

    #[arg(
        long,
        env = "NETWORK_PASSPHRASE",
        default_value = "Test SDF Network ; September 2015"
    )]
    network_passphrase: String,

    #[arg(long, env = "POLLING_INTERVAL_SEC", default_value = "5")]
    polling_interval: u64,

    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Serve {
        #[arg(long, env = "API_BIND", default_value = "127.0.0.1:8080")]
        bind: std::net::SocketAddr,
    },
    Index {
        #[command(subcommand)]
        mode: IndexMode,
    },
    Report {
        #[arg(short, long)]
        incident: String,
    },
}

#[derive(Subcommand)]
enum IndexMode {
    Once,
    Watch,
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
            tracing::info!("Initializing QuorumScope with DB: {}", cli.db_url);
            let pool = connect(&StorageConfig::new(cli.db_url)).await?;
            quorumscope_storage::pool::run_migrations(&pool).await?;
        }
        Commands::Serve { bind } => {
            let pool = connect(&StorageConfig::new(cli.db_url)).await?;
            quorumscope_storage::pool::run_migrations(&pool).await?;
            let listener = tokio::net::TcpListener::bind(bind).await?;
            tracing::info!(address = %bind, "Serving QuorumScope API");
            axum::serve(listener, quorumscope_api::app(pool)).await?;
        }
        Commands::Index { mode } => {
            tracing::info!("Starting indexer for RPC: {}", cli.rpc_url);
            let pool = connect(&StorageConfig::new(cli.db_url)).await?;
            let repo = IndexerRepository::new(pool);
            let client = RpcClient::new(RpcConfig {
                endpoint: cli.rpc_url.parse().unwrap(),
                timeout: std::time::Duration::from_secs(30),
                max_retries: 3,
                base_backoff: std::time::Duration::from_millis(500),
            })?;

            // Hardcode a NetworkId based on a predefined UUID for this assignment,
            // since we don't have a network registry lookup built yet.
            let network_id = NetworkId::new(); // or a lookup based on passphrase if registry existed

            let task = SyncTask::new(client, repo, network_id);

            match mode {
                IndexMode::Once => {
                    tracing::info!("Running indexer in one-shot mode...");
                    task.run_once().await?;
                    tracing::info!("One-shot index complete.");
                }
                IndexMode::Watch => {
                    tracing::info!("Running indexer in watch mode...");
                    task.run_watch(Duration::from_secs(cli.polling_interval))
                        .await?;
                }
            }
        }
        Commands::Report { incident } => {
            tracing::info!("Generating report for incident: {}", incident);
        }
    }

    Ok(())
}
