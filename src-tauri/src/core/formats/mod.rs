//! Format Registry and Capability definitions.
//!
//! This module provides the infrastructure for multi-format support in Mundam.
//! It defines traits for metadata extraction and thumbnail generation (Capabilities),
//! and a central Registry for O(1) format resolution.

pub mod capabilities;
pub mod provider;
pub mod registry;

pub use registry::FormatRegistry;

/// Factory function to build the main FormatRegistry.
///
/// This is used during application boot to register all supported format plugins.
pub fn build_format_registry() -> FormatRegistry {
    let registry = FormatRegistry::new();

    // Specific providers will be registered here in future sprints
    // Example:
    // registry.register(Arc::new(PhotoshopFormatProvider::new()));
    // registry.register(Arc::new(FfmpegVideoFormatProvider::new()));

    registry
}
