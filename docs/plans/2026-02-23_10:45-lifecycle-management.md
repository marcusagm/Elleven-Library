# Lifecycle Management — CancellationToken + LifecycleRegistry

**Data:** 2026-02-23  
**Status:** ✅ Implementado e Verificado  
**Roadmap Ref:** Phase 0 — Critical Fixes — "Lifecycle management: listeners e tarefas assíncronas"

---

## Contexto

O Mundam utiliza diversas tarefas de longa duração (watchers, thumbnail worker, streaming server, cleanup loops) e listeners no frontend (Tauri `listen()`). Nenhuma delas possuía uma política formal de **teardown/shutdown**, criando risco de:

- Acúmulo de tasks órfãs em cenários de restart/hot-reload
- Listeners duplicados no frontend
- Handles de I/O permanecendo abertos desnecessariamente
- Impossibilidade de shutdown graceful ao fechar a aplicação

## Solução Adotada

**Opção C — Híbrida**: `CancellationToken` hierárquico (do `tokio_util::sync`) para a mecânica de cancelamento + `LifecycleRegistry` centralizada para tracking de `JoinHandle`s + `unlisten` via `onCleanup` no frontend.

### Decisões de Design

| Aspecto | Decisão |
|---|---|
| **Timeouts** | Sem timeout agressivo para tarefas contínuas — aguarda conclusão natural do batch/request atual |
| **Axum Streaming Server** | `with_graceful_shutdown(token.cancelled())` — para de aceitar novos requests, finaliza os pendentes |
| **Thumbnail Worker** | Em shutdown normal, aguarda batch atual. Em interrupt, thumbnails ficam com `thumbnail_path IS NULL` no DB e voltam à fila automaticamente |
| **Watcher restart** | Token anterior é cancelado automaticamente via `WatcherRegistry.insert()` antes de registrar o novo |

---

## Passo a Passo da Implementação

### Task 1: Feature `sync` no `tokio-util`

**Objetivo:** Habilitar `CancellationToken` na dependência `tokio-util`.

**Resultado:** A feature `sync` **não existe** na versão `0.7.18` (locked). O `CancellationToken` está disponível sem feature flag adicional. A alteração no `Cargo.toml` foi **revertida** após o `cargo check` falhar com:

```
package `mundam` depends on `tokio-util` with feature `sync`
but `tokio-util` does not have that feature.
```

**Lição:** Sempre verificar a versão lockada antes de assumir feature flags da documentação genérica.

### Task 2: Criar `src-tauri/src/lifecycle.rs`

Novo módulo central com:
- `LifecycleRegistry` struct com `root_token: CancellationToken` + `tasks: Mutex<HashMap<String, (CancellationToken, JoinHandle<()>)>>`
- `root_token()` — clone do token raiz
- `child_token()` — cria token filho (cancelado automaticamente quando o root é cancelado)
- `register(name, token, handle)` — registra task; se já existir uma com o mesmo nome, cancela a anterior
- `shutdown_by_name(name)` — cancela token + `await` no handle
- `shutdown_all()` — cancela root + `await` em todos os handles
- `Default` trait implementado

### Task 3: Integrar no `lib.rs`

- Criação do `LifecycleRegistry` no `setup()` como `Arc<LifecycleRegistry>`
- Registrado via `app.manage(lifecycle.clone())`
- Passado para todos os subsistemas:
  - `ThumbnailWorker` recebe um `child_token` + handle registrado como `"thumbnail_worker"`
  - `StreamingServer` recebe um `child_token` + handle registrado como `"streaming_server"`
  - `Indexer` recebe o `Arc<LifecycleRegistry>` para registrar watchers sob demanda

### Task 4: Refatorar `ThumbnailWorker::start()`

**Antes:** `pub async fn start(self)` — loop infinito sem condição de parada  
**Depois:** `pub fn start(self, token: CancellationToken) -> JoinHandle<()>`

Mudanças:
- Checagem `token.is_cancelled()` no início de cada iteração do loop
- `tokio::select!` nos pontos de sleep (idle wait, error backoff) para responder ao cancelamento imediatamente
- Retorna o `JoinHandle` do `tauri::async_runtime::spawn` para tracking

### Task 5: Refatorar `StreamingServer::start()`

**Antes:** `axum::serve(listener, app).await` — sem mecanismo de parada  
**Depois:** `axum::serve(listener, app).with_graceful_shutdown(token.cancelled()).await`

Mudanças adicionais:
- 2 cleanup tasks internas (process cleanup, linear session cleanup) recebem **child tokens** do token do servidor
- Cada cleanup loop usa `tokio::select!` com `child_token.cancelled()` para parar cooperativamente
- `spawn_server()` aceita `CancellationToken` e retorna `JoinHandle<()>`

### Task 6: Refatorar `start_watcher()`

**Antes:** `oneshot::Sender<()>` / `oneshot::Receiver<()>` para sinalização  
**Depois:** `CancellationToken` no `WatcherRegistry` + `select!` no loop principal

Mudanças:
- `WatcherRegistry.watchers: HashMap<String, CancellationToken>` (era `oneshot::Sender<()>`)
- `stop_watcher()` usa `token.cancel()` (era `tx.send(())`)
- `start_watcher()` retorna `JoinHandle<()>` para tracking
- Mudou de `tokio::spawn` para `tauri::async_runtime::spawn` (compatibilidade de tipo)
- Removido ponto-e-vírgula após `spawn(...)` para retornar o handle

### Task 7: Fix frontend listeners

**`App.tsx`:**
- `listen('indexer:complete', ...)` agora armazena o `UnlistenFn` retornado
- `onCleanup(() => unlistenIndexerComplete())` adicionado para cleanup

**`appearanceStore.ts`:**
- `listen(SYNC_EVENT, ...)` armazena `unlistenSyncEvent` em variável de módulo
- Re-inicialização limpa o listener anterior antes de registrar novo
- Previne acúmulo de listeners em cenários de hot-reload

---

## Obstáculos Encontrados

### 1. Feature `sync` inexistente no `tokio-util 0.7.18`

A documentação genérica do `tokio-util` menciona features como `sync`, mas a versão `0.7.18` (lockada no projeto) disponibiliza `CancellationToken` sem feature flag. O `cargo check` falhou imediatamente, e a solução foi reverter a mudança no `Cargo.toml`.

### 2. `tauri::async_runtime::JoinHandle` vs `tokio::task::JoinHandle`

Estes são **tipos distintos** apesar de nomes similares. O `tauri::async_runtime::spawn` retorna `tauri::async_runtime::JoinHandle`, enquanto `tokio::spawn` retorna `tokio::task::JoinHandle`. A `LifecycleRegistry` precisou usar `tauri::async_runtime::JoinHandle` para ser compatível com todos os subsistemas que usam o runtime Tauri.

**Solução:** Padronizar todos os spawns para `tauri::async_runtime::spawn` e todos os imports de `JoinHandle` para `tauri::async_runtime::JoinHandle`.

### 3. Semicolon acidental no `watcher.rs`

O `tokio::spawn(async move { ... });` (com `;`) faz a função retornar `()` ao invés do `JoinHandle`. O compilador apontou o erro com a sugestão `help: remove this semicolon to return this value`.

### 4. Propagação de parâmetros pela cadeia `Indexer → scan → watcher`

A adição do `LifecycleRegistry` exigiu atualizar a assinatura de:
- `Indexer::new()` (4 call-sites: `lib.rs`, `indexing.rs`, `folders.rs` x2)
- `scan::run_scan()` (1 call-site)
- `start_watcher()` (1 call-site)

Cada call-site precisou obter o `LifecycleRegistry` via `app.try_state::<Arc<LifecycleRegistry>>()`.

---

## Arquivos Modificados

| Arquivo | Tipo | Mudança |
|---|---|---|
| `src-tauri/src/lifecycle.rs` | **Novo** | `LifecycleRegistry` — hub central de lifecycle |
| `src-tauri/src/lib.rs` | Backend | Integração do registry, tokens, e handles |
| `src-tauri/src/indexer/types.rs` | Backend | `WatcherRegistry` usa `CancellationToken` |
| `src-tauri/src/indexer/mod.rs` | Backend | `Indexer` recebe `LifecycleRegistry` |
| `src-tauri/src/indexer/watcher.rs` | Backend | `CancellationToken` + retorno de `JoinHandle` |
| `src-tauri/src/indexer/scan.rs` | Backend | Passa `LifecycleRegistry`, registra watcher handle |
| `src-tauri/src/thumbnails/worker.rs` | Backend | `select!` para shutdown cooperativo |
| `src-tauri/src/streaming/server.rs` | Backend | `graceful_shutdown` no axum, child tokens nos cleanups |
| `src-tauri/src/library/commands/indexing.rs` | Backend | Passa `LifecycleRegistry` ao `Indexer` |
| `src-tauri/src/library/commands/folders.rs` | Backend | Passa `LifecycleRegistry` ao `Indexer` (2 locais) |
| `src/App.tsx` | Frontend | `unlisten` via `onCleanup` |
| `src/core/store/appearanceStore.ts` | Frontend | `unlisten` idempotente com re-registro |

---

## Verificação

### Compilação
```
cargo check  →  Finished `dev` profile in 5.19s  ✅
cargo clippy →  0 novos warnings  ✅
```

### Runtime (Terminal)
```
LIFECYCLE: Registered task 'streaming_server'          ✅
LIFECYCLE: Registered task 'thumbnail_worker'          ✅
LIFECYCLE: Registered task 'watcher:/path/to/RAW'      ✅
LIFECYCLE: Replaced existing task 'watcher:/path/...'  ✅ (re-scan corretamente substitui watcher)
Watcher task received STOP for /path/...               ✅ (watcher anterior parou)
```

---

## Melhorias Futuras

1. **Hook de shutdown no `app.on_event(RunEvent::ExitRequested)`**: Atualmente o `LifecycleRegistry` possui `shutdown_all()` mas ninguém o chama no fechamento da app. Integrar com o evento de saída do Tauri para garantir shutdown ordenado.

2. **Logging estruturado**: Substituir os `println!("LIFECYCLE: ...")` por um sistema de logging formal (ex.: `tracing` crate) com níveis `info`/`debug`/`warn`.

3. **Métricas de task lifetime**: Registrar timestamps de início/fim de cada task para diagnóstico de performance e potenciais memory leaks.

4. **Timeout com abort forçado**: Para cenários onde uma task não responde ao cancelamento dentro de um período razoável, implementar `handle.abort()` como fallback após N segundos.

5. **Frontend lifecycle unificado**: Criar um `LifecycleManager` no frontend (TypeScript) que centralize todas as `UnlistenFn`s em um único ponto de cleanup, ao invés de gerenciar cada uma individualmente.

6. **Testes unitários**: Adicionar testes para `LifecycleRegistry`:
   - Registrar e cancelar uma task
   - Registrar task com nome duplicado (deve cancelar a anterior)
   - `shutdown_all` cancela e aguarda todas as tasks
   - Token filho é cancelado quando root é cancelado
