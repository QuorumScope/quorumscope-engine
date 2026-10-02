use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct IndexerPipeline;

impl IndexerPipeline {
    pub fn start() -> (mpsc::Sender<u32>, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<u32>(100);

        let handle = tokio::spawn(async move {
            while let Some(_ledger) = rx.recv().await {
                // Pipeline logic will go here
            }
        });

        (tx, handle)
    }
}
