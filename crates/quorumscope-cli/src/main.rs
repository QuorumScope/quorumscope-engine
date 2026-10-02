use clap::{Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use std::env;

#[derive(Parser)]
#[command(name = "quorumscope")]
#[command(about = "QuorumScope CAP-77 engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Index,
    Report {
        #[arg(short, long)]
        incident: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/quorumscope".into());
    let rpc_url = env::var("STELLAR_RPC_URL").unwrap_or_else(|_| "https://soroban-testnet.stellar.org".into());

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            tracing::info!("Initializing QuorumScope with DB: {}", database_url);
            // let pool = quorumscope_storage::pool::connect(&quorumscope_storage::pool::StorageConfig::new(database_url)).await?;
            // quorumscope_storage::pool::run_migrations(&pool).await?;
        }
        Commands::Index => {
            tracing::info!("Starting indexer for RPC: {}", rpc_url);
            // run pipeline
        }
        Commands::Report { incident } => {
            tracing::info!("Generating report for incident: {}", incident);
            // generate pdf
        }
    }

    Ok(())
}
