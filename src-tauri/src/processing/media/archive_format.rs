use crate::core::error::AppResult;
use crate::core::formats::capabilities::ThumbnailCapability;
use crate::core::formats::provider::FormatProvider;
use async_trait::async_trait;
use std::path::Path;

/// Provider for archive formats (.clip, .zip, .cbz).
pub struct ArchiveFormatProvider;

/// Implementation of `ArchiveFormatProvider`.
impl ArchiveFormatProvider {
    /// Create a new instance of `ArchiveFormatProvider`.
    pub fn new() -> Self {
        Self
    }
}

/// Implementation of `FormatProvider` for `ArchiveFormatProvider`.
impl FormatProvider for ArchiveFormatProvider {
    /// Returns the name of the format provider.
    fn name(&self) -> &'static str {
        "ARCHIVE_FORMAT_PROVIDER"
    }

    /// Returns the supported extensions for the format provider.
    ///
    /// # Returns
    ///
    /// * `Vec<&'static str>` - The supported extensions.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["zip", "cbz", "clip"]
    }

    /// Returns whether the format provider supports the given magic bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The magic bytes to check.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the format provider supports the given magic bytes.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // ZIP magic: PK
        header_bytes.starts_with(b"PK\x03\x04") || header_bytes.starts_with(b"CSFCHUNK")
    }

    /// Returns the thumbnail capability for the format provider.
    ///
    /// # Returns
    ///
    /// * `Option<&dyn ThumbnailCapability>` - The thumbnail capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implementation of `ThumbnailCapability` for `ArchiveFormatProvider`.
#[async_trait]
impl ThumbnailCapability for ArchiveFormatProvider {
    /// Generate a thumbnail for the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `size_hint` - The size hint for the thumbnail.
    ///
    /// # Returns
    ///
    /// * `AppResult<Vec<u8>>` - The thumbnail of the file.
    async fn generate(&self, path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let extension = path_owned
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if extension == "clip" {
                extract_clip_thumbnail(&path_owned)
            } else {
                extract_zip_thumbnail(&path_owned, size_hint)
            }
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

/// Helper: Extract thumbnail from CLIP Studio file.
/// Based on legacy `clip.rs`.
///
/// # Arguments
///
/// * `path` - The path to the file.
///
/// # Returns
///
/// * `AppResult<Vec<u8>>` - The thumbnail of the file.
fn extract_clip_thumbnail(path: &Path) -> AppResult<Vec<u8>> {
    use byteorder::{BigEndian, ReadBytesExt};
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).map_err(crate::core::error::AppError::Io)?;

    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)
        .map_err(crate::core::error::AppError::Io)?;
    if &magic != b"CSFCHUNK" {
        return Err(crate::core::error::AppError::Generic(
            "Invalid CLIP header".into(),
        ));
    }

    file.seek(SeekFrom::Current(16))
        .map_err(crate::core::error::AppError::Io)?;

    let mut sqlite_data = None;
    loop {
        let mut chunk_name = [0u8; 8];
        if file.read_exact(&mut chunk_name).is_err() {
            break;
        }

        let length = file
            .read_u64::<BigEndian>()
            .map_err(crate::core::error::AppError::Io)?;
        let start = file
            .stream_position()
            .map_err(crate::core::error::AppError::Io)?;

        if &chunk_name == b"CHNKSQLi" {
            let mut data = vec![0u8; length as usize];
            file.read_exact(&mut data)
                .map_err(crate::core::error::AppError::Io)?;
            sqlite_data = Some(data);
            break;
        }

        if &chunk_name == b"CHNKFoot" {
            break;
        }
        file.seek(SeekFrom::Start(start + length))
            .map_err(crate::core::error::AppError::Io)?;
    }

    let db_bytes = sqlite_data
        .ok_or_else(|| crate::core::error::AppError::Generic("CLIP missing SQL chunk".into()))?;

    // SQLite extraction requires a file path for SQLx
    let temp_path =
        std::env::temp_dir().join(format!("mundam_clip_{}.sqlite", uuid::Uuid::new_v4()));
    std::fs::write(&temp_path, db_bytes).map_err(crate::core::error::AppError::Io)?;

    let result = tauri::async_runtime::block_on(async {
        use sqlx::sqlite::SqlitePoolOptions;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", temp_path.display()))
            .await
            .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

        let query: (Vec<u8>,) = sqlx::query_as("SELECT ImageData FROM CanvasPreview LIMIT 1")
            .fetch_one(&pool)
            .await
            .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

        pool.close().await;
        Ok(query.0)
    });

    let _ = std::fs::remove_file(&temp_path);
    result
}

/// Helper: Extract thumbnail from regular ZIP/CBZ.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `size_hint` - The size hint for the thumbnail.
///
/// # Returns
///
/// * `AppResult<Vec<u8>>` - The thumbnail of the file.
fn extract_zip_thumbnail(path: &Path, size_hint: u32) -> AppResult<Vec<u8>> {
    use std::io::Read;

    let file = std::fs::File::open(path).map_err(crate::core::error::AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;

    let preview_paths = [
        "preview.png",
        "Thumbnails/thumbnail.png",
        "QuickLook/Preview.png",
        "QuickLook/Thumbnail.png",
        "icon.png",
    ];

    for p in &preview_paths {
        if let Ok(mut entry) = archive.by_name(p) {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(crate::core::error::AppError::Io)?;

            let img = image::load_from_memory(&buf)
                .map_err(|e| crate::core::error::AppError::Generic(e.to_string()))?;
            return super::raw_format::process_and_encode_webp(img, size_hint);
        }
    }

    Err(crate::core::error::AppError::Generic(
        "No preview found in ZIP".into(),
    ))
}
