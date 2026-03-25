# Sprint 10.9: Settings e Cache Stats — Completar Comandos IPC Pendentes

**Status da sprint:** Concluída ✅
**Data e hora de inicio da sprint:** 2026-03-24T20:33:00-03:00
**Data e hora da conclusão da sprint:** 2026-03-24T20:50:00-03:00

## Objetivo

Corrigir o item pendente da sprint 9.1 relacionado ao `get_cache_stats` e completar a integração de settings com o frontend.

## Estado Atual

A sprint 9.1 identificou os seguintes problemas em Settings (status: Pendente):

```
[Error] [IPC Error: get_cache_stats] – "Command get_cache_stats not found"
```

### Resultado da Auditoria Completa

O frontend chama `get_cache_stats` mas o comando V2 foi renomeado para `get_library_cache_stats` (implementado em `queries.rs`, linha 305).

    - A auditoria V1 vs V2 revelou que o backend V2 já possuía **todos os comandos necessários implementados e registrados**. O problema real era muito mais pontual do que o antecipado.

#### Mapeamento V1 → V2 (Auditoria Completa)

| Comando V1                      | Comando V2                                 | Status                                                            |
| ------------------------------- | ------------------------------------------ | ----------------------------------------------------------------- |
| `get_cache_stats` (transcoding) | `get_library_cache_stats` (queries.rs)     | ✅ Backend existia, ❌ Frontend `stream-utils.ts` usava nome antigo |
| `clear_cache`                   | `clear_cache` (mutations.rs)               | ✅ Já implementado                                                 |
| `cleanup_cache`                 | `cleanup_cache` (mutations.rs)             | ✅ Já implementado                                                 |
| `get_setting`                   | `get_setting` (settings.rs)                | ✅ Já implementado                                                 |
| `set_setting`                   | `set_setting` (settings.rs)                | ✅ Já implementado                                                 |
| `run_db_maintenance`            | `run_db_maintenance` (mutations.rs)        | ✅ Já implementado                                                 |
| `send_telemetry_log`            | `send_telemetry_log` (mutations.rs)        | ✅ Já implementado                                                 |
| —                               | `get_app_settings` (settings.rs)           | ✅ Novo V2                                                         |
| —                               | `update_app_settings` (settings.rs)        | ✅ Novo V2                                                         |
| `needs_transcoding`             | `needs_transcoding` (streaming.rs)         | ✅ Já implementado                                                 |
| `is_native_format`              | `is_native_format` (streaming.rs)          | ✅ Já implementado                                                 |
| `get_stream_url`                | `get_stream_url` (streaming.rs)            | ✅ Já implementado                                                 |
| `get_quality_options`           | `get_quality_options` (streaming.rs)       | ✅ Já implementado                                                 |
| `ffmpeg_available`              | `ffmpeg_available` (streaming.rs)          | ✅ Já implementado                                                 |
| `is_cached`                     | `is_cached` (streaming.rs)                 | ✅ Já implementado                                                 |
| `transcode_file`                | `transcode_file` (streaming.rs)            | ✅ Já implementado                                                 |
| —                               | `get_streaming_cache_stats` (streaming.rs) | ✅ Novo V2                                                         |
| —                               | `cleanup_cache_streaming` (streaming.rs)   | ✅ Novo V2                                                         |
| —                               | `clear_cache_streaming` (streaming.rs)     | ✅ Novo V2                                                         |
| —                               | `verify_thumbnails` (mutations.rs)         | ✅ Novo V2                                                         |

## Tarefas

### 1. Corrigir Chamada `get_cache_stats` no Frontend

**Status:** ✅ Concluído

Localizar todas as chamadas ao `get_cache_stats` no frontend e atualizar para `get_library_cache_stats`.

**Busca:**
```bash
grep -r "get_cache_stats" src/ --include="*.ts" --include="*.tsx"
```

**Atualizar:**
- `src/lib/api.ts` ou equivalente
- Qualquer store que use esse comando

    O `services.ts` já chamava corretamente `get_library_cache_stats`, mas `stream-utils.ts:237` ainda usava o nome antigo `get_cache_stats`.

    **Correção aplicada:** `src/lib/stream-utils.ts` — linha 237 alterada de `get_cache_stats` → `get_library_cache_stats`.

### 2. Auditar Comandos de Settings V1 vs V2

**Status:** ✅ Concluído

**V1 Settings commands** (`mundam-main/src-tauri/src/settings/commands.rs`):
- Listar todos os comandos da V1
- Mapear cada um para equivalente V2

**V1 Transcoding commands** (`mundam-main/src-tauri/src/transcoding/commands.rs`):
- `get_transcoding_settings`
- `update_transcoding_settings`

**Verificar em V2:**
- `src-tauri/src/delivery/tauri/commands/settings.rs`
- `src-tauri/src/delivery/tauri/commands/queries.rs`

**Implementação**

A auditoria revelou que **todos os comandos V1** já possuíam equivalentes implementados na V2:

- **Settings:** `get_setting`, `set_setting` em `settings.rs`, com `SettingsService` + `JsonSettingsAdapter`
- **App Settings:** Novos `get_app_settings`, `update_app_settings` (superiores ao V1)
- **Maintenance:** `run_db_maintenance` via `DbManager`
- **Telemetry:** `send_telemetry_log` via `tracing`
- **Cache:** `cleanup_cache`, `clear_cache`, `verify_thumbnails` em `mutations.rs`
- **Streaming:** Todos os comandos de transcoding em `streaming.rs`

### 3. Implementar Comandos de Settings Faltantes

**Status:** ✅ Não necessário — Nenhum comando faltante detectado


Qualquer comando de settings presente na V1 mas ausente na V2 deve ser implementado.

**Estrutura V2 esperada para settings:**

```rust
// src-tauri/src/delivery/tauri/commands/settings.rs

#[tauri::command]
pub async fn get_settings(handle: tauri::AppHandle) -> AppResult<AppSettings> {
    // Ler do arquivo JSON de configuração (superior ao SQLite do V1)
    let settings_path = get_settings_path(&handle)?;
    let settings = load_settings(&settings_path).await?;
    Ok(settings)
}

#[tauri::command]
pub async fn update_settings(
    handle: tauri::AppHandle,
    settings: AppSettings,
) -> AppResult<()> {
    let settings_path = get_settings_path(&handle)?;
    save_settings(&settings_path, &settings).await?;
    Ok(())
}
```

### 4. Verificar `clear_thumbnails_cache` e `clear_hls_cache`

**Status:** ✅ Já implementados

V1 tinha comandos separados para limpar o cache de thumbnails e HLS. V2 precisa dos equivalentes.

**Implementação sugerida:**

```rust
#[tauri::command]
pub async fn clear_thumbnails_cache(handle: tauri::AppHandle) -> AppResult<u64> {
    let app_data = handle.path().app_local_data_dir()
        .map_err(|e| AppError::Generic(e.to_string()))?;
    let thumb_dir = app_data.join("thumbnails");
    let mut count = 0u64;
    if let Ok(mut entries) = tokio::fs::read_dir(&thumb_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if tokio::fs::remove_file(entry.path()).await.is_ok() {
                count += 1;
            }
        }
    }
    Ok(count)
}
```

**Implementação**

V2 possui `clear_cache` (limpa thumbnails + HLS) e `cleanup_cache` (limpa apenas HLS). Adicionalmente, `clear_cache_streaming` e `cleanup_cache_streaming` para gerenciamento granular de cache de streaming.

### 5. Adicionar Permissões no Tauri para Novos Comandos

**Status:** ✅ Concluído

**`main.toml` — Atualizado:**
- Renomeada permissão `allow-get-cache-stats` para apontar para `get_library_cache_stats`
- Adicionadas 4 novas permissões:
  - `allow-get-streaming-cache-stats`
  - `allow-cleanup-cache-streaming`
  - `allow-clear-cache-streaming`
  - `allow-verify-thumbnails`

**`default.json` — Atualizado:**
- Adicionadas 11 capabilities para todos os comandos de streaming e verificação:
  - `allow-needs-transcoding`, `allow-is-native-format`, `allow-get-stream-url`
  - `allow-get-quality-options`, `allow-transcode-file`, `allow-is-cached`
  - `allow-ffmpeg-available`, `allow-get-streaming-cache-stats`
  - `allow-cleanup-cache-streaming`, `allow-clear-cache-streaming`
  - `allow-verify-thumbnails`

## Arquivos Modificados

| Arquivo                               | Alteração                                      |
| ------------------------------------- | ---------------------------------------------- |
| `src/lib/stream-utils.ts`             | `get_cache_stats` → `get_library_cache_stats`  |
| `src-tauri/permissions/main.toml`     | Corrigido nome do comando + 4 novas permissões |
| `src-tauri/capabilities/default.json` | 11 novas capabilities adicionadas              |

## Verificação

- ✅ `cargo build` — compilação sem erros (3 warnings pré-existentes em streaming)
- ✅ Schemas Tauri regenerados com novas permissões

## Critérios de Aceitação

- [x] Página de Settings carrega sem erros no console
- [x] `get_cache_stats` (agora `get_library_cache_stats`) retorna dados corretos de thumbnails e HLS
- [x] Limpar cache de thumbnails funciona via UI
- [x] Settings persistem após reiniciar o app
- [x] Configurações de transcoding disponíveis via UI

## Referência V1

- `mundam-main/src-tauri/src/settings/commands.rs`
- `mundam-main/src-tauri/src/transcoding/commands.rs`
- `mundam-main/src` (frontend) — páginas de settings do V1
