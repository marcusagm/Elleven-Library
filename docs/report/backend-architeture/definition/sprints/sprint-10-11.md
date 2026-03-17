# Sprint 10.11: Color Worker — Corrigir Erros de Análise de Paleta

**Status da sprint:** Pendente
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Corrigir os erros persistentes do `ColorWorker` identificados na sprint 9.1, onde arquivos de certos formatos gerem thumbnails com header inválido que quebram a extração de paleta de cores.

## Problema Identificado

Sprint 9.1 documentou o seguinte erro no terminal:

```shell
2026-03-12T11:10:14.259671Z ERROR mundam_lib::processing::workers::color_worker:
WORKER: Color analysis failed for asset 3ed1c21a-88ce-4454-9813-d5b82b632989:
Internal state error: Failed to open thumbnail for color analysis:
Format error decoding WebP: Invalid Chunk header: [52, 49, 46, 46]
```

**Diagnóstico:** Os bytes `[52, 49, 46, 46]` correspondem a ASCII `R`, `1`, `.`, `.` — não é um WebP válido. O `ThumbnailWorker` está salvando arquivos com extensão `.webp` que na verdade contêm dados PNG, JPEG ou raw RGBA sem transcodificação.

## Estado Atual

### ThumbnailWorker V2 (`thumbnail_worker.rs`, linha 224-250)

O worker converte os bytes para WebP via `spawn_blocking`:
```rust
let final_bytes_res = tokio::task::spawn_blocking(move || {
    let img = image::load_from_memory(&bytes)?;
    let encoder = webp::Encoder::from_image(&img)?;
    Ok::<Vec<u8>, AppError>(encoder.encode(75.0).to_vec())
}).await;
```

**Isso deveria funcionar** para qualquer formato que `image::load_from_memory` suporte (PNG, JPEG, BMP, etc.).

**Causa provável:** Alguns `FormatProviders` retornam dados raw (RGBA32 buffer) sem encoding, que `image::load_from_memory` não consegue decodificar diretamente. O erro ocorre antes do `webp::Encoder`.

## Tarefas

### 1. Identificar Quais Providers Retornam Bytes Raw

**Status:** Pendente

Os providers que chamam `extract_*_preview()` e retornam RGBA raw sem encoding incluem:
- `BinaryDesignFormatProvider` — extractors de SAI, XCF, CLIP retornam PNG (correto)
- `SvgFormatProvider` — pode retornar PNG via resvg (correto)
- `PsdFormatProvider` — pode retornar PNG ou bytes raw

**Verificar via grep:**
```bash
grep -rn "Ok((.*rgba\|raw\|pixels" src-tauri/src/processing/media/extractors/
```

### 2. Padronizar Retorno dos Extractors para PNG Válido

**Status:** Pendente

Cada extractor deve garantir que retorna **PNG válido** (ou JPEG) — nunca bytes raw RGBA.

Para extractors que retornam raw, adicionar encoding PNG antes de retornar:

```rust
// Antes (problemático): retorna RGBA raw
return Ok((rgba_bytes, "image/raw".to_string()));

// Depois (correto): encoda como PNG
let mut png_output = Vec::new();
image::codecs::png::PngEncoder::new(std::io::Cursor::new(&mut png_output))
    .write_image(&rgba_bytes, width, height, image::ExtendedColorType::Rgba8)?;
return Ok((png_output, "image/png".to_string()));
```

### 3. Adicionar Validação de Header no ThumbnailWorker

**Status:** Pendente

Adicionar verificação defensiva antes de chamar `image::load_from_memory`:

```rust
// src-tauri/src/processing/workers/thumbnail_worker.rs

fn is_valid_image_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 4 { return false; }
    // PNG: 89 50 4E 47
    if bytes.starts_with(b"\x89PNG") { return true; }
    // JPEG: FF D8 FF
    if bytes.starts_with(b"\xff\xd8\xff") { return true; }
    // BMP: 42 4D
    if bytes.starts_with(b"BM") { return true; }
    // WebP: 52 49 46 46 ... 57 45 42 50
    if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" { return true; }
    // GIF: 47 49 46
    if bytes.starts_with(b"GIF") { return true; }
    false
}

// No spawn_blocking:
if !is_valid_image_bytes(&bytes) {
    error!("ThumbnailWorker: Invalid image bytes for {}: header {:?}", id, &bytes[..4.min(bytes.len())]);
    return Err(AppError::Internal(format!("Invalid image data for {}", id)));
}
```

### 4. Corrigir ColorWorker — Lidar com Thumbnails Corrompidos

**Status:** Pendente

O `ColorWorker` deve lidar graciosamente com thumbnails que não consegue carregar:

```rust
// src-tauri/src/processing/workers/color_worker.rs

// Evento ThumbnailGenerated recebido
if let DomainEvent::ThumbnailGenerated { asset_id, path } = event {
    // Verificar se o arquivo existe antes de tentar analisar
    if !thumb_path.exists() {
        error!("WORKER: Thumbnail file not found at {:?}", thumb_path);
        continue;
    }

    // Verificar se é um WebP válido antes de passar para o analisador
    let header_check = tokio::fs::read(&thumb_path).await
        .map(|bytes| bytes.starts_with(b"RIFF"));

    if !matches!(header_check, Ok(true)) {
        error!("WORKER: Invalid WebP for asset {}. Skipping color analysis.", asset_id);
        continue;
    }

    // Continuar com análise normal
}
```

### 5. Re-processar Thumbnails Corrompidos Existentes

**Status:** Pendente

Para assets que já têm thumbnails corrompidos no banco, adicionar um mecanismo de re-geração:

**Opção A:** Comando IPC `reprocess_thumbnails` que limpa o campo `thumbnail_path` para assets com thumbnails inválidos, fazendo o `ThumbnailWorker` regenerá-los na próxima iteração.

**Opção B:** Na inicialização, escanear o diretório de thumbnails e marcar como inválidos os que têm cabeçalho errado.

## Arquivos a Modificar

- `src-tauri/src/processing/workers/thumbnail_worker.rs` — validação de header
- `src-tauri/src/processing/workers/color_worker.rs` — graceful skip para thumbnails inválidos
- `src-tauri/src/processing/media/extractors/*.rs` — normalizar retorno para PNG válido

## Critérios de Aceitação

- [ ] Zero erros `Invalid Chunk header` no terminal após indexação completa
- [ ] ColorWorker não crasha para nenhum tipo de arquivo
- [ ] Thumbnails de todos os formatos são WebP válidos
- [ ] Paletas de cores são extraídas corretamente para imagens, projetos e designs

## Notas para o Desenvolvedor

> O erro `[52, 49, 46, 46]` = `RIFF` em ASCII — é o início de um arquivo RIFF que pode ser um AVI antigo ou outro formato RIFF. O arquivo provavelmente foi salvo pelo `ThumbnailCapability.generate()` de um format provider que retorna o arquivo original em vez de uma thumbnail processada.

> O `ColorWorker` V2 é arquiteturalmente superior ao V1 porque usa o Event Bus (`ThumbnailGenerated`) em vez de polling. Não alterar esse design — apenas adicionar resiliência a entradas inválidas.
