//! # Nervous System Events Module
//!
//! This module defines the contracts (Traits) and payloads for
//! asynchronous event-driven communication (EDA) in Mundam.

pub mod bus;
pub mod payloads;

pub use bus::AppEventBus;
pub use payloads::DomainEvent;
