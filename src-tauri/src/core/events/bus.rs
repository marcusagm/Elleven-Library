use crate::core::error::AppResult;
use async_trait::async_trait;

use super::payloads::DomainEvent;

/// Injectable Interface Port for the Event Bus.
///
/// Follows the Hexagonal "Port" pattern, allowing the domain to publish events
/// without knowing the underlying implementation (Tokio Broadcast, NATS, etc).
#[async_trait]
pub trait AppEventBus: Send + Sync {
    /// Dispatches an event to all logged-in listeners.
    ///
    /// # Arguments
    /// * `event` - The domain event to be published.
    ///
    /// # Errors
    /// Returns `AppError::EventBus` in case of critical publication failure.
    fn publish(&self, event: DomainEvent) -> AppResult<()>;

    /// Subscribes to receive an event stream.
    ///
    /// Returns a Receiver that can be used in an asynchronous loop.
    /// Note: Due to the nature of `tokio::sync::broadcast`, the receiver
    /// may throw a `Lagged` error if the consumer is slower than the producer.
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<DomainEvent>;
}
