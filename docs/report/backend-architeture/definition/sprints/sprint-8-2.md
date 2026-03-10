# Sprint 8.2: Streaming Server HTTP e Transcoding Commands

**Status:** Pendente
**Data e hora de inicio:** -  
**Data da conclusão:** -

**Fase 8:** Paridade IPC — Mídia, Manutenção e Utilidades
**Objetivo:** Restaurar todo o subsistema de streaming de mídia pesada: o servidor HTTP embarcado (warp/axum) com range requests (206 Partial Content), token de autenticação por sessão, detecção de necessidade de transcoding, geração de URL para stream, e gerenciamento de cache de transcoding.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. O backend inicia um servidor HTTP local em porta configurável com token de segurança.
2. O frontend pode solicitar `get_stream_url(asset_id)` e receber uma URL com token válido.
3. O `<video>` tag do frontend reproduz vídeos via HTTP Range Requests (206 Partial Content) sem carregar o arquivo inteiro em memória.
4. O frontend pode verificar se um arquivo necessita transcoding (`needs_transcoding`).
5. O frontend pode verificar se FFmpeg está disponível (`ffmpeg_available`).
6. O frontend pode pré-transcodificar um arquivo e gerenciar cache (is_cached, cleanup_cache, clear_cache, get_cache_stats).
7. `get_streaming_token` retorna o token de sessão para autenticação.
8. `cargo build` compila sem warnings.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Restaurar Streaming Server HTTP
- [ ] **O V2 já possui `HlsManager`** (sprint 5.2) em `feature/transcoding/hls_manager.rs`. Verificar o que já está implementado.
- [ ] O que FALTA é o **servidor HTTP genérico** que serve bytes diretos com Range Requests para formatos nativos (MP4/WebM que o Chromium suporta sem transcoding).
- [ ] Criar `delivery/streaming/server.rs` (ou verificar se já existe):
  - Bind em `127.0.0.1:<port>` com `warp` ou `axum` (usar o mesmo framework do HLS).
  - Rota `/stream/<asset_id>?token=<session_token>` → valida token → serve com 206 Partial Content.
  - Rota `/hls/<asset_id>/master.m3u8?token=<token>` → serve manifesto HLS via HlsManager.
  - Rota `/hls/<asset_id>/<segment>.ts?token=<token>` → serve segmentos TS.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/streaming/server.rs` — servidor warp completo com routes, token validation e lifecycle.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/streaming/linear.rs` — streaming de arquivo direto com Range headers.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/streaming/helpers.rs` — utilidades de parsing de range headers.

### 2. Implementar Token de Sessão
- [ ] Criar struct `StreamingSessionToken(String)` gerenciada via `app.manage()`.
- [ ] Gerar UUID v4 no boot e armazenar via `App::manage`.
- [ ] IPC `get_streaming_token` → retorna o token ao frontend.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/lib.rs` L25-38 — struct e command inline.

### 3. Implementar Transcoding Commands (Detecção e Cache)
- [ ] O V2 já possui `feature/transcoding/profiles.rs`. Verificar o que existe.
- [ ] Criar módulo `feature/transcoding/detector.rs` (ou adaptar do V1):
  - `needs_transcoding(path) -> bool` — checa se codec é incompatível com Chromium.
  - `is_native_format(path) -> bool` — checa se é MP4/WebM nativo.
  - `get_media_type(path) -> Audio | Video | Unknown`.
- [ ] Criar módulo `feature/transcoding/cache.rs`:
  - `TranscodeCache` struct que gerencia arquivos transcodificados em `<app_data>/transcoded/`.
  - Métodos: `exists()`, `get_cache_size()`, `get_file_count()`, `cleanup(max_age_days)`, `clear_all()`.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/transcoding/detector.rs`, `cache.rs`, `quality.rs`.

### 4. Criar get_stream_url com lógica de roteamento
- [ ] Lógica:
  1. Se `is_native_format(path)` → retorna URL `http://127.0.0.1:<port>/stream/<id>?token=<token>`.
  2. Se `needs_transcoding(path)` → verifica se HLS está pronto, senão inicia via HlsManager → retorna URL HLS.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/transcoding/commands.rs` → `get_stream_url()` L22-61.

### 5. Criar IPC Commands
- [ ] Em `delivery/tauri/commands/queries.rs` ou novo arquivo `delivery/tauri/commands/streaming.rs`:
  ```rust
  get_streaming_token() -> String
  needs_transcoding(path) -> bool
  is_native_format(path) -> bool
  get_stream_url(path, quality) -> String
  get_quality_options() -> Vec<QualityOption>
  ffmpeg_available() -> bool
  is_cached(path, quality) -> bool
  get_cache_stats() -> AppResult<CacheStats>
  ```
- [ ] Em mutações:
  ```rust
  transcode_file(path, quality) -> AppResult<String>
  cleanup_cache(max_age_days) -> AppResult<usize>
  clear_cache() -> AppResult<usize>
  ```

### 6. Iniciar Streaming Server no Boot (`lib.rs`)
- [ ] No setup do `lib.rs`, após DB init:
  1. Criar `StreamingSessionToken` e gerenciar.
  2. Spawnar o servidor HTTP com lifecycle token.
  3. Registrar no `LifecycleRegistry` para graceful shutdown.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/lib.rs` L132-145 — `spawn_server()`.

### 7. Registrar commands no `lib.rs`
- [ ] Adicionar os 11 novos commands ao `invoke_handler`.

### 8. Verificar Custom Protocols V2
- [ ] O V2 tem `delivery/protocols/asset.rs` como protocolo unificado. Verificar se `audio://`, `video://`, `audio-stream://`, `video-stream://` precisam ser registrados separadamente ou se `asset://` cobre tudo.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/protocols/` — 10 arquivos para protocolos distintos (audio, video, font, image, model, etc.).
- [ ] **Decisão:** No V2, o `asset://` unificado pode servir thumbnails e imagens. Para streaming de vídeo/áudio, o servidor HTTP é a rota correta (não custom protocols).

---

## 📁 Arquivos de Referência V1

| Funcionalidade       | Arquivo V1 (Mundam-main)                     | Notas                    |
| -------------------- | -------------------------------------------- | ------------------------ |
| Streaming server     | `src-tauri/src/streaming/server.rs`          | Warp server completo     |
| Linear streaming     | `src-tauri/src/streaming/linear.rs`          | File streaming com Range |
| Range helpers        | `src-tauri/src/streaming/helpers.rs`         | Parse Range headers      |
| HLS playlist         | `src-tauri/src/streaming/playlist.rs`        | m3u8 generation          |
| HLS segment          | `src-tauri/src/streaming/segment.rs`         | TS segment serving       |
| Probe                | `src-tauri/src/streaming/probe.rs`           | FFprobe integration      |
| Process manager      | `src-tauri/src/streaming/process_manager.rs` | FFmpeg process lifecycle |
| Transcoding commands | `src-tauri/src/transcoding/commands.rs`      | 11 IPC commands          |
| Transcoding detector | `src-tauri/src/transcoding/detector.rs`      | Codec compatibility      |
| Transcoding cache    | `src-tauri/src/transcoding/cache.rs`         | File-based cache         |
| Transcoding quality  | `src-tauri/src/transcoding/quality.rs`       | Quality profiles         |
| FFmpeg pipe          | `src-tauri/src/transcoding/ffmpeg_pipe.rs`   | Subprocess orchestration |
| Custom protocols     | `src-tauri/src/protocols/*.rs`               | 10 protocol handlers     |
| Token struct         | `src-tauri/src/lib.rs` L25-38                | StreamingSessionToken    |

## 📁 Arquivos a Criar/Modificar no V2

| Arquivo V2 (Mundam)                                         | Ação                                       |
| ----------------------------------------------------------- | ------------------------------------------ |
| `src-tauri/src/delivery/streaming/server.rs` (novo)         | Servidor HTTP com range requests           |
| `src-tauri/src/delivery/streaming/mod.rs` (novo)            | Módulo                                     |
| `src-tauri/src/feature/transcoding/detector.rs` (novo)      | Detecção de compatibilidade                |
| `src-tauri/src/feature/transcoding/cache.rs` (novo)         | Cache de transcoding                       |
| `src-tauri/src/delivery/tauri/commands/streaming.rs` (novo) | IPC commands streaming                     |
| `src-tauri/src/delivery/tauri/commands/mod.rs`              | Adicionar mod streaming                    |
| `src-tauri/src/lib.rs`                                      | Token, server spawn, registrar 11 commands |

---

## 💡 Notas para o Desenvolvedor / Agente
> Esta é a sprint **mais complexa** da Fase 8. O streaming server é fundamental para a reprodução de vídeo no frontend. Sem ele, qualquer vídeo maior que ~50MB causará OOM no WebView.

> **IMPORTANTE:** O V2 já tem `HlsManager` (feature/transcoding/). NÃO reimplemente HLS. Use o que já existe. O foco aqui é: (1) o servidor HTTP que serve Range Requests para vídeos nativos, (2) os IPC commands de transcoding, e (3) a integração com lifecycle para shutdown.

> **DECISÃO ARQUITETURAL:** No V2, NÃO usar custom protocols (`video://`, `audio-stream://`) para streaming. Usar HTTP puro via o servidor embarcado. Custom protocols são péssimos para Range Requests em Tauri. O `asset://` fica apenas para thumbnails/imagens estáticas.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- 
