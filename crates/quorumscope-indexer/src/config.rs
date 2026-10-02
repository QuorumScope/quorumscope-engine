use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub worker_threads: usize,
    pub batch_size: u32,
    pub poll_interval: Duration,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        let threads = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        Self {
            worker_threads: threads,
            batch_size: 100,
            poll_interval: Duration::from_secs(10),
        }
    }
}
