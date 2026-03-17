# Sprint 10.9: Settings e Cache Stats — Completar Comandos IPC Pendentes

**Status da sprint:** Pendente
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Corrigir o item pendente da sprint 9.1 relacionado ao `get_cache_stats` e completar a integração de settings com o frontend.

## Estado Atual

A sprint 9.1 identificou os seguintes problemas em Settings (status: Pendente):

```
[Error] [IPC Error: get_cache_stats] – "Command get_cache_stats not found"
```

### Análise

O frontend chama `get_cache_stats` mas o comando V2 foi renomeado para `get_library_cache_stats` (implementado em `queries.rs`, linha 305).

Mapeamento V1 → V2:

| Comando V1 | Comando V2 | Status |
|---|---|---|
| `get_cache_stats` | `get_library_cache_stats` | ❌ Frontend usando nome antigo |
| `clear_cache` | ❓ | Verificar se existe |
| `get_settings` | ❓ | Verificar |
| `update_settings` | ❓ | Verificar |
| `get_transcoding_settings` | ❓ | Verificar |
| `update_transcoding_settings` | ❓ | Verificar |

## Tarefas

### 1. Corrigir Chamada `get_cache_stats` no Frontend

**Status:** Pendente

Localizar todas as chamadas ao `get_cache_stats` no frontend e atualizar para `get_library_cache_stats`.

**Busca:**
```bash
grep -r "get_cache_stats" src/ --include="*.ts" --include="*.tsx"
```

**Atualizar:**
- `src/lib/api.ts` ou equivalente
- Qualquer store que use esse comando

### 2. Auditar Comandos de Settings V1 vs V2

**Status:** Pendente

**V1 Settings commands** (`mundam-main/src-tauri/src/settings/commands.rs`):
- Listar todos os comandos da V1
- Mapear cada um para equivalente V2

**V1 Transcoding commands** (`mundam-main/src-tauri/src/transcoding/commands.rs`):
- `get_transcoding_settings`
- `update_transcoding_settings`

**Verificar em V2:**
- `src-tauri/src/delivery/tauri/commands/settings.rs`
- `src-tauri/src/delivery/tauri/commands/queries.rs`

### 3. Implementar Comandos de Settings Faltantes

**Status:** Pendente (após auditoria)

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

**Status:** Pendente

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

### 5. Adicionar Permissões no Tauri para Novos Comandos

**Status:** Dependente das tarefas anteriores

Após adicionar novos comandos, atualizar:
- `src-tauri/permissions/main.toml`
- `src-tauri/capabilities/default.json`

## Arquivos a Modificar

- `src/` (frontend) — corrigir chamada `get_cache_stats` → `get_library_cache_stats`
- `src-tauri/src/delivery/tauri/commands/settings.rs` — completar comandos faltantes
- `src-tauri/permissions/main.toml` — adicionar permissões
- `src-tauri/capabilities/default.json` — habilitar capacidades

## Critérios de Aceitação

- [ ] Página de Settings carrega sem erros no console
- [ ] `get_cache_stats` (agora `get_library_cache_stats`) retorna dados corretos de thumbnails e HLS
- [ ] Limpar cache de thumbnails funciona via UI
- [ ] Settings persistem após reiniciar o app
- [ ] Configurações de transcoding disponíveis via UI

## Referência V1

- `mundam-main/src-tauri/src/settings/commands.rs`
- `mundam-main/src-tauri/src/transcoding/commands.rs`
- `mundam-main/src` (frontend) — páginas de settings do V1
