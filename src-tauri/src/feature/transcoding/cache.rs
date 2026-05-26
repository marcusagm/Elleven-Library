//! Transcode Cache Manager
//!
//! Handles persistent storage and naming of transcoded media chunks and full files.
//! Integrated with the AppData directory for long-term caching of MKV/AVI conversions.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::detector;
use crate::core::formats::registry::FormatRegistry;
use crate::feature::transcoding::profiles::TranscodeQuality;
use std::sync::Arc;

/// Statistics about the transcoding cache.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    pub directory: String,
    pub size_bytes: u64,
    pub file_count: usize,
}

/// Manager for transcoded media files.
pub struct TranscodeCache {
    cache_dir: PathBuf,
    registry: Arc<FormatRegistry>,
}

impl TranscodeCache {
    /// Initializes the cache manager.
    pub fn new(app_data_dir: &Path, registry: Arc<FormatRegistry>) -> Self {
        let cache_dir = app_data_dir.join("transcoded");
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            tracing::error!("Failed to initialize transcode cache directory: {}", e);
        }
        Self {
            cache_dir,
            registry,
        }
    }

    /// Returns the internal cache directory.
    pub fn dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Generates a unique key for a source file and quality.
    fn generate_key(source: &Path, quality: TranscodeQuality) -> String {
        let mut hasher = DefaultHasher::new();
        source.to_string_lossy().hash(&mut hasher);
        format!("{:?}-quality", quality).hash(&mut hasher);

        // Include mod time for cache invalidation
        if let Ok(metadata) = fs::metadata(source) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                }
            }
        }

        format!("{:16x}", hasher.finish())
    }

    /// Returns the full path to a cached file.
    pub fn get_cache_path(&self, source: &Path, quality: TranscodeQuality) -> PathBuf {
        let key = Self::generate_key(source, quality);
        let ext = detector::get_output_extension(&self.registry, source);
        self.cache_dir.join(format!("{}.{}", key, ext))
    }

    /// Checks if a cached version of the file exists.
    pub fn exists(&self, source: &Path, quality: TranscodeQuality) -> bool {
        let path = self.get_cache_path(source, quality);
        path.exists() && path.is_file()
    }

    /// Cleans up files older than `max_age_days`.
    pub fn cleanup(&self, max_age_days: u64) -> usize {
        let mut deleted_count = 0;
        let max_age = Duration::from_secs(max_age_days * 24 * 60 * 60);
        let now = SystemTime::now();

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(age) = now.duration_since(modified) {
                                if age > max_age && fs::remove_file(&path).is_ok() {
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        deleted_count
    }

    /// Clears the entire cache.
    pub fn clear_all(&self) -> usize {
        let mut deleted_count = 0;
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && fs::remove_file(&path).is_ok() {
                    deleted_count += 1;
                }
            }
        }
        deleted_count
    }

    /// Returns current cache statistics.
    pub fn get_stats(&self) -> CacheStats {
        let mut size_bytes = 0;
        let mut file_count = 0;

        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        size_bytes += metadata.len();
                        file_count += 1;
                    }
                }
            }
        }

        CacheStats {
            directory: self.cache_dir.to_string_lossy().to_string(),
            size_bytes,
            file_count,
        }
    }
}
