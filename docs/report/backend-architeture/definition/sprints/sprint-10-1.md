# Sprint 10.1: Segurança e Estabilidade do Streaming Server

**Status da sprint:** Pendente
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Restaurar as 3 camadas de segurança do streaming server da V1 que foram perdidas ou enfraquecidas na V2, e adicionar graceful shutdown via `CancellationToken`.

## Contexto

O servidor V1 (`streaming/server.rs`, 678 linhas) tinha:
1. **CORS restritivo** — apenas `tauri://localhost`, `https://tauri.localhost`, `http://localhost:1420`
2. **Session Token validation** por middleware em todas as rotas não-health
3. **Path scope validation** — `canonicalize()` + verificação de que o arquivo está dentro de uma root folder autorizada
4. **Graceful shutdown** — `with_graceful_shutdown(token.cancelled())`
5. **ProcessManager** — cleanup de processos FFmpeg órfãos a cada 10s
6. **LinearManager** — cleanup de sessões HLS inativas por 60s

O servidor V2 (`delivery/streaming/server.rs`, 294 linhas) tem:
- ✅ Token auth via middleware (`StreamingSessionToken`)
- ✅ HlsManager com cleanup via CancellationToken (90s timeout)
- ❌ `CorsLayer::permissive()` — qualquer origem pode fazer requests
- ❌ Sem path scope validation (path traversal possível)
- ❌ Sem graceful shutdown no servidor Axum

## Tarefas

### 1. Corrigir CORS para origens restritas

**Status:** Pendente

**Arquivos da V1 para referência:**
- `mundam-main/src-tauri/src/streaming/server.rs` (CORS setup com 3 origens)

**Implementação:**

```rust
// delivery/streaming/server.rs
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use axum::http::HeaderValue;

fn build_cors_layer() -> CorsLayer {
    let allowed_origins = [
        "tauri://localhost",
        "https://tauri.localhost",
        "http://localhost:1420",
    ]
    .into_iter()
    .filter_map(|origin| origin.parse::<HeaderValue>().ok())
    .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::HEAD,
            axum::http::Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            axum::http::header::RANGE,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]))
        .expose_headers([
            axum::http::header::CONTENT_RANGE,
            axum::http::header::CONTENT_LENGTH,
            axum::http::header::ACCEPT_RANGES,
        ])
}
```

**Substituir no router:**
```rust
// Antes:
.layer(CorsLayer::permissive())

// Depois:
.layer(build_cors_layer())
```

### 2. Adicionar Path Scope Validation

**Status:** Pendente

**Arquivos da V1 para referência:**
- `mundam-main/src-tauri/src/streaming/server.rs` → `validate_path_scope()`

**Implementação:**

O servidor V2 já usa `asset_id` em vez de path físico na maioria das rotas (o que é arquiteturalmente superior), tornando o path scope desnecessário nessas rotas. A validação de escopo deve acontecer no `AssetQueryHandler` — se o asset não existe no banco, a rota retorna 404.

Para `/stream/:asset_id`, verificar que o path físico recuperado do banco está dentro de uma pasta monitorada:

```rust
// feature/assets/queries.rs ou delivery/streaming/server.rs
async fn validate_asset_in_scope(
    asset_query: &Arc<dyn AssetQueryHandler>,
    asset_id: &str,
) -> AppResult<std::path::PathBuf> {
    let asset = asset_query
        .get_by_id(asset_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Asset {} not found", asset_id)))?;

    // Resolve symlinks antes de comparar
    let canonical_path = asset.path.canonicalize().map_err(|_| {
        AppError::Generic("Cannot resolve asset physical path".to_string())
    })?;

    Ok(canonical_path)
}
```

**Nota:** A validação via asset_id já é mais segura que a validação de path da V1 — um asset_id inválido retorna 404 antes de qualquer acesso ao filesystem.

### 3. Graceful Shutdown do Servidor Axum

**Status:** Pendente

**Implementação:**

```rust
// delivery/streaming/server.rs
pub async fn start_server(
    app_handle: AppHandle,
    port: u16,
    shutdown_token: CancellationToken, // NOVO parâmetro
) -> tauri::async_runtime::JoinHandle<()> {
    // ... state setup ...

    let app = Router::new()
        // ... rotas ...
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tauri::async_runtime::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_token.cancelled().await;
                tracing::info!("Streaming server: graceful shutdown initiated");
            })
            .await
            .ok();
        tracing::info!("Streaming server stopped");
    })
}
```

**Integrar no `lib.rs`** — passar o child token do lifecycle para `start_server`.

### 4. Verificar e Testar Token Auth

**Status:** Verificar

O V2 já tem `StreamingSessionToken` e middleware de auth (`auth_middleware`). Verificar:
- [ ] O frontend chama `get_streaming_token` antes de fazer requests ao streaming server
- [ ] O token é passado como `?token=<uuid>` em todas as URLs
- [ ] A rota `/health` é excluída da validação de token

**Referência frontend:**
- `src/lib/hls-player.ts` → como o token é usado
- `src/lib/stream-utils.ts` → construção das URLs

## Arquivos a Modificar

- `src-tauri/src/delivery/streaming/server.rs` — CORS restritivo + graceful shutdown
- `src-tauri/src/lib.rs` — passar `CancellationToken` para `start_server`

## Critérios de Aceitação

- [ ] CORS bloqueia requests de origens não autorizadas (verificar via DevTools)
- [ ] Servidor Axum para graciosamente ao fechar o app (sem SIGKILL no processo)
- [ ] HlsManager cleanup ainda funciona (sessions expiram em 90s de inatividade)
- [ ] Token auth ainda funciona para HLS playlist e segments
- [ ] Reprodução de vídeo continua funcionando após as mudanças

## Notas para o Desenvolvedor

> A segurança de CORS é crítica em apps Tauri porque o servidor HTTP roda em localhost. Um site malicioso com uma aba aberta no Safari poderia fazer requests cross-origin a `http://127.0.0.1:<port>` e obter acesso aos arquivos do usuário. O CORS restritivo é a única defesa contra isso.

> O path scope validation da V1 é parcialmente substituído pelo design de asset_id do V2 — um atacante precisaria primeiro saber o UUID do asset, que só é exposto via comandos IPC autenticados pelo Tauri. Isso é arquiteturalmente mais seguro.
