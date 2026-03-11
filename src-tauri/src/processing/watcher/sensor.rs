use super::debouncer::EventDebouncer;
use crate::core::error::AppResult;
use crate::core::events::bus::AppEventBus;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

/// Singleton service that manages active filesystem watchers.
///
/// It listens for low-level OS events, debounces them, and publishes
/// clean Domain Events to the application's Event Bus.
pub struct WatcherService {
    /// Broadcast bus for publishing domain events.
    event_bus: Arc<dyn AppEventBus>,
    /// Handle to the native watcher (kept alive).
    native_watcher: Mutex<Option<RecommendedWatcher>>,
}

/// Implementation of the WatcherService.
impl WatcherService {
    /// Create a new WatcherService.
    ///
    /// # Arguments
    ///
    /// * `event_bus` - Broadcast bus for publishing domain events.
    ///
    /// # Returns
    ///
    /// * `Self` - A new WatcherService instance.
    pub fn new(event_bus: Arc<dyn AppEventBus>) -> Self {
        Self {
            event_bus,
            native_watcher: Mutex::new(None),
        }
    }

    /// Start watching a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to watch.
    /// * `token` - Cancellation token for the background loop.
    ///
    /// # Returns
    ///
    /// * `AppResult<()>` - Result of the watch operation.
    #[instrument(skip_all)]
    pub async fn watch(&self, path: PathBuf, token: CancellationToken) -> AppResult<()> {
        let (tx, mut rx) = mpsc::channel::<Event>(1000);
        let event_bus = self.event_bus.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        watcher.watch(&path, RecursiveMode::Recursive)?;
        let path_for_log = path.clone();

        // Spawn the debouncer loop
        tokio::spawn(async move {
            // Refactored approach: The loop consumes from rx, feeds debouncer, and ticks.
            let (out_tx, mut out_rx) = mpsc::channel(100);
            let mut debouncer = EventDebouncer::new(out_tx);
            let mut interval = tokio::time::interval(Duration::from_millis(200));

            loop {
                tokio::select! {
                    _ = token.cancelled() => {
                        info!("Watcher: Shutdown signal received for path: {:?}", path);
                        break;
                    }
                    Some(event) = rx.recv() => {
                        debouncer.handle_event(event).await;
                    }
                    _ = interval.tick() => {
                        debouncer.tick(Duration::from_millis(600)).await;
                    }
                    Some(domain_event) = out_rx.recv() => {
                        let _ = event_bus.publish(domain_event);
                    }
                }
            }
        });

        let mut guard = self.native_watcher.lock().await;
        *guard = Some(watcher);

        info!("Watcher service started for: {:?}", path_for_log);
        Ok(())
    }

    /// Stop watching a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to unwatch.
    ///
    /// # Returns
    ///
    /// * `AppResult<()>` - Result of the unwatch operation.
    #[instrument(skip(self))]
    pub async fn unwatch(&self, path: PathBuf) -> AppResult<()> {
        let mut guard = self.native_watcher.lock().await;
        if let Some(watcher) = guard.as_mut() {
            if let Err(e) = watcher.unwatch(&path) {
                tracing::error!("Failed to unwatch path {:?}: {}", path, e);
            } else {
                info!("Watcher stopped for: {:?}", path);
            }
        }
        Ok(())
    }
}
