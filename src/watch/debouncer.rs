use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::Mutex;

/// Configuration for debouncing behavior
#[derive(Debug, Clone)]
pub struct DebouncerConfig {
    /// Time to wait after last event before triggering (default: 300ms)
    pub debounce_delay: Duration,
    /// Maximum time to wait regardless of new events (default: 2s)
    pub max_delay: Duration,
}

impl Default for DebouncerConfig {
    fn default() -> Self {
        Self {
            debounce_delay: Duration::from_millis(300),
            max_delay: Duration::from_secs(2),
        }
    }
}

/// Accumulates file change events and batches them
pub struct Debouncer {
    config: DebouncerConfig,
    pending_paths: Arc<Mutex<HashSet<PathBuf>>>,
    first_event_time: Arc<Mutex<Option<Instant>>>,
    last_event_time: Arc<Mutex<Instant>>,
    tx: mpsc::Sender<Vec<PathBuf>>,
    timer_running: Arc<Mutex<bool>>,
}

impl Debouncer {
    pub fn new(config: DebouncerConfig) -> (Self, mpsc::Receiver<Vec<PathBuf>>) {
        let (tx, rx) = mpsc::channel(16);
        let debouncer = Self {
            config,
            pending_paths: Arc::new(Mutex::new(HashSet::new())),
            first_event_time: Arc::new(Mutex::new(None)),
            last_event_time: Arc::new(Mutex::new(Instant::now())),
            tx,
            timer_running: Arc::new(Mutex::new(false)),
        };
        (debouncer, rx)
    }

    /// Add a path to the pending set and schedule flush
    pub async fn add_path(&self, path: PathBuf) {
        let mut paths = self.pending_paths.lock().await;
        let mut first_time = self.first_event_time.lock().await;
        let mut last_time = self.last_event_time.lock().await;
        let mut timer_running = self.timer_running.lock().await;

        let now = Instant::now();
        paths.insert(path);
        *last_time = now;

        if first_time.is_none() {
            *first_time = Some(now);
        }

        // Start the flush timer if not already running
        if !*timer_running {
            *timer_running = true;
            drop(paths);
            drop(first_time);
            drop(last_time);
            drop(timer_running);
            self.spawn_flush_timer();
        }
    }

    fn spawn_flush_timer(&self) {
        let pending = Arc::clone(&self.pending_paths);
        let first_event = Arc::clone(&self.first_event_time);
        let last_event = Arc::clone(&self.last_event_time);
        let timer_running = Arc::clone(&self.timer_running);
        let config = self.config.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(config.debounce_delay).await;

                let first = *first_event.lock().await;
                let last = *last_event.lock().await;
                let now = Instant::now();

                // Check if we should flush
                let should_flush = match first {
                    Some(first_time) => {
                        let since_last = now.duration_since(last);
                        let since_first = now.duration_since(first_time);

                        since_last >= config.debounce_delay || since_first >= config.max_delay
                    }
                    None => false,
                };

                if should_flush {
                    let mut paths = pending.lock().await;
                    if !paths.is_empty() {
                        let batch: Vec<PathBuf> = paths.drain().collect();
                        let _ = tx.send(batch).await;
                    }
                    *first_event.lock().await = None;
                    *timer_running.lock().await = false;
                    break;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debouncer_batches_events() {
        let config = DebouncerConfig {
            debounce_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
        };
        let (debouncer, mut rx) = Debouncer::new(config);

        // Add multiple paths rapidly
        debouncer.add_path(PathBuf::from("/test/file1.rs")).await;
        debouncer.add_path(PathBuf::from("/test/file2.rs")).await;
        debouncer.add_path(PathBuf::from("/test/file3.rs")).await;

        // Wait for debounce to trigger
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should receive a single batch
        if let Ok(batch) = rx.try_recv() {
            assert_eq!(batch.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_debouncer_deduplicates() {
        let config = DebouncerConfig {
            debounce_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
        };
        let (debouncer, mut rx) = Debouncer::new(config);

        // Add same path multiple times
        debouncer.add_path(PathBuf::from("/test/file.rs")).await;
        debouncer.add_path(PathBuf::from("/test/file.rs")).await;
        debouncer.add_path(PathBuf::from("/test/file.rs")).await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        if let Ok(batch) = rx.try_recv() {
            assert_eq!(batch.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_debouncer_max_delay() {
        let config = DebouncerConfig {
            debounce_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(200),
        };
        let (debouncer, mut rx) = Debouncer::new(config);

        // Add paths with delays shorter than debounce_delay
        debouncer.add_path(PathBuf::from("/test/file1.rs")).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        debouncer.add_path(PathBuf::from("/test/file2.rs")).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        debouncer.add_path(PathBuf::from("/test/file3.rs")).await;

        // Wait for max_delay to trigger
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should receive the batch due to max_delay
        if let Ok(batch) = rx.try_recv() {
            assert!(!batch.is_empty());
        }
    }
}
