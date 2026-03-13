use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// The central "Cartório" (Registry) for all supported file formats.
#[derive(Clone)]
pub struct FormatRegistry {
    /// Instant routing (O(1)) for 99% of cases using normalized extensions.
    by_extension: HashMap<String, Arc<dyn FormatProvider>>,
    /// Passive fallback for binary formats without extensions or deep validation.
    deep_checkers: Vec<Arc<dyn FormatProvider>>,
}

/// Format Registry implementation.
impl FormatRegistry {
    /// Creates a new, empty FormatRegistry.
    pub fn new() -> Self {
        Self {
            by_extension: HashMap::new(),
            deep_checkers: Vec::new(),
        }
    }

    /// Registers a provider in the registry.
    ///
    /// This will automatically index the provider by its supported extensions
    /// and add it to the deep checker fallback list.
    pub fn register(&mut self, provider: Arc<dyn FormatProvider>) {
        for extension in provider.supported_extensions() {
            self.by_extension
                .insert(extension.to_lowercase(), provider.clone());
        }
        self.deep_checkers.push(provider);
    }

    /// Resolves the most appropriate format provider for a given file.
    ///
    /// # Performance
    /// 1. Tries O(1) lookup via file extension.
    /// 2. If no extension matches or if the provider requires magic byte validation,
    ///    it falls back to O(N) magic byte inspection across all registered providers.
    ///
    /// # Arguments
    /// * `path` - The canonical file path.
    /// * `header` - A buffer containing the initial bytes of the file for magic byte validation.
    pub fn resolve(&self, path: &Path, header: &[u8]) -> Option<Arc<dyn FormatProvider>> {
        // 1. FAST ATTEMPT (O(1)): Cache by Extension
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            if let Some(provider) = self.by_extension.get(&extension.to_lowercase()) {
                // Double check magic bytes if the provider supports it and we have header content
                if header.is_empty() || provider.supports_magic_bytes(header) {
                    return Some(provider.clone());
                }
            }
        }

        // 2. SLOW FALLBACK (O(N)): Magic Bytes inferred (For files without extension or ghost extensions)
        if !header.is_empty() {
            return self
                .deep_checkers
                .iter()
                .find(|provider| provider.supports_magic_bytes(header))
                .cloned();
        }

        None
    }

    /// Detects the granular format for a given path using extension-based routing.
    pub fn detect(&self, path: &Path) -> Option<SupportedFormat> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        let provider = self.by_extension.get(&extension)?;
        provider.supported_formats().into_iter().find(|sf| {
            sf.extensions
                .iter()
                .any(|ext| ext.to_lowercase() == extension)
        })
    }

    /// Returns a provider by its unique name.
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn FormatProvider>> {
        self.deep_checkers.iter().find(|p| p.name() == name).cloned()
    }

    /// Returns a list of all supported formats and their extensions.
    pub fn get_supported_formats(&self) -> Vec<SupportedFormat> {
        self.deep_checkers
            .iter()
            .flat_map(|provider| provider.supported_formats())
            .collect()
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tests for the format registry.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::AppResult;
    use crate::core::formats::capabilities::MetadataCapability;
    use async_trait::async_trait;

    struct MockProvider;

    /// Tests the metadata capability.
    #[async_trait]
    impl MetadataCapability for MockProvider {
        async fn extract_technical(&self, _path: &Path) -> AppResult<serde_json::Value> {
            Ok(serde_json::json!({ "type": "mock" }))
        }
        async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
            Ok(serde_json::json!({ "tags": ["mock"] }))
        }
    }

    /// Tests the format provider.
    impl FormatProvider for MockProvider {
        fn name(&self) -> &'static str {
            "MOCK"
        }
        fn supported_extensions(&self) -> Vec<&'static str> {
            vec!["mock", "MOK"]
        }
        fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
            header_bytes.starts_with(b"MOCK")
        }
        fn metadata(&self) -> Option<&dyn MetadataCapability> {
            Some(self)
        }
    }

    /// Tests the format registry resolution.
    #[test]
    fn test_registry_resolution() {
        let mut registry = FormatRegistry::new();
        let provider = Arc::new(MockProvider);
        registry.register(provider.clone());

        // Test O(1) extension resolution
        let path = Path::new("test.mock");
        let resolved = registry.resolve(path, &[]);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name(), "MOCK");

        // Test case-insensitivity
        let path = Path::new("test.MOK");
        let resolved = registry.resolve(path, &[]);
        assert!(resolved.is_some());

        // Test magic bytes fallback
        let path = Path::new("no_extension");
        let resolved = registry.resolve(path, b"MOCK_HEADER");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name(), "MOCK");

        // Test failure
        let path = Path::new("test.unknown");
        let resolved = registry.resolve(path, b"RANDOM");
        assert!(resolved.is_none());
    }
}
