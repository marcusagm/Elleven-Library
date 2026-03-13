use crate::core::error::{AppError, AppResult};
use crate::core::formats::capabilities::{MetadataCapability, ThumbnailCapability};
use crate::core::formats::provider::{FormatProvider, SupportedFormat};
use async_trait::async_trait;
use resvg::usvg;
use std::path::Path;
use std::sync::Arc;
use tiny_skia::Pixmap;
use tracing::instrument;

const FONT_SVG_TEMPLATE: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 400 500\">\
  <rect width=\"400\" height=\"500\" fill=\"#f8f9fa\"/>\
  <text x=\"200\" y=\"220\" font_family=\"{family}\" font-size=\"160\" text-anchor=\"middle\" fill=\"#1f2937\">Aa</text>\
  <text x=\"200\" y=\"330\" font_family=\"{family}\" font-size=\"32\" text-anchor=\"middle\" fill=\"#4b5563\">{family}</text>\
  <text x=\"200\" y=\"380\" font_family=\"{family}\" font-size=\"20\" text-anchor=\"middle\" fill=\"#9ca3af\">ABCDEFGHIJKLMNOPQRSTUVWXYZ</text>\
  <text x=\"200\" y=\"410\" font_family=\"{family}\" font-size=\"20\" text-anchor=\"middle\" fill=\"#9ca3af\">abcdefghijklmnopqrstuvwxyz</text>\
  <text x=\"200\" y=\"440\" font_family=\"{family}\" font-size=\"20\" text-anchor=\"middle\" fill=\"#9ca3af\">0123456789</text>\
</svg>";

/// Provider for Font files (.ttf, .otf, .woff, .woff2).
#[derive(Default)]
pub struct FontFormatProvider;

impl FontFormatProvider {
    /// Cria um novo provedor de formato de imagem.
    ///
    /// # Returns
    ///
    /// `FontFormatProvider` - Novo provedor de formato de imagem.
    pub fn new() -> Self {
        Self
    }
}

/// Implementação do provedor de formato de imagem.
impl FormatProvider for FontFormatProvider {
    /// Nome do provedor.
    ///
    /// # Returns
    ///
    /// `&'static str` - Nome do provedor.
    fn name(&self) -> &'static str {
        "FONT_FORMAT_PROVIDER"
    }

    /// Extensões de arquivos suportadas para fontes.
    ///
    /// # Returns
    ///
    /// `Vec<&'static str>` - Vetor de extensões suportadas.
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ttf", "otf", "woff", "woff2"]
    }

    fn supported_formats(&self) -> Vec<SupportedFormat> {
        use crate::core::formats::types::{MediaType, PlaybackStrategy, PreviewStrategy};

        vec![
            SupportedFormat::with_metadata(
                "TrueType Font",
                vec!["ttf"],
                vec!["font/ttf"],
                MediaType::Font,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "OpenType Font",
                vec!["otf"],
                vec!["font/otf"],
                MediaType::Font,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Web Open Font Format",
                vec!["woff"],
                vec!["font/woff"],
                MediaType::Font,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
            SupportedFormat::with_metadata(
                "Web Open Font Format 2",
                vec!["woff2"],
                vec!["font/woff2"],
                MediaType::Font,
                PreviewStrategy::NativeExtractor,
                PlaybackStrategy::None,
            ),
        ]
    }

    /// Verifica se o provedor suporta magic bytes específicos.
    ///
    /// # Arguments
    ///
    /// `header_bytes` - Bytes do cabeçalho do arquivo.
    ///
    /// # Returns
    ///
    /// `bool` - True se o provedor suporta os magic bytes, false caso contrário.
    fn supports_magic_bytes(&self, header_bytes: &[u8]) -> bool {
        // TTF/OTF magic bytes or common headers
        header_bytes.starts_with(&[0, 1, 0, 0, 0]) || // TTF
        header_bytes.starts_with(b"OTTO") ||         // OTF
        header_bytes.starts_with(b"wOFF") // WOFF/WOFF2
    }

    /// Retorna o provedor de metadados.
    ///
    /// # Returns
    ///
    /// `Option<&dyn MetadataCapability>` - Provedor de metadados.
    fn metadata(&self) -> Option<&dyn MetadataCapability> {
        Some(self)
    }

    /// Retorna o provedor de thumbnail.
    ///
    /// # Returns
    ///
    /// `Option<&dyn ThumbnailCapability>` - Provedor de thumbnail.
    fn thumbnail(&self) -> Option<&dyn ThumbnailCapability> {
        Some(self)
    }
}

/// Implementação da capacidade de metadados.
#[async_trait]
impl MetadataCapability for FontFormatProvider {
    /// Extrai metadados técnicos do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Metadados técnicos do arquivo.
    #[instrument(skip(self, path))]
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let font_data = std::fs::read(&path_owned).map_err(AppError::Io)?;
            let face = ttf_parser::Face::parse(&font_data, 0)
                .map_err(|e| AppError::Generic(format!("Font parse error: {:?}", e)))?;

            let mut family_name = None;
            for name in face.names() {
                if name.name_id == ttf_parser::name_id::FULL_NAME {
                    family_name = name.to_string();
                    break;
                }
            }

            Ok(serde_json::json!({
                "family": family_name,
                "is_bold": face.is_regular(),
                "units_per_em": face.units_per_em(),
            }))
        })
        .await
        .map_err(|_| AppError::ExtractionProcessTimeout)?
    }

    /// Extrai metadados semânticos do arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    ///
    /// # Returns
    ///
    /// `AppResult<serde_json::Value>` - Metadados semânticos do arquivo.
    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}

/// Implementação da capacidade de thumbnail.
#[async_trait]
impl ThumbnailCapability for FontFormatProvider {
    /// Gera uma thumbnail para o arquivo.
    ///
    /// # Arguments
    ///
    /// `path` - Caminho do arquivo.
    /// `size_hint` - Hint de tamanho para a thumbnail.
    ///
    /// # Returns
    ///
    /// `AppResult<Vec<u8>>` - Thumbnail do arquivo.
    #[instrument(skip(self, path))]
    async fn generate(&self, path: &Path, _asset_id: &str, size_hint: u32) -> AppResult<Vec<u8>> {
        let path_owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            // 1. Setup FontDB
            let mut fontdb = usvg::fontdb::Database::new();

            let ext = path_owned
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let data = std::fs::read(&path_owned).map_err(AppError::Io)?;

            if ext == "woff" {
                let decoded = wuff::decompress_woff1(&data)
                    .map_err(|e| AppError::Generic(format!("WOFF1 decode failed: {:?}", e)))?;
                fontdb.load_font_source(usvg::fontdb::Source::Binary(Arc::new(decoded)));
            } else if ext == "woff2" {
                let decoded = wuff::decompress_woff2(&data)
                    .map_err(|e| AppError::Generic(format!("WOFF2 decode failed: {:?}", e)))?;
                fontdb.load_font_source(usvg::fontdb::Source::Binary(Arc::new(decoded)));
            } else {
                fontdb.load_font_source(usvg::fontdb::Source::Binary(Arc::new(data)));
            }

            // 2. Identify the font family name
            let face = fontdb
                .faces()
                .next()
                .ok_or_else(|| AppError::Generic("No font faces found in file".into()))?;

            let family_name = face
                .families
                .first()
                .map(|(name, _)| name.clone())
                .unwrap_or_else(|| face.post_script_name.clone());

            // 3. Prepare options with the custom fontdb
            let opt = usvg::Options {
                fontdb: Arc::new(fontdb),
                ..Default::default()
            };

            // 4. Inject family name into SVG
            let safe_family = family_name.replace("\"", "&quot;").replace("'", "&apos;");
            let svg_content = FONT_SVG_TEMPLATE.replace("{family}", &safe_family);

            // 5. Parse SVG
            let tree = usvg::Tree::from_str(&svg_content, &opt)
                .map_err(|e| AppError::Generic(format!("SVG parse error: {}", e)))?;

            // 6. Calculate scale and render
            let size = tree.size();
            let width = size.width();
            let height = size.height();

            if width == 0.0 || height == 0.0 {
                return Err(AppError::Generic("Invalid SVG dimensions".into()));
            }

            let scale = if width > height {
                size_hint as f32 / width
            } else {
                size_hint as f32 / height
            };

            let target_width = (width * scale).ceil() as u32;
            let target_height = (height * scale).ceil() as u32;

            let mut pixmap = Pixmap::new(target_width, target_height)
                .ok_or_else(|| AppError::Generic("Failed to create pixmap".into()))?;

            let transform = tiny_skia::Transform::from_scale(scale, scale);
            resvg::render(&tree, transform, &mut pixmap.as_mut());

            // 7. Encode to WebP
            let encoder = webp::Encoder::from_rgba(pixmap.data(), target_width, target_height);
            let webp_data = encoder.encode(90.0);
            Ok(webp_data.to_vec())
        })
        .await
        .map_err(|_| AppError::ExtractionProcessTimeout)?
    }
}
