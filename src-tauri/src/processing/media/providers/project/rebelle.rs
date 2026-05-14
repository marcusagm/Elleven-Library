use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use crate::processing::media::extractors;
use async_trait::async_trait;
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::instrument;

/// Provider for Rebelle artwork files (.reb).
///
/// This provider handles Rebelle documents by parsing the internal ZIP structure.
/// It extracts technical metadata from `artwork.xml`, including canvas dimensions,
/// layers, and reference images. Previews are extracted from `canvas.png` or other
/// fallback candidates within the archive.
#[derive(Default)]
pub struct RebelleFormatProvider;

impl RebelleFormatProvider {
    /// Creates a new instance of `RebelleFormatProvider`.
    ///
    /// # Returns
    ///
    /// `RebelleFormatProvider` - A new instance of the provider.
    pub fn new() -> Self {
        Self
    }

    /// Internal helper to extract the string content of an XML file from the Rebelle ZIP archive.
    ///
    /// # Arguments
    ///
    /// * `archive` - The opened Rebelle ZIP archive.
    /// * `file_name` - The name of the XML file to extract (e.g., "artwork.xml").
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

impl FormatProvider for RebelleFormatProvider {
    /// Returns the unique name for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name of the provider.
    fn name(&self) -> &'static str {
        "REBELLE_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["reb"]
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
            "Rebelle Artwork",
            vec!["reb"],
            vec!["application/x-rebelle"],
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
    /// * `header_bytes` - The first 4 bytes of the file to check.
    ///
    /// # Returns
    ///
    /// `bool` - `true` if the file header matches the ZIP magic bytes, `false` otherwise.
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
impl MetadataCapability for RebelleFormatProvider {
    /// Extracts technical metadata like dimensions and layer count from `artwork.xml`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Rebelle file.
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

            let mut canvas_width = 0.0;
            let mut canvas_height = 0.0;
            let mut rebelle_version = String::new();
            let mut total_layers = 0;
            let mut paper_name = String::new();

            if let Some(xml_content) = Self::extract_xml_content(&mut archive, "artwork.xml") {
                let mut xml_reader = quick_xml::Reader::from_str(&xml_content);
                let mut event_buffer = Vec::new();
                loop {
                    match xml_reader.read_event_into(&mut event_buffer) {
                        Ok(quick_xml::events::Event::Start(ref element))
                        | Ok(quick_xml::events::Event::Empty(ref element)) => {
                            let element_name = element.name();
                            match element_name.as_ref() {
                                b"aquarelle_artwork" => {
                                    for attribute in element.attributes().flatten() {
                                        if attribute.key.as_ref() == b"version_str" {
                                            rebelle_version = attribute
                                                .unescape_value()
                                                .unwrap_or_default()
                                                .into_owned();
                                        }
                                    }
                                }
                                b"canvas" => {
                                    for attribute in element.attributes().flatten() {
                                        let key = attribute.key.as_ref();
                                        let value = attribute.unescape_value().unwrap_or_default();
                                        match key {
                                            b"width" => canvas_width = value.parse().unwrap_or(0.0),
                                            b"height" => {
                                                canvas_height = value.parse().unwrap_or(0.0)
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                b"paper" => {
                                    for attribute in element.attributes().flatten() {
                                        if attribute.key.as_ref() == b"name" {
                                            paper_name = attribute
                                                .unescape_value()
                                                .unwrap_or_default()
                                                .into_owned();
                                        }
                                    }
                                }
                                b"layer" => {
                                    total_layers += 1;
                                }
                                _ => {}
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
                "width": canvas_width as u32,
                "height": canvas_height as u32,
                "rebelle_version": rebelle_version,
                "layer_count": total_layers,
                "paper": paper_name,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata like layer names and reference image names.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Rebelle file.
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

            let mut layer_names = Vec::new();
            let mut reference_images = Vec::new();

            if let Some(xml_content) = Self::extract_xml_content(&mut archive, "artwork.xml") {
                let mut xml_reader = quick_xml::Reader::from_str(&xml_content);
                let mut event_buffer = Vec::new();
                loop {
                    match xml_reader.read_event_into(&mut event_buffer) {
                        Ok(quick_xml::events::Event::Start(ref element))
                        | Ok(quick_xml::events::Event::Empty(ref element)) => {
                            let element_name = element.name();
                            match element_name.as_ref() {
                                b"layer" => {
                                    for attribute in element.attributes().flatten() {
                                        if attribute.key.as_ref() == b"name" {
                                            layer_names.push(
                                                attribute
                                                    .unescape_value()
                                                    .unwrap_or_default()
                                                    .into_owned(),
                                            );
                                        }
                                    }
                                }
                                b"reference_image" => {
                                    for attribute in element.attributes().flatten() {
                                        if attribute.key.as_ref() == b"name" {
                                            reference_images.push(
                                                attribute
                                                    .unescape_value()
                                                    .unwrap_or_default()
                                                    .into_owned(),
                                            );
                                        }
                                    }
                                }
                                _ => {}
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
                "layer_names": layer_names,
                "reference_images": reference_images,
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl ThumbnailCapability for RebelleFormatProvider {
    /// Generates a thumbnail for the Rebelle file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Rebelle file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Requested dimension for the thumbnail.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        self.generate_preview(path, asset_id)
            .await
            .map(|(image_data, _mime_type)| image_data)
    }
}

#[async_trait]
impl PreviewCapability for RebelleFormatProvider {
    /// Generates a high-resolution preview from the Rebelle ZIP archive.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Rebelle file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Generic` - If extraction fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            extractors::extract_rebelle_preview(&path_owned)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
