use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::instrument;

/// Provider for Krita files (.kra).
///
/// This provider handles Krita documents by parsing the internal ZIP structure.
/// It extracts technical metadata from `maindoc.xml` and semantic metadata from
/// `documentinfo.xml`. Previews are extracted from `mergedimage.png` (high-res)
/// or `preview.png` (standard).
#[derive(Default)]
pub struct KritaFormatProvider;

impl KritaFormatProvider {
    /// Creates a new instance of `KritaFormatProvider`.
    ///
    /// # Returns
    ///
    /// `KritaFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }

    /// Internal helper to extract the string content of an XML file from the Krita ZIP archive.
    ///
    /// # Arguments
    ///
    /// * `archive` - The opened Krita ZIP archive.
    /// * `file_name` - The name of the XML file to extract (e.g., "maindoc.xml").
    ///
    /// # Returns
    ///
    /// `Option<String>` - The content of the file if found and successfully read.
    fn extract_xml_content(archive: &mut zip::ZipArchive<File>, file_name: &str) -> Option<String> {
        let mut entry = archive.by_name(file_name).ok()?;
        let mut content_string = String::new();
        entry.read_to_string(&mut content_string).ok()?;
        Some(content_string)
    }
}

impl FormatProvider for KritaFormatProvider {
    /// Returns the unique name for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "KRITA_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["kra"]
    }

    /// Returns the detailed format definitions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<SupportedFormat>` - List of supported formats with metadata.
    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{
            MediaType, PlaybackStrategy, PreviewStrategy, ThumbnailStrategy,
        };

        vec![SupportedFormat::with_metadata(
            "Krita Document",
            vec!["kra"],
            vec!["application/x-krita"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file header matches the ZIP magic bytes.
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid Krita file.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        header_bytes.starts_with(b"PK\x03\x04")
    }

    /// Returns the metadata extraction capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - The metadata extraction capability.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Returns the thumbnail generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - The thumbnail generation capability.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }

    /// Returns the preview generation capability.
    ///
    /// # Returns
    ///
    /// `Option<&dyn PreviewCapability>` - The preview generation capability.
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl MetadataCapability for KritaFormatProvider {
    /// Extracts technical metadata like dimensions, resolution, and layer count from `maindoc.xml`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Krita file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If the ZIP or XML parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let mut image_width = 0;
            let mut image_height = 0;
            let mut resolution_x = 0.0;
            let mut resolution_y = 0.0;
            let mut color_space = String::new();
            let mut krita_version = String::new();
            let mut total_layers = 0;

            if let Some(xml_content) = Self::extract_xml_content(&mut archive, "maindoc.xml") {
                let mut xml_reader = quick_xml::Reader::from_str(&xml_content);
                let mut event_buffer = Vec::new();
                loop {
                    match xml_reader.read_event_into(&mut event_buffer) {
                        Ok(quick_xml::events::Event::Start(ref element))
                        | Ok(quick_xml::events::Event::Empty(ref element)) => {
                            let element_name = element.name();
                            if element_name.as_ref() == b"IMAGE" {
                                for attribute in element.attributes().flatten() {
                                    let key = attribute.key.as_ref();
                                    let value = attribute.unescape_value().unwrap_or_default();
                                    match key {
                                        b"width" => image_width = value.parse().unwrap_or(0),
                                        b"height" => image_height = value.parse().unwrap_or(0),
                                        b"x-res" => resolution_x = value.parse().unwrap_or(0.0),
                                        b"y-res" => resolution_y = value.parse().unwrap_or(0.0),
                                        b"colorspacename" => color_space = value.into_owned(),
                                        _ => {}
                                    }
                                }
                            } else if element_name.as_ref() == b"DOC" {
                                for attribute in element.attributes().flatten() {
                                    if attribute.key.as_ref() == b"kritaVersion" {
                                        krita_version = attribute
                                            .unescape_value()
                                            .unwrap_or_default()
                                            .into_owned();
                                    }
                                }
                            } else if element_name.as_ref() == b"layer" {
                                total_layers += 1;
                            }
                        }
                        Ok(quick_xml::events::Event::Eof) => break,
                        Err(_) => break,
                        _ => {}
                    }
                    event_buffer.clear();
                }
            }

            Ok(json!({
                "width": image_width,
                "height": image_height,
                "resolution_x": resolution_x,
                "resolution_y": resolution_y,
                "color_space": color_space,
                "krita_version": krita_version,
                "layer_count": total_layers,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata like title and author from `documentinfo.xml`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Krita file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If the ZIP or XML parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let mut document_title = String::new();
            let mut document_author = String::new();
            let mut document_description = String::new();
            let mut creation_timestamp = String::new();

            if let Some(xml_content) = Self::extract_xml_content(&mut archive, "documentinfo.xml") {
                let mut xml_reader = quick_xml::Reader::from_str(&xml_content);
                let mut event_buffer = Vec::new();
                let mut current_tag_name = String::new();
                loop {
                    match xml_reader.read_event_into(&mut event_buffer) {
                        Ok(quick_xml::events::Event::Start(ref element)) => {
                            current_tag_name =
                                String::from_utf8_lossy(element.name().as_ref()).to_string();
                        }
                        Ok(quick_xml::events::Event::Text(text_event)) => {
                            let text_content =
                                text_event.unescape().unwrap_or_default().into_owned();
                            match current_tag_name.as_str() {
                                "title" => document_title = text_content,
                                "initial-creator" | "full-name" => {
                                    if document_author.is_empty() {
                                        document_author = text_content
                                    }
                                }
                                "description" => document_description = text_content,
                                "creation-date" => creation_timestamp = text_content,
                                _ => {}
                            }
                        }
                        Ok(quick_xml::events::Event::End(_)) => {
                            current_tag_name = String::new();
                        }
                        Ok(quick_xml::events::Event::Eof) => break,
                        Err(_) => break,
                        _ => {}
                    }
                    event_buffer.clear();
                }
            }

            Ok(json!({
                "title": document_title,
                "author": document_author,
                "description": document_description,
                "creation_date": creation_timestamp,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl ThumbnailCapability for KritaFormatProvider {
    /// Generates a thumbnail by extracting the `preview.png` file from the Krita archive.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Krita file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Hint for the desired thumbnail size (currently unused).
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - The thumbnail image data as bytes.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If reading from the file fails.
    /// * `AppError::Generic` - If the ZIP archive is invalid.
    /// * `AppError::FormatNotSupported` - If no preview image is found in the archive.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let mut zip_entry = archive.by_name("preview.png").map_err(|_| {
                crate::core::error::AppError::FormatNotSupported(
                    "No preview.png found in Krita file".into(),
                )
            })?;

            let mut image_data = Vec::new();
            zip_entry
                .read_to_end(&mut image_data)
                .map_err(crate::core::error::AppError::Io)?;
            Ok(image_data)
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl PreviewCapability for KritaFormatProvider {
    /// Generates a high-resolution preview by extracting `mergedimage.png` or `preview.png`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Krita file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::FormatNotSupported` - If no preview image is found.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            // Krita has mergedimage.png which is full-res render
            if let Ok(mut zip_entry) = archive.by_name("mergedimage.png") {
                let mut image_data = Vec::new();
                zip_entry
                    .read_to_end(&mut image_data)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok((image_data, "image/png".to_string()));
            }

            // Fallback to preview.png
            if let Ok(mut zip_entry) = archive.by_name("preview.png") {
                let mut image_data = Vec::new();
                zip_entry
                    .read_to_end(&mut image_data)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok((image_data, "image/png".to_string()));
            }

            Err(crate::core::error::AppError::FormatNotSupported(
                "No preview image found in Krita file".into(),
            ))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
