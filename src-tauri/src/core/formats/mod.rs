//! Format Registry and Capability definitions.
//!
//! This module provides the infrastructure for multi-format support in Mundam.
//! It defines traits for metadata extraction and thumbnail generation (Capabilities),
//! and a central Registry for O(1) format resolution.
//!
//! # Architecture
//!
//! This module lives in `core/` and therefore contains only **ports** (traits)
//! and **domain types**. Concrete provider implementations live in
//! `processing::media::providers`, and the registry assembly happens at
//! bootstrap time via `build_format_registry()` in `bootstrap::system`.

pub mod capabilities;
pub mod provider;
pub mod registry;
pub mod types;

pub use provider::SupportedFormat;
pub use registry::FormatRegistry;
