# Isolamento de Segurança do Streaming Server

**Data:** 2026-02-23
**Status:** Concluído
**Origem:** Roadmap Fase 0.4 + Análise de Excelência (§4.5)

---

## Objetivo

Fechar 3 superfícies de ataque do servidor de streaming embutido (Axum, 127.0.0.1:9876):

1. **CORS permissivo (Any)** → Restringir para origens legítimas do Tauri
2. **Ausência de token de sessão** → Token efêmero UUID v4 gerado no boot
3. **Rotas sem validação de path scope** → Sandboxing em diretórios autorizados (DB)

---

## Decisões Arquiteturais

- **Token via query param** (`?token=xxx`): compatível com HLS.js sem config extra
- **Diretórios via `Arc<Db>`**: sempre atualizado, sem cache stale
- **Token regenerado a cada restart**: circula apenas via IPC Tauri (nunca sai da máquina)

---

## Plano de Implementação

### Camada 1 — CORS Restritivo

**Arquivo:** `src-tauri/src/streaming/server.rs`

- [x] Substituir `CorsLayer::new().allow_origin(Any)` por allowlist explícito
- [x] Origens: `tauri://localhost`, `https://tauri.localhost`, `http://localhost:1420` (dev)
- [x] Manter `allow_methods(Any)` e `allow_headers(Any)` (necessário para HLS)

### Camada 2 — Token Efêmero de Sessão

**Backend:**
- [x] Gerar token UUID v4 no boot da app (em `lib.rs`, durante `setup`)
- [x] Armazenar em struct `StreamingToken(String)` via `app.manage()`
- [x] Injetar no `AppState` do streaming server
- [x] Criar middleware Axum que valida `?token=xxx` em todas as rotas (exceto `/health`)
- [x] Criar comando Tauri `get_streaming_token` que retorna o token ao frontend

**Frontend:**
- [x] No `hls-player.ts`, invocar `get_streaming_token` no boot
- [x] Anexar `&token=xxx` em todas as URLs geradas (`getHlsPlaylistUrl`, `getHlsProbeUrl`, etc.)
- [x] Passar token para `HlsPlayerManager` para segment URLs

### Camada 3 — Path Scope Validation

**Arquivo:** `src-tauri/src/streaming/server.rs`

- [x] Adicionar `Arc<Db>` ao `AppState`
- [x] Criar função `validate_path_scope(db, path)` que:
  1. Chama `db.get_all_root_folders()` para obter diretórios autorizados
  2. `canonicalize()` o path solicitado (via `spawn_blocking`)
  3. Verifica se o path canonicalizado `starts_with` algum root folder
  4. Retorna 403 se estiver fora do escopo
- [x] Invocar em `probe_handler`, `playlist_handler`, `segment_handler`, `linear_hls_handler`

---

## Verificação

- [x] Testar que playback HLS funciona normalmente
- [x] Testar que request sem token retorna 401
- [x] Testar que request com path fora do acervo retorna 403
- [x] Testar que request de origem externa é bloqueado por CORS
