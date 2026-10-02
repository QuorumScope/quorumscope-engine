use tokio::signal;
use tokio::sync::broadcast;

pub struct ShutdownManager {
    shutdown_tx: broadcast::Sender<()>,
}

impl ShutdownManager {
    pub fn new() -> (Self, broadcast::Receiver<()>) {
        let (tx, rx) = broadcast::channel(1);
        (Self { shutdown_tx: tx }, rx)
    }

    pub async fn listen_for_shutdown(&self) {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        
        tracing::info!("Shutdown signal received");
        let _ = self.shutdown_tx.send(());
    }
}
