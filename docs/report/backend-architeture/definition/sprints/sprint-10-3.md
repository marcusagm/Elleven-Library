# Sprint 10.3: Migração Completa — Extractor SAI (PaintTool SAI v1)

**Status da sprint:** Concluída
**Data e hora de inicio da sprint:** 2026-03-25T08:27:00-03:00
**Data e hora da conclusão da sprint:** 2026-03-25T11:05:51-03:00

## Objetivo

Verificar e garantir paridade completa do extractor de PaintTool SAI v1 (`.sai`) entre V1 e V2, incluindo thumbnail, metadados técnicos (dimensões) e preview.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/sai.rs`
- **Tamanho:** 25,671 bytes (≈280 linhas)
- Implementação completa de decriptação do formato proprietário SAI
- Parse de FAT (File Allocation Table) com decriptação XOR baseada em `USER_KEY[256]`
- Extração do bloco `/thumbnail` (BM32 format) → RGBA32 → PNG
- Validação de checksum por página (4096 bytes)

### V2 — `mundam-main/src-tauri/src/processing/media/extractors/sai.rs`
- **Tamanho:** 13,614 bytes (283 linhas)
- ✅ Algoritmo de decriptação XOR completo com `USER_KEY[256]`
- ✅ `SaiPageReader` com cache de page table
- ✅ FAT entry parse com `FatEntryType` enum
- ✅ `extract_sai_preview()` extrai thumbnail BM32 → PNG
- ⚠️ **MetadataCapability de `BinaryDesignFormatProvider` retorna `{}`** — dimensões não são extraídas

## Análise de Gap

| Funcionalidade | V1 | V2 |
|---|---|---|
| Decriptação XOR FAT | ✅ | ✅ |
| Extract thumbnail BM32→PNG | ✅ | ✅ |
| Validação de checksum | ✅ | ✅ |
| Extract dimensões (width, height) | ✅ | ❌ |
| MetadataCapability com dimensões | ✅ | ❌ (retorna `{}`) |
| Suporte a SAI2 separado | ✅ | ✅ (sai2.rs separado) |

## Tarefas

### 1. Extrair Dimensões do Thumbnail SAI

**Status:** Concluído

O thumbnail BM32 no arquivo SAI contém width e height nos primeiros 8 bytes do bloco. A V2 já lê esses dados em `extract_sai_preview()`, mas descarta os valores numéricos.

**Implementação — criar `extract_sai_metadata()`:**

```rust
// src-tauri/src/processing/media/extractors/sai.rs

/// Extrai apenas as dimensões do thumbnail do arquivo SAI.
pub fn extract_sai_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mut reader = SaiPageReader::new(file)?;
    let entry = find_root_entry(&mut reader, "thumbnail")?
        .ok_or(SaiError::ThumbnailNotFound)?;
    let raw = reader.read_file_data(entry.page_index as usize, entry.size as usize)?;
    if raw.len() < 12 { return Err(SaiError::InvalidThumbnailMagic.into()); }
    let width = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let height = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
    Ok((width, height))
}
```

### 2. Implementar MetadataCapability Real no BinaryDesignFormatProvider

**Status:** Concluído

**Arquivo:** `src-tauri/src/processing/media/binary_design_formats.rs`

```rust
#[async_trait]
impl MetadataCapability for BinaryDesignFormatProvider {
    async fn extract_technical(&self, path: &Path) -> AppResult<serde_json::Value> {
        let extension = path.extension()
            .and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        let dimensions = match extension.as_str() {
            "sai" => {
                tokio::task::spawn_blocking({
                    let path = path.to_path_buf();
                    move || extract_sai_dimensions(&path).ok()
                }).await.ok().flatten()
            }
            "sai2" => {
                tokio::task::spawn_blocking({
                    let path = path.to_path_buf();
                    move || extract_sai2_dimensions(&path).ok()
                }).await.ok().flatten()
            }
            "xcf" => {
                tokio::task::spawn_blocking({
                    let path = path.to_path_buf();
                    move || extract_xcf_dimensions(&path).ok()
                }).await.ok().flatten()
            }
            _ => None,
        };

        if let Some((width, height)) = dimensions {
            Ok(serde_json::json!({
                "width": width,
                "height": height
            }))
        } else {
            Ok(serde_json::json!({}))
        }
    }

    async fn extract_semantic(&self, _path: &Path) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
}
```

### 3. Adicionar PreviewCapability ao BinaryDesignFormatProvider

**Status:** Concluído

O `asset://localhost/{id}?type=preview` chama `provider.preview().generate_preview()`. O `BinaryDesignFormatProvider` não implementa `PreviewCapability`.

**Implementação:**

```rust
// src-tauri/src/processing/media/binary_design_formats.rs
use crate::core::formats::capabilities::PreviewCapability;

impl FormatProvider for BinaryDesignFormatProvider {
    // ... (existente)
    fn preview(&self) -> Option<&dyn PreviewCapability> {
        Some(self)
    }
}

#[async_trait]
impl PreviewCapability for BinaryDesignFormatProvider {
    async fn generate_preview(&self, path: &Path, _asset_id: &str) -> AppResult<(Vec<u8>, String)> {
        // Reutiliza o mesmo extractor do thumbnail — para esses formatos o preview = thumbnail
        let extension = path.extension()
            .and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        let result = tokio::task::spawn_blocking({
            let path = path.to_path_buf();
            let ext = extension.clone();
            move || match ext.as_str() {
                "sai" => extract_sai_preview(&path).map_err(|e| e.to_string()),
                "sai2" => extract_sai2_preview(&path).map_err(|e| e.to_string()),
                "xcf" => extract_xcf_preview(&path).map_err(|e| e.to_string()),
                "clip" => extract_clip_preview(&path).map_err(|e| e.to_string()),
                "rif" | "riff" => extract_corel_painter_preview(&path).map_err(|e| e.to_string()),
                _ => Err("Unsupported".to_string()),
            }
        }).await.map_err(|e| AppError::Internal(e.to_string()))?
         .map_err(|e| AppError::Internal(e))?;

        Ok(result)
    }
}
```

### 4. Corrigir Resolução de Provedor no Protocolo de Asset

**Status:** Concluído

Durante os testes de preview, foi identificado que a geração de previews na URL `asset://localhost/{id}?type=preview` falhava silenciosamente e entregava o próprio arquivo binário original (`.sai`, `.clip`, etc) para o navegador, que não é capaz de exibi-lo formatado. 

Isso acontecia porque a descoberta do Capability de preview utilizava `get_provider(&format.name)`, onde `format.name` era o nome humanizado (e.g. "PaintTool SAI v1") em vez de o nome registrado do sistema (`BINARY_DESIGN_PROVIDER`).

**Implementação:**
A lógica defeituosa no `src-tauri/src/delivery/protocols/asset.rs` foi substituída para resolver de forma mais simples e garantida acionando diretamente o `registry.inner().resolve()`:

```diff
-        if let Some(format) = registry.inner().detect(&physical_path) {
-            if let Some(provider) = registry.inner().get_provider(&format.name) {
+        if let Some(provider) = registry.inner().resolve(&physical_path, &[]) {
```

## Arquivos a Modificar

- `src-tauri/src/processing/media/extractors/sai.rs` — adicionar `extract_sai_dimensions()`
- `src-tauri/src/processing/media/binary_design_formats.rs` — MetadataCapability + PreviewCapability reais
- `src-tauri/src/delivery/protocols/asset.rs` — Corrigido bug de falha silenciosa na carga de dependências nativas para visualização do preview

## Critérios de Aceitação

- [x] Arquivo `.sai` gera thumbnail correto (PNG com RGBA correto)
- [x] Inspector mostra `width` e `height` corretos do arquivo SAI
- [x] ItemView no modo preview mostra a imagem do arquivo SAI em tamanho original
- [x] Sem erros `Provider does not support metadata extraction` para `.sai`

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/sai.rs`
- `mundam-main/src-tauri/src/thumbnails/extractors/mod.rs` (registro)
