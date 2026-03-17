# Sprint 10.12: Superioridade V2 — Settings Integration e Polimento Final

**Status da sprint:** Em progresso (settings IPC corrigido em `services.ts`)
**Data e hora de inicio da sprint:** 2026-03-17
**Data e hora da conclusão da sprint:** -

## Objetivo

Garantir que o painel de Settings está 100% integrado com a arquitetura V2, corrigir gaps de IPC entre frontend e backend, e implementar features que fazem a V2 superior à V1.

---

## Estado Real Identificado (Auditoria 2026-03-17)

### ✅ Settings — Já Integrado Corretamente

| Comando IPC | Backend V2 | Frontend |
|---|---|---|
| `run_db_maintenance` | ✅ `mutations.rs` | ✅ `systemStore.runDbMaintenance()` |
| `get_setting` | ✅ `settings.rs` | ✅ `settingsStore.initialize()` |
| `set_setting` | ✅ `settings.rs` | ✅ `settingsStore.updateSettings()` |
| `cleanup_cache` | ✅ `mutations.rs` | ✅ `systemStore.cleanupCache()` |
| `clear_cache` | ✅ `mutations.rs` | ✅ `systemStore.clearCache()` |
| `get_library_supported_formats` | ✅ `queries.rs` | ✅ `systemStore.initialize()` |

### ✅ Color Search — Já Funcionando

A busca por cor via `ColorCriterionField.tsx` e `AdvancedSearchModal.tsx` já estava implementada na V2, com `ColorPaletteSection.tsx` no inspector. **Não há gap aqui.**

### ✅ Folders Panel — Já Implementado

O gerenciamento de pastas monitoradas está implementado em `LibrarySidebarPanel.tsx` com `libraryActions.addLocation()` e `removeLocation()`.

### ⚠️ Correção Aplicada — get_cache_stats

**Problema:** `services.ts` chamava `get_cache_stats` mas o backend V2 registra `get_library_cache_stats` com schema diferente.

**Schema V2 real:**
```json
{
  "thumbnails": { "count": number, "size": number },
  "hls":        { "count": number, "size": number },
  "total":      { "count": number, "size": number }
}
```

**Correção aplicada** em `src/core/tauri/services.ts`:
- `getCacheStats()` → mapeia `total.count` e `total.size` para o contrato do frontend
- `cleanupCache()` → computa arquivos deletados via diff de stats antes/após
- `clearCache()` → computa arquivos deletados a partir do count anterior ao clear

---

## Tarefas Pendentes

### 1. Verificar `get_setting` / `set_setting` — Persistência de `thumbnail_threads`

**Status:** Verificar

O `settingsStore.initialize()` chama:
```typescript
tauriService.getSetting('thumbnail_threads')
tauriService.getSetting('cache_retention_days')
```

O backend `get_setting` usa `SettingsService.get_setting(key)`. Verificar se o `SettingsService` persiste e lê esses campos no `settings.json`:
```bash
grep -n "thumbnail_threads\|cache_retention_days" src-tauri/src/core/settings.rs
grep -n "thumbnail_threads\|cache_retention_days" src-tauri/src/feature/settings/mod.rs
```

Se as chaves não existem no `AppSettings` struct, o `get_setting` retornará `null` silenciosamente e o frontend usará os valores padrão — funcional mas não persistente.

### 2. Graceful Migration — Re-Index Diferencial

**Status:** Pendente

Ao reiniciar após atualização, assets sem `format` ou sem thumbnail devem ser reparados sem re-inserção:

```rust
// feature/library/indexer.rs
pub async fn repair_library(&self) -> AppResult<()> {
    let broken_assets = self.query_handler
        .get_assets_needing_repair()
        .await?;

    for asset in broken_assets {
        let format = self.format_registry
            .resolve(&asset.path, &[])
            .map(|provider| provider.name().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        self.ledger.execute(LedgerCommand::UpdateFormat {
            asset_id: asset.id,
            format,
        }).await.ok();
    }
    Ok(())
}
```

### 3. Health Check na Inicialização

**Status:** Pendente

Emitir evento de domínio se FFmpeg não estiver disponível:

```rust
// lib.rs — dentro do setup
if !ffmpeg_is_available() {
    let _ = event_bus.publish(DomainEvent::SystemHealthIssue {
        component: "ffmpeg".to_string(),
        message: "FFmpeg not found. Video transcoding unavailable.".to_string(),
    });
}
```

Frontend exibe um banner de aviso.

---

## Arquivos Modificados

- ✅ `src/core/tauri/services.ts` — `get_cache_stats` → `get_library_cache_stats` com mapeamento correto

## Arquivos a Verificar/Modificar

- `src-tauri/src/core/settings.rs` — verificar campos `thumbnail_threads` e `cache_retention_days`
- `src-tauri/src/lib.rs` — health check de FFmpeg na inicialização

---

## Critérios de Aceitação

- [x] `getCacheStats()` funciona sem erro 404 no console
- [x] `cleanupCache()` e `clearCache()` retornam contagem correta
- [x] Gerenciamento de pastas monitoradas funciona (`LibrarySidebarPanel.tsx`)
- [x] Otimização de banco de dados (`runDbMaintenance`) funciona via UI
- [ ] Settings de `thumbnail_threads` persistem entre sessões
- [ ] Health check de FFmpeg exibe banner no frontend se o binário não for encontrado
- [x] Sem erros de IPC em toda a página de Settings (após correção de get_cache_stats)
