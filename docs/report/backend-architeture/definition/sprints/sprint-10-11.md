# Sprint 10.11: Color Worker — Corrigir Erros de Análise de Paleta

**Status da sprint:** Concluído
**Data e hora de inicio da sprint:** 2026-03-20T11:15:00-03:00
**Data e hora da conclusão da sprint:** 2026-03-20T22:55:00-03:00

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

**Status:** Concluído

Os providers que chamam `extract_*_preview()` e retornam RGBA raw sem encoding incluem:
- `BinaryDesignFormatProvider` — extractors de SAI, XCF, CLIP retornam PNG (correto)
- `SvgFormatProvider` — pode retornar PNG via resvg (correto)
- `PsdFormatProvider` — pode retornar PNG ou bytes raw

**Verificar via grep:**
```bash
grep -rn "Ok((.*rgba\|raw\|pixels" src-tauri/src/processing/media/extractors/
```

### 2. Padronizar Retorno dos Extractors para PNG Válido

**Status:** Concluído

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

**Status:** Concluído

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

**Status:** Concluído

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

**Status:** Concluído

Para assets que já têm thumbnails corrompidos no banco, adicionar um mecanismo de re-geração:

**Opção A:** Comando IPC `reprocess_thumbnails` que limpa o campo `thumbnail_path` para assets com thumbnails inválidos, fazendo o `ThumbnailWorker` regenerá-los na próxima iteração.

**Opção B:** Na inicialização, escanear o diretório de thumbnails e marcar como inválidos os que têm cabeçalho errado.

## Arquivos a Modificar

- `src-tauri/src/processing/workers/thumbnail_worker.rs` — validação de header
- `src-tauri/src/processing/workers/color_worker.rs` — graceful skip para thumbnails inválidos
- `src-tauri/src/processing/media/extractors/*.rs` — normalizar retorno para PNG válido

## Informações da Implementação

### Soluções e Melhorias Realizadas
- **Correção Crítica de Infraestrutura (Batch Event Bug)**: Descobrimos um bug fundamental no `ledger.rs` onde comandos executados em lote (batch) emitiam todos os seus eventos de domínio usando apenas o ID do último asset processado. Isso causava um "event storm" de extração para o último asset do lote e deixava os demais sem nenhum evento. Refatoramos o `execute` para expandir e associar cada evento ao seu asset correspondente.
- **Resiliência a Thumbnails Inválidos**: Implementamos uma camada de defesa em `image_utils.rs` que detecta se os bytes de uma imagem são válidos antes de qualquer processamento pesado. Isso protege tanto o `ThumbnailWorker` (ao criar) quanto o `ColorWorker` (ao consumir).
- **Auto-Cura**: Em vez de um comando manual, o `ColorWorker` agora detecta thumbnails corrompidos em tempo real, deleta o arquivo inválido e pula o processamento. Isso força o sistema a regenerar a thumbnail corretamente no próximo ciclo.
- **Resolução de Loop Infinito**: Identificamos que o frontend (`Thumbnail.tsx`) entrava em loop de regeneração porque o protocolo `thumb://` não estava registrado no backend. Registramos o protocolo em `lib.rs` apontando para o handler de assets, resolvendo a causa raiz do "event storm".
- **Extração Universal de Cores**: Refinamos a lógica do `ColorWorker` para sempre extrair cores a partir da thumbnail gerada (WebP), garantindo paridade com a V1 mesmo em formatos que não possuem um `ThumbnailProvider` nativo.
- **Limpeza e Qualidade**: O código foi revisado para remover warnings de variáveis não utilizadas e imports desnecessários no `ColorWorker.rs`, mantendo o padrão de build limpo (clippy-clean).
- **Resolução de Loop de Processamento (Backend & Frontend)**: Implementamos travas no `ThumbnailWorker` (pular existentes) e filtros no frontend (`useVirtualViewport`, `VirtualListView`) para garantir que assets só sejam processados uma vez, eliminando o consumo excessivo de CPU e escrita em disco.
- **Sincronização UI Auto-Refresh**: Criamos um listener de eventos Tauri (`extraction:completed`) no `ColorPaletteSection.tsx`. A UI agora reflete as cores extraídas em tempo real sem necessidade de trocar de asset manualmente.
- **Resiliência do Componente Thumbnail**: Adicionamos um limite de 2 tentativas de regeneração no frontend para evitar loops infinitos em caso de erros persistentes.
- **Aumento de Clusters Cromáticos**: Elevamos o `DEFAULT_CLUSTER_COUNT` para 24 para garantir paridade visual com os detalhes de paleta da V1.

### Desvios e Decisões de Escopo
- **Expansão de Comandos em Lote**: O escopo inicial era focado apenas no `ColorWorker`. No entanto, a descoberta do bug no `Ledger` exigiu uma refatoração da infraestrutura de eventos que afetou também o `BatchCreate` e operações de tags em lote, garantindo que o sistema como um todo se comporte de forma previsível.
- **Dados Raw vs Transcoding**: Identificamos que alguns providers retornavam RGBA raw que quebrava o decodificador WebP. Em vez de consertar individualmente cada extractor (devido ao alto risco de regressão), centralizamos a validação e transcodificação forçada no `ThumbnailWorker`.
- **Payload de Evento**: Expandimos o `DomainEvent::ThumbnailGenerated` para carregar o `format` e o `path` do asset, otimizando a performance do `ColorWorker` e do `thumbnailStore.ts` ao evitar consultas repetitivas ao banco de dados e disco.

## Critérios de Aceitação

- [x] Zero erros `Invalid Chunk header` no terminal após indexação completa
- [x] ColorWorker não crasha para nenhum tipo de arquivo
- [x] Loop infinito de `UPDATE_ASSET_COLORS` resolvido (causa: missing `thumb://` protocol)
- [x] Thumbnails de todos os formatos são WebP válidos
- [x] Paletas de cores são extraídas corretamente para todos os assets (extração via thumbnail)
- [x] UI de paleta atualiza automaticamente após extração (auto-refresh)

## Notas para o Desenvolvedor

> O erro `[52, 49, 46, 46]` = `RIFF` em ASCII — é o início de um arquivo RIFF que pode ser um AVI antigo ou outro formato RIFF. O arquivo provavelmente foi salvo pelo `ThumbnailCapability.generate()` de um format provider que retorna o arquivo original em vez de uma thumbnail processada.

> O `ColorWorker` V2 é arquiteturalmente superior ao V1 porque usa o Event Bus (`ThumbnailGenerated`) em vez de polling. Não alterar esse design — apenas adicionar resiliência a entradas inválidas e garantir que o protocolo de transporte (`asset://localhost/id?type=thumb`) esteja presente e correto.

## Arquivos Modificados
- `src-tauri/src/core/events/payloads.rs` (Atualização de payload de evento)
- `src-tauri/src/core/formats/registry.rs` (Suporte a resolução de formato por nome)
- `src-tauri/src/infra/database/ledger.rs` (Emissão de eventos enriquecidos)
- `src-tauri/src/processing/media/image_utils.rs` (Utilitários de validação de integridade)
- `src-tauri/src/processing/media/mod.rs` (Exportação de novos módulos utilitários)
- `src-tauri/src/processing/workers/color_worker.rs` (Lógica de validação, auto-cura e extração universal)
- `src-tauri/src/processing/workers/thumbnail_worker.rs` (Defesa contra corrupção e transcoding bypass)
- `src-tauri/src/processing/workers/mod.rs` (Registro de dependências internas)
- `src-tauri/src/delivery/tauri/commands/mutations.rs` (Novo comando RPC `verify_thumbnails`)
- `src-tauri/src/delivery/protocols/asset.rs` (Correção do protocolo `asset://` e parsing de ID)
- `src-tauri/src/lib.rs` (Registro do protocolo `asset://`, ponte de eventos e injeção de dependência)
- `src-tauri/src/feature/analysis/colors.rs` (Aumento de clusters para 24)
- `src/core/hooks/useVirtualViewport.ts` (Filtro de visibilidade para thumbnails)
- `src/components/features/viewport/layouts/VirtualListView.tsx` (Filtro de visibilidade na visualização de lista)
- `src/components/features/inspector/image/ColorPaletteSection.tsx` (Auto-refresh via eventos Tauri)
- `src/components/features/viewport/assets/Thumbnail.tsx` (Retry limit e tratamento de erros)
- `src/core/store/thumbnailStore.ts` (Sincronização global de estado de thumbnails)

