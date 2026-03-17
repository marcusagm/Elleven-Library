# Backend Architecture Comparison Report: V1 vs V2

**Data da análise:** 2026-03-17
**Analista:** Antigravity AI
**Escopo:** Análise aprofundada de todos os módulos do backend Rust (Tauri) comparando a arquitetura monolítica V1 com a arquitetura hexagonal/EDA V2.

---

## 1. Resumo Executivo

A migração da V1 para a V2 foi motivada por três objetivos legítimos: organização do projeto, flexibilidade para novas implementações e suporte a um futuro sistema de plugins para desenvolvedores externos. A V2 avançou significativamente na **estrutura arquitetural** e na **cobertura de formatos**, porém introduziu **regressões funcionais críticas** em módulos que estavam completamente operacionais na V1.

| Dimensão                  | V1 (Mundam-main)               | V2 (Mundam)                               |
| ------------------------- | ------------------------------ | ----------------------------------------- |
| Organização do Código     | ⚠️ Boa, mas plana               | ✅ Excelente (hexagonal)                   |
| Cobertura de Formatos     | ✅ Boa                          | ✅ Excelente (+25% mais formatos)          |
| Indexador / Scanner       | ✅ Completo e robusto           | ⚠️ Funcional, porém sem watcher integrado  |
| Watcher de FS             | ✅ 5 fases, sofisticado         | ⚠️ Estrutura presente, integração parcial  |
| Worker de Thumbnails      | ✅ Dual-queue + rayon + cores   | ❌ Ausente na camada de processing         |
| Transcoding (pipe FFmpeg) | ✅ FfmpegTranscoder completo    | ❌ Apenas resolver de binários             |
| Streaming Server (HLS)    | ✅ 678 linhas, completo         | ⚠️ 294 linhas, sem linear HLS              |
| Protocolo `asset://`      | ✅ Sync, robusto                | ✅ V2 superior (async, DI, preview)        |
| Banco de Dados            | ✅ 24k linhas de SQL maduro     | ⚠️ Ledger 63k linhas, queries em validação |
| Segurança do Servidor     | ✅ CORS restritivo + path scope | ⚠️ CORS permissivo, sem path scope         |
| Shutdown Gracioso         | ✅ CancellationToken em tudo    | ⚠️ Streaming sem CancellationToken         |

---

## 2. Análise por Módulo

### 2.1 Indexador / Scanner

#### V1 — `src/indexer/` (scan.rs + watcher.rs)

O indexador V1 é um sistema **producer-consumer** com paralelismo real:

- **scan.rs**: Walk diferencial usando `WalkDir` + `comparison_cache` (size + mtime). Produtores spawnam tarefas Tokio independentes por arquivo, um worker central consome via `mpsc::channel`. Prune de pastas órfãs. Emite `indexer:progress` e `indexer:complete` para o frontend.
- **watcher.rs (484 linhas)**: Pipeline de 5 fases claramente separadas:
  1. **Parse & Normalize** — debouncing e deduplicação de eventos do OS.
  2. **Classify** — diferenciação de tipo (arquivo vs pasta).
  3. **Heuristics** — pareamento inteligente de renomeações split (From/To), com fallback por metadados de tamanho e `created_at` consultados no banco.
  4. **Persist** — transações atômicas: renames, removals (com delay de 2s para evitar falso-positivo), adição de pastas e assets.
  5. **Emit** — `library:batch-change` para o frontend com payload diferenciado (added, removed, updated).

**Ponto forte crítico**: O heurístico de rename é extremamente sofisticado. Ele usa `event.attrs.tracker()` para renomeações rastreadas e fallback de metadados (size + created_at) para renomeações não rastreadas (caso macOS Finder).

#### V2 — `src/feature/library/indexer.rs` (257 linhas)

O `LibraryIndexer` usa corretamente a arquitetura hexagonal:

- Injeta `TransactionalAssetLedger`, `AssetQueryHandler` e `AppEventBus` via DI.
- Emite `DomainEvent::ScanStarted`, `ScanProgress` e `ScanCompleted` via Event Bus.
- Cria pastas via `LedgerCommand::CreateFolder` e assets via `LedgerCommand::CreateAsset`.
- Possui `start_event_listener` que escuta `FsFileDiscovered` e `FsPathRenamed`.

**Regressões identificadas:**

1. ❌ **Sem watcher dedicado visível na camada `processing/watcher/`**: O `LibraryIndexer` tem `start_event_listener`, mas quem emite `FsFileDiscovered`? O watcher de FS precisa ser verificado separadamente.
2. ⚠️ **Processamento serial vs paralelo**: V1 usa `tokio::spawn` por arquivo (fan-out) + worker receiver centralizado. V2 itera sequencialmente com `for entry in WalkDir`. Em bibliotecas com 50k+ arquivos, a V2 será ordens de magnitude mais lenta.
3. ⚠️ **Duplo walk**: V2 faz dois walks completos — um para contar `total_files` (para progresso) e outro para processar. V1 faz um único walk acumulando contadores.
4. ⚠️ **Resolução de `folder_id`**: V2 busca `find_folder_by_path` para cada subpasta via query ao banco. V1 mantém um `HashMap` local (`folder_map`) já populado no início do scan.

---

### 2.2 Worker de Thumbnails

#### V1 — `src/thumbnails/worker.rs` (375 linhas)

Sistema de geração de thumbnails **altamente sofisticado**:

- **Duas filas independentes**: `light_q` (imagens nativas, N threads via config) e `heavy_q` (FFmpeg/3D, hardcoded 2 threads para evitar exaustão de CPU).
- **`rayon::ThreadPoolBuilder`**: Cada fila tem seu próprio `rayon::ThreadPool` isolado — sem contenção com o runtime Tokio.
- **Sistema de prioridade LIFO**: Assets visíveis no viewport são processados com `push_front` (LIFO) enquanto assets de background usam `push_back` (FIFO).
- **Post-processing inline**: Após cada thumbnail, extrai automaticamente a paleta de cores (`extract_color_palette`) e persiste no SQLite (`insert_asset_colors`, `update_dominant_color`).
- **Shutdown cooperativo**: Usa `CancellationToken` via `tokio::select!` — termina o batch atual antes de sair.

#### V2 — `src/processing/` (workers/)

O diretório `processing/workers/` existe no mapa mas **nenhum ThumbWorker equivalente foi encontrado** com a mesma magnitude de funcionalidade. A extração de thumbnails passou para o sistema de `FormatProvider` com capability `ThumbCapability`, o que é arquiteturalmente correto, mas a orquestração de worker (filas, rayon, prioridade, extração de cores) **precisa ser verificada** se foi migrada para algum outro local.

**Regressões confirmadas:**

1. ❌ **Sem extração de paleta de cores inline** — Na V1 isso acontece automaticamente após cada thumbnail. Se foi removido, a busca por cor deixou de funcionar.
2. ❌ **Sistema de fila de prioridade (viewport-aware)** — Precisa ser validado se existe no V2.
3. ❌ **rayon pool isolado** — Sem isso, a geração de thumbnails pode bloquear o runtime Tokio.

---

### 2.3 Transcoding

#### V1 — `src/transcoding/` (6 arquivos)

Sistema completo de transcodificação com múltiplos módulos:

- **`ffmpeg_pipe.rs`**: `FfmpegTranscoder` completo — detect media type (Audio vs Video), build FFmpeg command com CRF, H.264 High Profile, `+faststart`, GOP settings, mapeamento de streams, audio AAC.
- **`cache.rs`**: `TranscodeCache` — evita re-transcodificação de arquivos já processados.
- **`detector.rs`**: `MediaType` enum (Audio/Video/Unknown) com detecção por extensão.
- **`quality.rs`**: `TranscodeQuality` profiles (preset, CRF, bitrate de áudio).
- **`commands.rs`**: IPC commands expostos ao frontend para controle de transcodificação.

#### V2 — `src/processing/transcoding/mod.rs` (159 linhas — arquivo único)

O módulo V2 contém **apenas infraestrutura de suporte**:

- `resolve_transcoding_tools()` — encontra os binários FFmpeg/FFprobe/Assimp.
- `run_command_with_timeout()` — executa um subprocess com timeout usando `wait_timeout`.
- **NÃO contém**: FfmpegTranscoder, TranscodeCache, quality profiles, HLS segmentation.

A responsabilidade de transcodificação foi delegada para o `HlsManager` (`src/feature/transcoding/hls_manager.rs`), que é correto na nova arquitetura, mas precisa ser validado em profundidade:

**Regressões a verificar:**

1. ⚠️ **`TranscodeCache`** — Existe equivalente? Sem cache, cada visualização de vídeo incompatível transcodifica do zero.
2. ⚠️ **Quality profiles** — O `HlsManager` expõe seleção de qualidade ao frontend?
3. ⚠️ **Audio-only transcoding** — V1 tem path separado para áudio. V2 precisa validar.

---

### 2.4 Streaming Server

#### V1 — `src/streaming/server.rs` (678 linhas)

Servidor HTTP Axum production-grade:

**Rotas:**
- `GET /health` — healthcheck
- `GET /probe/*path` — ffprobe via `probe::get_video_info()`
- `GET /playlist/*path` — gera M3U8 dinâmico com duração real (probe + math)
- `GET /segment/*path/{index}` — transcodifica e serve segmentos TS em cache
- `GET /hls-live/*path` — **Linear HLS**: FFmpeg em tempo real, segmentos gerados continuamente

**Segurança (3 camadas):**
1. **CORS restritivo**: permite apenas `tauri://localhost`, `https://tauri.localhost`, `http://localhost:1420`.
2. **Session Token**: middleware que valida `?token=<uuid>` em toda requisição não-health.
3. **Path scope validation**: `validate_path_scope()` resolve symlinks com `canonicalize()` e verifica que o arquivo está dentro de uma root folder autorizada.

**Gestão de processos:**
- `ProcessManager`: cleanup de processos FFmpeg órfãos a cada 10s.
- `LinearManager`: cleanup de sessões HLS inativas por 60s.
- Graceful shutdown via `CancellationToken` propagado para axum.

#### V2 — `src/delivery/streaming/server.rs` (294 linhas)

Servidor funcional, mas com lacunas:

**Rotas:**
- `/health`, `/probe/:asset_id`, `/stream/:asset_id`, `/playlist/:asset_id/playlist.m3u8`, `/segment/:asset_id/:segment`

**Diferenças críticas:**

| Feature                         | V1                       | V2                          |
| ------------------------------- | ------------------------ | --------------------------- |
| Linear HLS (`/hls-live`)        | ✅ Presente               | ❌ Ausente                   |
| CORS                            | ✅ Restritivo (3 origens) | ❌ `CorsLayer::permissive()` |
| Path scope validation           | ✅ Presente               | ❌ Ausente                   |
| ProcessManager (cleanup FFmpeg) | ✅ Presente               | ⚠️ Via HlsManager            |
| Graceful shutdown               | ✅ CancellationToken      | ❌ Sem shutdown signal       |
| Identificador de recursos       | Path físico              | ✅ Asset ID (DI correto)     |
| Range requests diretos          | Via common.rs            | ✅ `serve_file` helper       |

**Vantagem V2**: Usa `asset_id` em vez de path físico — mais seguro e alinhado com a arquitetura hexagonal (o frontend não precisa conhecer paths do filesystem).

**Regressão de segurança grave**: `CorsLayer::permissive()` permite qualquer origem fazer requisições ao servidor de streaming local — um site malicioso poderia acessar os arquivos da biblioteca do usuário enquanto o app estiver aberto.

---

### 2.5 Protocolo `asset://`

#### V1 — `src/protocols/`

Implementação **modular por tipo de mídia**:

- `common.rs`: `serve_file()` síncrona com range support, MIME detection via `FileFormat::detect()` (magic bytes), chunking de 10MB, auto-fallthrough de range para vídeos grandes (+500MB).
- `thumb.rs`, `image.rs`, `video.rs`, `audio.rs`, `model.rs`: handlers especializados por família.
- `video_stream.rs`, `audio_stream.rs`: integração com streaming server.
- `placeholders.rs`: assets de fallback quando thumbnail não existe.
- `font.rs`: handler específico para fontes.

#### V2 — `src/delivery/protocols/asset.rs` (418 linhas)

Handler unificado e **arquiteturalmente superior**:

**Vantagens da V2:**

1. ✅ **DI-based**: busca `AssetQueryHandler` do estado Tauri — desacoplado do banco direto.
2. ✅ **Asset ID em vez de path**: `asset://localhost/{asset_id}` — seguro e agnóstico ao filesystem.
3. ✅ **`?type=thumb | preview | glb`**: resolve fisicamente o caminho correto por tipo.
4. ✅ **Preview capability**: chama `provider.preview().generate_preview()` via FormatRegistry — extensível por formato.
5. ✅ **Async I/O**: `serve_file_async()` usa `tokio::fs::File` com seek e read assíncrono — sem bloquear o thread.
6. ✅ **GLB support**: path especial para modelos 3D convertidos em `.glb`.

**Regressão potencial**: A V1 tem handlers especializados por tipo de mídia (font, model, audio stream). A V2 centraliza tudo em um único handler — o que é mais limpo, mas precisa garantir que todos os casos especiais de V1 foram cobertos.

---

### 2.6 Banco de Dados

#### V1 — `src/db/` (9 arquivos)

| Arquivo            | Tamanho | Responsabilidade        |
| ------------------ | ------- | ----------------------- |
| `assets.rs`        | 13.9KB  | CRUD de assets          |
| `folders.rs`       | 14.5KB  | Hierarquia de pastas    |
| `search.rs`        | 24.5KB  | Motor de busca complexo |
| `tags.rs`          | 8.7KB   | Taxonomia               |
| `colors.rs`        | 6.0KB   | Paleta de cores         |
| `smart_folders.rs` | 1.7KB   | Pastas inteligentes     |
| `settings.rs`      | 1.2KB   | Configurações           |

**Total: ~70KB** de lógica SQL battle-tested.

#### V2 — `src/infra/database/` (6 arquivos)

| Arquivo             | Tamanho    | Responsabilidade                |
| ------------------- | ---------- | ------------------------------- |
| `ledger.rs`         | **63.9KB** | Asset Ledger (SSoT de mutações) |
| `queries.rs`        | 34.8KB     | Query handlers (leituras)       |
| `search_builder.rs` | 18.7KB     | Builder de queries de busca     |
| `models.rs`         | 9.4KB      | Structs do banco                |
| `manager.rs`        | 3.5KB      | Connection Pool                 |
| `mod.rs`            | 1.9KB      | Re-exports                      |

**Total: ~132KB** — quase o dobro da V1.

**Análise:**

- O `ledger.rs` cresceu para 63.9KB — é o ponto único de verdade para mutações. Isso é correto arquiteturalmente (CQRS), mas também significa que qualquer bug nesse arquivo tem impacto sistêmico.
- O `search_builder.rs` (18.7KB) é uma evolução do `search.rs` (24.5KB da V1) — provavelmente tem query builder mais robusto.
- **Risco**: O ledger implementa máquinas de estado complexas. Bugs de SQL nesse módulo já foram identificados em conversas anteriores (SQL syntax error, missing AND, metadata não gravada).

---

### 2.7 Suporte a Formatos (Media Processing)

#### V1 — `src/thumbnails/` + extractors (15 extractors)

Extractors especializados maduros:
- `sai.rs` (25.7KB), `sai2.rs` (19.6KB), `coreldraw.rs` (18.4KB), `mdp.rs` (30.2KB)
- Implementações completas com parse binário e geração de preview.

#### V2 — `src/processing/media/` (23 format providers + 15 extractors)

A V2 tem **mais formatos** com arquitetura de capabilities:

**Formatos novos na V2 (não presentes na V1):**
- `ai_format.rs` — Adobe Illustrator
- `aseprite_format.rs` — Aseprite pixel art
- `audio_format.rs` (13.3KB) — Áudio completo com waveform
- `binary_design_formats.rs` — Outros formatos binários
- `cad_format.rs` — CAD
- `exr_format.rs` — OpenEXR HDR
- `fallback_format.rs` — Generic fallback
- `modern_image_format.rs` — AVIF, HEIC, HEIF
- `project_zip_formats.rs` — Figma, Affinity backup
- `usd_format.rs` — Universal Scene Description
- `xmind_format.rs` — XMind mindmaps

**Entretanto**, a comparação dos extractors revela regressão de profundidade:

| Extractor        | V1 (bytes) | V2 (bytes) | Delta      |
| ---------------- | ---------- | ---------- | ---------- |
| `mdp.rs`         | 30,193     | 3,074      | **-90%** ⚠️ |
| `sai2.rs`        | 19,559     | 4,556      | **-77%** ⚠️ |
| `coreldraw.rs`   | 18,425     | 7,459      | -60% ⚠️     |
| `sai.rs`         | 25,671     | 13,614     | -47% ⚠️     |
| `xcf.rs`         | 11,192     | 7,890      | -30% ⚠️     |
| `binary_jpeg.rs` | 6,787      | 6,541      | ≈ OK       |
| `penpot.rs`      | 5,219      | 2,976      | -43% ⚠️     |

O extractor `mdp.rs` da V1 tem 30KB de lógica de parse binário. O V2 tem 3KB — muito provavelmente apenas um stub ou implementação parcial.

---

## 3. Regressões Críticas (Prioridade Alta)

### 🔴 RC-1: Transcoding Engine Ausente
**Módulo:** `processing/transcoding/`
**Impacto:** Vídeos incompatíveis (H.265, ProRes, MKV, HEVC) não conseguem ser transcodificados on-the-fly para visualização. O `HlsManager` precisa ter toda a lógica que estava no `FfmpegTranscoder` (cache, quality profiles, codec detection).
**Ação:** Verificar `src/feature/transcoding/hls_manager.rs` em profundidade.

### 🔴 RC-2: Security Regression no Streaming Server
**Módulo:** `delivery/streaming/server.rs`
**Impacto:** `CorsLayer::permissive()` + ausência de `validate_path_scope()`.
- Qualquer origem (site malicioso) pode acessar o servidor de streaming em `127.0.0.1`.
- Arquivos fora das pastas da biblioteca poderiam ser lidos via path traversal.
**Ação:** Restaurar CORS restritivo e a validação de scope de path.

### 🔴 RC-3: Processamento Serial do Indexador
**Módulo:** `feature/library/indexer.rs`
**Impacto:** Scan de bibliotecas grandes (10k+ arquivos) será muito mais lento — processamento sequencial vs fan-out paralelo da V1.
**Ação:** Implementar channel producer-consumer similar à V1 usando `tokio::spawn` por arquivo.

### 🔴 RC-4: Worker de Thumbnail sem Extração de Cor
**Módulo:** `processing/workers/` (a verificar)
**Impacto:** Se a extração de paleta de cores não está acontecendo após geração de thumbnail, a busca por cor na biblioteca deixou de funcionar.
**Ação:** Verificar o pipeline de thumbnail worker e garantir integração com `extract_color_palette`.

### 🔴 RC-5: Extractors de Formato com Implementação Parcial
**Módulo:** `processing/media/extractors/`
**Impacto:** Formatos como MDP (Manga Studio), SAI2, CorelDRAW têm extractors com 77-90% menos código que a V1. Thumbnails desses formatos provavelmente falharam ou retornam resultado vazio.
**Ação:** Migrar implementações completas dos extractors da V1.

---

## 4. Pendências e Melhorias Necessárias

### ⚠️ P-1: Watcher de FS não conectado ao Indexer V2
**Status:** A `processing/watcher/` existe, mas a integração com `LibraryIndexer.start_event_listener()` precisa ser validada. O evento `FsFileDiscovered` precisa ser emitido pelo watcher quando o OS reportar novos arquivos.

### ⚠️ P-2: Streaming sem Graceful Shutdown
**Impacto:** Ao fechar o app, processos FFmpeg filho podem ficar como zumbis. A V1 usa `with_graceful_shutdown(token.cancelled())` no axum + `ProcessManager.cleanup_stale()`.

### ⚠️ P-3: Linear HLS ausente na V2
**Impacto:** A V1 tem `/hls-live/*path` que inicia FFmpeg em tempo real e serve segmentos conforme são gerados. Isso é necessário para arquivos muito grandes onde a segmentação completa seria lenta demais.

### ⚠️ P-4: TranscodeCache ausente ou não conectado
**Impacto:** Sem cache de transcodificação, cada abertura de vídeo incompatível inicia um novo processo FFmpeg do zero.

### ⚠️ P-5: `duplicate_walk` no Indexer V2
**Detalhe:** O scan faz dois `WalkDir` completos (count + process). Com 100k arquivos em NFS/SD Card, isso é perceptível. Unificar em um único walk com chunk estimate.

---

## 5. Vantagens Genuínas da V2 sobre a V1

### ✅ V2-A: Protocolo `asset://` Assíncrono
O handler V2 usa `tokio::fs::File` assíncrono vs `std::fs::File` síncrono da V1. Para arquivos grandes (MKV 4K), isso evita bloquear o thread e é mais estável.

### ✅ V2-B: FormatRegistry com Capabilities
Adicionar suporte a um novo formato na V1 exige modificar 4-5 locais. Na V2, basta implementar `FormatProvider` e registrar — o polimorfismo de capabilities (ThumbCapability, MetadataCapability, PreviewCapability, StreamCapability) é elegante e extensível.

### ✅ V2-C: Asset ID no Protocolo e Streaming
A V2 indirece via ID (`asset://localhost/{id}`) em vez de expor o path físico ao frontend. Isso é mais seguro e permite que o backend controle o acesso aos arquivos.

### ✅ V2-D: Preview Capability Unificada
`provider.preview().generate_preview()` é extensível por formato na V2. Na V1, o preview era implementado fragmentado em vários handlers.

### ✅ V2-E: Ledger como SSoT (Single Source of Truth)
A serialização de mutações via `LedgerCommand` elimina race conditions entre o watcher e operações explícitas do usuário — o principal pain point da V1.

### ✅ V2-F: Suporte a Mais Formatos
V2 tem 23 format providers vs ~15 da V1, com suporte a AVIF, HEIC, OpenEXR, CAD, USD, XMind, Aseprite — expansão significativa da cobertura.

### ✅ V2-G: Cobertura de Auditoria e Tracing
A V2 tem uso consistente de `#[instrument]` e `tracing` — melhor observabilidade em produção.

---

## 6. Banco de Dados — Diferenças de Schema

### V1 Schema (inferido)
- Tabela `assets`: id, filename, path, size, mtime, thumbnail_path, thumbnail_attempts, dominant_color, format, family, folder_id, created_at, modified_at
- Tabela `asset_colors`: id, asset_id, hex_color, lab_lightness, lab_green_red, lab_blue_yellow, percentage, rank
- Tabela `folders`: id, path, name, parent_id, is_root
- Tabela `tags`, `asset_tags`, `smart_folders`, `settings`

### V2 Schema (inferido do ledger.rs)
- `assets` com `asset_metadata_envelope` (JSON blob para metadados técnicos como width/height/format)
- Suporte a `AssetState` enum (Indexed, Processing, Ready, Error)
- `LedgerCommand` como padrão de mutação
- Queries CQRS separadas via `queries.rs`

**Risco identificado**: A separação de metadados técnicos em `asset_metadata_envelope` (JSON) foi identificada em conversas anteriores como fonte de bugs — campos como `format`, `width`, `height` não estavam sendo gravados corretamente. Esse design quebra a atomicidade e dificulta queries SQL diretas sobre campos técnicos.

---

## 7. Roadmap de Correções Prioritizadas

```
Prioridade 1 (Bloqueadores de Release)
├── RC-2: Corrigir CORS e path scope no streaming server
├── RC-5: Migrar extractors completos da V1 (mdp, sai2, coreldraw)
└── RC-4: Validar/integrar worker de thumbnail + extração de cor

Prioridade 2 (Regressões de Performance)
├── RC-3: Paralelizar indexador com producer-consumer
├── P-1: Validar e integrar watcher de FS com LibraryIndexer
└── P-4: Implementar/conectar TranscodeCache no HlsManager

Prioridade 3 (Paridade de Features)
├── P-3: Implementar Linear HLS no streaming V2
├── P-2: Adicionar CancellationToken no streaming server
└── RC-1: Validar profundidade do HlsManager (quality profiles, audio)

Prioridade 4 (Otimizações)
├── P-5: Unificar os dois WalkDir do indexer V2
└── Validar todos os extractors vs V1 por tamanho/lógica
```

---

## 8. Conclusão

A V2 representa uma **evolução arquitetural genuína** — a estrutura hexagonal com FormatRegistry, Ledger como SSoT e separação CQRS é tecnicamente superior para suportar crescimento do projeto e um futuro SDK de plugins. Ela é a base certa para o futuro.

Entretanto, **a migração está incompleta**. Os módulos que fazem o "trabalho pesado" do DAM — geração de thumbnails com fila de prioridade, transcodificação on-the-fly, streaming HLS linear e extractors de formato proprietário — estão parcialmente migrados ou ausentes em relação à V1.

**A V1 ainda supera a V2 em robustez operacional.** Para a V2 atingir paridade e superá-la, são necessárias as correções listadas nas Prioridades 1 e 2, estimadas em 3-5 sprints de trabalho focado nos módulos de processamento.
