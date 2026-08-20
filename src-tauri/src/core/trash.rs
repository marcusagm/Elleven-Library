//! Centralized Trash Path Resolution
//!
//! Provides a single source of truth for computing the physical path of an
//! asset inside the internal Mundam trash directory. All layers of the
//! application (protocol handlers, streaming server, RPC commands) must use
//! this module instead of inlining the path logic.
//!
//! ## Naming Convention
//!
//! Files inside `{app_data}/trash/` are stored as:
//!
//! ```text
//! {asset_id}_{epoch_seconds}_{original_filename}
//! ```
//!
//! Where `epoch_seconds` is the Unix timestamp of the `deleted_at` field,
//! guaranteeing uniqueness even in edge-case scenarios such as re-deletion
//! after a restore.

use std::path::{Path, PathBuf};

use crate::core::models::asset::Asset;

/// The name of the directory inside `app_data` where trashed files are stored.
pub const TRASH_DIRECTORY_NAME: &str = "trash";

/// Resolves the physical path to the trash directory for a given `app_data` root.
///
/// # Arguments
/// * `app_data_directory` - The resolved `app_local_data_dir()` root.
///
/// # Returns
/// The full path to the trash directory.
pub fn trash_directory(app_data_directory: &Path) -> PathBuf {
    app_data_directory.join(TRASH_DIRECTORY_NAME)
}

/// Builds the trash file path for an asset using its `deleted_at` timestamp.
///
/// The resulting path follows the format:
/// `{app_data}/trash/{asset_id}_{epoch_secs}_{filename}`
///
/// # Arguments
/// * `app_data_directory` - The resolved `app_local_data_dir()` root.
/// * `asset_id` - The unique identifier of the asset.
/// * `original_path` - The original filesystem path of the asset (used to extract the filename).
/// * `deleted_at` - The `deleted_at` timestamp from the database record.
///
/// # Returns
/// `Some(PathBuf)` if the original path has a valid filename, `None` otherwise.
pub fn build_trash_path(
    app_data_directory: &Path,
    asset_id: &str,
    original_path: &Path,
    deleted_at: &chrono::DateTime<chrono::Utc>,
) -> Option<PathBuf> {
    let file_name = original_path.file_name()?;
    let epoch_seconds = deleted_at.timestamp();
    Some(
        trash_directory(app_data_directory).join(format!(
            "{}_{}_{}",
            asset_id,
            epoch_seconds,
            file_name.to_string_lossy()
        )),
    )
}

/// Resolves the physical path of an asset, automatically redirecting to the
/// internal trash directory if the asset has been soft-deleted.
///
/// This is the **primary entry point** that all delivery layers should call
/// instead of using `asset.path` directly.
///
/// # Arguments
/// * `asset` - The asset record from the database.
/// * `app_data_directory` - The resolved `app_local_data_dir()` root.
///
/// # Returns
/// - If the asset is trashed (`deleted_at` is set): the path inside the trash directory.
/// - Otherwise: the original `asset.path`.
pub fn resolve_physical_path(asset: &Asset, app_data_directory: &Path) -> PathBuf {
    if let Some(deleted_at) = asset.deleted_at {
        if let Some(trash_path) =
            build_trash_path(app_data_directory, &asset.id, &asset.path, &deleted_at)
        {
            // If the timestamped path exists, use it directly.
            if trash_path.exists() {
                return trash_path;
            }

            // Fallback: try the legacy format ({asset_id}_{filename}) for backward
            // compatibility with files trashed before the timestamp migration.
            if let Some(file_name) = asset.path.file_name() {
                let legacy_path = trash_directory(app_data_directory)
                    .join(format!("{}_{}", asset.id, file_name.to_string_lossy()));
                if legacy_path.exists() {
                    return legacy_path;
                }
            }

            // Return the timestamped path even if it doesn't exist yet
            // (e.g. during the move_to_trash flow before the rename completes).
            return trash_path;
        }
    }
    asset.path.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_build_trash_path_creates_correct_format() {
        let app_data = PathBuf::from("/tmp/mundam_test");
        let deleted_at = chrono::Utc.with_ymd_and_hms(2026, 8, 12, 20, 42, 0).unwrap();

        let result = build_trash_path(
            &app_data,
            "abc-123",
            &PathBuf::from("/library/photos/sunset.jpg"),
            &deleted_at,
        );

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(
            path,
            PathBuf::from("/tmp/mundam_test/trash/abc-123_1786567320_sunset.jpg")
        );
    }

    #[test]
    fn test_build_trash_path_returns_none_for_empty_filename() {
        let app_data = PathBuf::from("/tmp/mundam_test");
        let deleted_at = chrono::Utc::now();

        let result = build_trash_path(
            &app_data,
            "abc-123",
            &PathBuf::from("/"),
            &deleted_at,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_trash_directory_path() {
        let app_data = PathBuf::from("/home/user/.local/share/mundam");
        assert_eq!(
            trash_directory(&app_data),
            PathBuf::from("/home/user/.local/share/mundam/trash")
        );
    }

    #[test]
    fn test_resolve_physical_path_returns_original_when_not_trashed() {
        let asset = create_test_asset("test-id", "/library/photo.jpg", None);

        let result = resolve_physical_path(&asset, &PathBuf::from("/tmp/app"));
        assert_eq!(result, PathBuf::from("/library/photo.jpg"));
    }

    #[test]
    fn test_resolve_physical_path_returns_trash_path_when_deleted() {
        let deleted_at = chrono::Utc.with_ymd_and_hms(2026, 8, 12, 20, 42, 0).unwrap();
        let asset = create_test_asset(
            "asset-456",
            "/library/videos/clip.mp4",
            Some(deleted_at),
        );

        let result = resolve_physical_path(&asset, &PathBuf::from("/tmp/app"));
        assert_eq!(
            result,
            PathBuf::from("/tmp/app/trash/asset-456_1786567320_clip.mp4")
        );
    }

    #[test]
    fn test_different_timestamps_produce_different_paths() {
        let app_data = PathBuf::from("/tmp/app");
        let path = PathBuf::from("/library/photo.jpg");

        let timestamp_a = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let timestamp_b = chrono::Utc.with_ymd_and_hms(2026, 6, 15, 12, 30, 0).unwrap();

        let path_a = build_trash_path(&app_data, "same-id", &path, &timestamp_a).unwrap();
        let path_b = build_trash_path(&app_data, "same-id", &path, &timestamp_b).unwrap();

        assert_ne!(path_a, path_b, "Different timestamps must produce different paths");
    }

    #[test]
    fn test_different_asset_ids_produce_different_paths() {
        let app_data = PathBuf::from("/tmp/app");
        let path = PathBuf::from("/library/photo.jpg");
        let deleted_at = chrono::Utc::now();

        let path_a = build_trash_path(&app_data, "id-aaa", &path, &deleted_at).unwrap();
        let path_b = build_trash_path(&app_data, "id-bbb", &path, &deleted_at).unwrap();

        assert_ne!(path_a, path_b, "Different asset IDs must produce different paths");
    }

    #[test]
    fn test_trash_path_preserves_file_extension() {
        let app_data = PathBuf::from("/tmp/app");
        let deleted_at = chrono::Utc::now();

        let path = build_trash_path(
            &app_data,
            "id-1",
            &PathBuf::from("/photos/my image.png"),
            &deleted_at,
        )
        .unwrap();

        assert!(
            path.to_string_lossy().ends_with(".png"),
            "Trash path must preserve the original file extension"
        );
    }

    #[test]
    fn test_trash_directory_name_constant() {
        assert_eq!(TRASH_DIRECTORY_NAME, "trash");
    }

    /// Helper to create a test Asset with minimal required fields.
    fn create_test_asset(
        id: &str,
        path: &str,
        deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Asset {
        Asset {
            id: id.to_string(),
            name: PathBuf::from(path)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            path: PathBuf::from(path),
            state: crate::core::models::asset::AssetState::Idle,
            format_type: "JPEG".to_string(),
            family: "image".to_string(),
            file_size: 1024,
            created_at: None,
            modified_at: None,
            added_at: None,
            updated_at: None,
            width: None,
            height: None,
            duration_secs: None,
            technical_payload: None,
            semantic_payload: None,
            dominant_color: None,
            folder_id: None,
            thumbnail_path: None,
            rating: None,
            notes: None,
            is_favorite: false,
            deleted_at,
        }
    }
}
