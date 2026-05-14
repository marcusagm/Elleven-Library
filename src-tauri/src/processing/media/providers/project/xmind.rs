use crate::core::error::AppResult;
use crate::core::formats::capabilities::{
    MetadataCapability, PreviewCapability, ThumbnailCapability,
};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::instrument;

/// Provider for XMind mind mapping files (.xmind).
///
/// This provider supports both modern XMind (ZEN/Pro) which uses JSON-based content,
/// and legacy XMind Classic which uses XML-based content. It extracts technical
/// details like topic and sheet counts, as well as semantic data like mind map titles.
#[derive(Default)]
pub struct XMindFormatProvider;

impl XMindFormatProvider {
    /// Creates a new instance of the format provider for XMind.
    ///
    /// # Returns
    ///
    /// `XMindFormatProvider` - A new instance of the format provider for XMind.
    pub fn new() -> Self {
        Self
    }

    /// Recursively counts topics in a JSON-based sheet structure.
    ///
    /// # Arguments
    ///
    /// * `topic` - The root topic JSON value to start counting from.
    ///
    /// # Returns
    ///
    /// `usize` - The total number of topics found.
    fn count_topics_json(topic: &Value) -> usize {
        let mut topic_count = 1;
        if let Some(children) = topic.get("children") {
            if let Some(attached_nodes) =
                children.get("attached").and_then(|value| value.as_array())
            {
                for child_node in attached_nodes {
                    topic_count += Self::count_topics_json(child_node);
                }
            }
        }
        topic_count
    }

    /// Counts topics in an XML-based content string by scanning for `<topic>` tags.
    ///
    /// # Arguments
    ///
    /// * `content` - The XML content string.
    ///
    /// # Returns
    ///
    /// `usize` - The total number of topics found.
    fn count_topics_xml(content: &str) -> usize {
        let mut topic_count = 0;
        let mut xml_reader = quick_xml::Reader::from_str(content);
        let mut event_buffer = Vec::new();
        loop {
            match xml_reader.read_event_into(&mut event_buffer) {
                Ok(quick_xml::events::Event::Start(ref element))
                    if element.name().as_ref() == b"topic" =>
                {
                    topic_count += 1;
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            event_buffer.clear();
        }
        topic_count
    }

    /// Counts sheets in an XML-based content string by scanning for `<sheet>` tags.
    ///
    /// # Arguments
    ///
    /// * `content` - The XML content string.
    ///
    /// # Returns
    ///
    /// `usize` - The total number of sheets found.
    fn count_sheets_xml(content: &str) -> usize {
        let mut sheet_count = 0;
        let mut xml_reader = quick_xml::Reader::from_str(content);
        let mut event_buffer = Vec::new();
        loop {
            match xml_reader.read_event_into(&mut event_buffer) {
                Ok(quick_xml::events::Event::Start(ref element))
                    if element.name().as_ref() == b"sheet" =>
                {
                    sheet_count += 1;
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            event_buffer.clear();
        }
        sheet_count
    }

    /// Extracts the preview image bytes from an XMind ZIP archive using multiple candidate paths.
    ///
    /// # Arguments
    ///
    /// * `archive` - The opened ZIP archive.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If reading from the archive fails.
    /// * `AppError::FormatNotSupported` - If no preview image is found in the archive.
    fn extract_preview_bytes(
        archive: &mut zip::ZipArchive<File>,
    ) -> AppResult<(Vec<u8>, &'static str)> {
        // Comprehensive candidate list for XMind previews (Classic, 8, ZEN, Pro)
        let candidate_paths = [
            "previews/preview.png",
            "Previews/preview.png",
            "Canvas/thumbnail.png",
            "Thumbnails/thumbnail.png",
            "Thumbnail/thumbnail.png",
            "QuickLook/Preview.png",
            "QuickLook/Thumbnail.png",
            "preview.png",
            "thumbnail.png",
            "icon.png",
        ];

        for path in candidate_paths {
            if let Ok(mut entry) = archive.by_name(path) {
                let mut image_data = Vec::new();
                entry
                    .read_to_end(&mut image_data)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok((image_data, "image/png"));
            }
        }

        // Final fallback: search for anything ending with preview.png or thumbnail.png
        for index in 0..archive.len() {
            let Ok(mut entry) = archive.by_index(index) else {
                continue;
            };
            let entry_name = entry.name().to_lowercase();
            if entry_name.ends_with("preview.png") || entry_name.ends_with("thumbnail.png") {
                let mut image_data = Vec::new();
                entry
                    .read_to_end(&mut image_data)
                    .map_err(crate::core::error::AppError::Io)?;
                return Ok((image_data, "image/png"));
            }
        }

        Err(crate::core::error::AppError::FormatNotSupported(
            "No preview found in XMind ZIP archive".into(),
        ))
    }
}

impl FormatProvider for XMindFormatProvider {
    /// Returns the unique name for this provider.
    ///
    /// # Returns
    ///
    /// `&'static str` - The unique name for this provider.
    fn name(&self) -> &'static str {
        "XMIND_PROVIDER"
    }

    /// Returns the file extensions supported by this provider.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - The file extensions supported by this provider.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["xmind"]
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
            "XMind MindMap",
            vec!["xmind"],
            vec!["application/x-xmind"],
            MediaType::Project,
            ThumbnailStrategy::NativeExtractor,
            PreviewStrategy::NativeExtractor,
            PlaybackStrategy::None,
        )]
    }

    /// Validates if the file is a ZIP archive (XMind standard).
    ///
    /// # Arguments
    ///
    /// * `header_bytes` - The first bytes of the file.
    ///
    /// # Returns
    ///
    /// `bool` - True if it's a valid XMind file.
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
impl MetadataCapability for XMindFormatProvider {
    /// Extracts technical metadata like topic counts and app version.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the XMind file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If ZIP or JSON parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let mut sheet_count = 0;
            let mut topic_count = 0;
            let mut app_version = String::new();
            let mut image_width = None;
            let mut image_height = None;

            // 1. Try JSON (Modern ZEN/Pro)
            let mut content_processed = false;
            if let Ok(mut content_file) = archive.by_name("content.json") {
                let mut json_buffer = String::new();
                if content_file.read_to_string(&mut json_buffer).is_ok() {
                    if let Ok(Value::Array(sheets)) = serde_json::from_str::<Value>(&json_buffer) {
                        sheet_count = sheets.len();
                        for sheet in sheets {
                            if let Some(root_topic) = sheet.get("rootTopic") {
                                topic_count += Self::count_topics_json(root_topic);
                            }
                        }
                        content_processed = true;
                    }
                }
            }

            if !content_processed {
                if let Ok(mut content_xml_file) = archive.by_name("content.xml") {
                    // 2. Try XML (Classic)
                    let mut xml_buffer = String::new();
                    if content_xml_file.read_to_string(&mut xml_buffer).is_ok() {
                        sheet_count = Self::count_sheets_xml(&xml_buffer);
                        topic_count = Self::count_topics_xml(&xml_buffer);
                    }
                }
            }

            // Extract version from metadata.json (Modern) or meta.xml (Classic)
            let mut version_processed = false;
            if let Ok(mut metadata_json_file) = archive.by_name("metadata.json") {
                let mut json_buffer = String::new();
                if metadata_json_file.read_to_string(&mut json_buffer).is_ok() {
                    if let Ok(metadata) = serde_json::from_str::<Value>(&json_buffer) {
                        if let Some(creator) =
                            metadata.get("creator").and_then(|v| v.get("version"))
                        {
                            app_version = creator.as_str().unwrap_or("").to_string();
                            version_processed = true;
                        }
                    }
                }
            }

            if !version_processed {
                if let Ok(mut _meta_xml_file) = archive.by_name("meta.xml") {
                    app_version = "Classic".to_string();
                }
            }

            // 3. Extract dimensions from preview image
            if let Ok((image_bytes, _)) = Self::extract_preview_bytes(&mut archive) {
                if let Ok(dimensions) = imagesize::blob_size(&image_bytes) {
                    image_width = Some(dimensions.width as i64);
                    image_height = Some(dimensions.height as i64);
                }
            }

            Ok(serde_json::json!({
                "sheet_count": sheet_count,
                "topic_count": topic_count,
                "app_version": app_version,
                "width": image_width,
                "height": image_height,
                "is_xmind_zen": !app_version.is_empty() && app_version != "Classic"
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }

    /// Extracts semantic metadata like author and mind map titles.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the XMind file.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If the file cannot be read.
    /// * `AppError::Generic` - If parsing fails.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    async fn extract_semantic(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let mut document_author = String::new();
            let mut map_titles = Vec::new();

            // Extract titles from content.json (Modern)
            let mut semantic_processed = false;
            if let Ok(mut content_file) = archive.by_name("content.json") {
                let mut json_buffer = String::new();
                if content_file.read_to_string(&mut json_buffer).is_ok() {
                    if let Ok(Value::Array(sheets)) = serde_json::from_str::<Value>(&json_buffer) {
                        for sheet in sheets {
                            if let Some(title) = sheet.get("title").and_then(|v| v.as_str()) {
                                map_titles.push(title.to_string());
                            } else if let Some(root_topic) =
                                sheet.get("rootTopic").and_then(|v| v.get("title"))
                            {
                                if let Some(title) = root_topic.as_str() {
                                    map_titles.push(title.to_string());
                                }
                            }
                        }
                        semantic_processed = true;
                    }
                }
            }

            if !semantic_processed {
                if let Ok(mut content_xml_file) = archive.by_name("content.xml") {
                    // Extract titles from XML (Classic)
                    let mut xml_buffer = String::new();
                    if content_xml_file.read_to_string(&mut xml_buffer).is_ok() {
                        let mut xml_reader = quick_xml::Reader::from_str(&xml_buffer);
                        let mut event_buffer = Vec::new();
                        let mut is_inside_title = false;
                        loop {
                            match xml_reader.read_event_into(&mut event_buffer) {
                                Ok(quick_xml::events::Event::Start(ref element))
                                    if element.name().as_ref() == b"title" =>
                                {
                                    is_inside_title = true;
                                }
                                Ok(quick_xml::events::Event::Text(text_event))
                                    if is_inside_title =>
                                {
                                    if let Ok(unescaped_text) = text_event.unescape() {
                                        map_titles.push(unescaped_text.into_owned());
                                    }
                                    is_inside_title = false;
                                }
                                Ok(quick_xml::events::Event::End(ref element))
                                    if element.name().as_ref() == b"title" =>
                                {
                                    is_inside_title = false;
                                }
                                Ok(quick_xml::events::Event::Eof) => break,
                                Err(_) => break,
                                _ => {}
                            }
                            event_buffer.clear();
                        }
                    }
                }
            }

            // Extract author from metadata.json (ZEN) or meta.xml (Classic)
            let mut author_processed = false;
            if let Ok(mut metadata_json_file) = archive.by_name("metadata.json") {
                let mut json_buffer = String::new();
                if metadata_json_file.read_to_string(&mut json_buffer).is_ok() {
                    if let Ok(metadata) = serde_json::from_str::<Value>(&json_buffer) {
                        if let Some(author_name) = metadata.get("Author").and_then(|v| v.as_str()) {
                            document_author = author_name.to_string();
                            author_processed = true;
                        }
                    }
                }
            }

            if !author_processed {
                if let Ok(mut meta_xml_file) = archive.by_name("meta.xml") {
                    let mut xml_buffer = String::new();
                    if meta_xml_file.read_to_string(&mut xml_buffer).is_ok() {
                        let mut xml_reader = quick_xml::Reader::from_str(&xml_buffer);
                        let mut event_buffer = Vec::new();
                        let mut is_inside_name_tag = false;
                        loop {
                            match xml_reader.read_event_into(&mut event_buffer) {
                                Ok(quick_xml::events::Event::Start(ref element))
                                    if element.name().as_ref() == b"Name" =>
                                {
                                    is_inside_name_tag = true;
                                }
                                Ok(quick_xml::events::Event::Text(text_event))
                                    if is_inside_name_tag =>
                                {
                                    if let Ok(unescaped_text) = text_event.unescape() {
                                        document_author = unescaped_text.into_owned();
                                    }
                                    break;
                                }
                                Ok(quick_xml::events::Event::End(ref element))
                                    if element.name().as_ref() == b"Name" =>
                                {
                                    is_inside_name_tag = false;
                                }
                                Ok(quick_xml::events::Event::Eof) => break,
                                Err(_) => break,
                                _ => {}
                            }
                            event_buffer.clear();
                        }
                    }
                }
            }

            Ok(serde_json::json!({
                "author": document_author,
                "titles": map_titles,
                "main_title": map_titles.first().cloned().unwrap_or_default()
            }))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}

#[async_trait]
impl ThumbnailCapability for XMindFormatProvider {
    /// Generates a thumbnail for the XMind file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the XMind file.
    /// * `asset_id` - Unique identifier for the asset.
    /// * `size_hint` - Hint for the desired thumbnail size (currently unused).
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - The thumbnail image data as bytes.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, asset_id: &str, _size_hint: u32) -> AppResult<Vec<u8>> {
        // For XMind, preview and thumbnail use the same extraction logic
        self.generate_preview(path, asset_id)
            .await
            .map(|(image_data, _mime_type)| image_data)
    }
}

#[async_trait]
impl PreviewCapability for XMindFormatProvider {
    /// Generates a high-resolution preview from the XMind ZIP archive.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the XMind file.
    /// * `asset_id` - Unique identifier for the asset.
    ///
    /// # Errors
    ///
    /// * `AppError::Io` - If reading from the archive fails.
    /// * `AppError::Generic` - If ZIP or image processing fails.
    /// * `AppError::FormatNotSupported` - If no preview image is found.
    /// * `AppError::ExtractionProcessTimeout` - If the blocking task times out.
    #[instrument(skip(self, path))]
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let file = File::open(path_owned).map_err(crate::core::error::AppError::Io)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|error| crate::core::error::AppError::Generic(error.to_string()))?;

            let (image_data, mime_type) = Self::extract_preview_bytes(&mut archive)?;
            Ok((image_data, mime_type.to_string()))
        })
        .await
        .map_err(|_| crate::core::error::AppError::ExtractionProcessTimeout)?
    }
}
