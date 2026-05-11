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
- [x] Settings de `thumbnail_threads` persistem entre sessões
- [x] Health check de FFmpeg exibe banner no frontend se o binário não for encontrado
- [x] Sem erros de IPC em toda a página de Settings (após correção de get_cache_stats)

---

## Implementações e Correções (Sessão Adicional)

Abaixo estão detalhadas todas as correções e novas lógicas inseridas no sistema durante a Sprint 10.12 para estabilizar o backend V2:

### 1. Correção de Conversão Numérica de Configurações
- **Arquivo**: `src-tauri/src/feature/settings/service.rs`
- **Problema**: A tela de *Settings* (`GeneralPanel.tsx`) enviava propriedades numéricas como strings via IPC (ex: `String(payload.thumbnailThreads)`). O Rust tentava converter diretamente com `value.as_u64()`, resultando em falha silenciosa e os valores não sendo persistidos ou retornados.
- **Solução**: O método `set_setting` foi ajustado para tentar primeiro `value.as_u64()` e, como fallback, tentar o parsing se o valor for recebido como string (`value.as_str().and_then(|s| s.parse().ok())`), salvando de forma correta e robusta a configuração `thumbnail_threads` e `cache_retention_days`.

### 2. Correção de State Mismatch no DB Maintenance
- **Arquivo**: `src-tauri/src/delivery/tauri/commands/mutations.rs`
- **Problema**: Ao chamar o método `run_db_maintenance`, a aplicação falhava com *Internal Server Error*. A injeção do gerenciador do banco estava descompassada: o Tauri possuía no state um `Arc<DbManager>`, mas a assinatura da mutation esperava diretamente um `DbManager`.
- **Solução**: Assinatura da mutation foi alterada de `State<'_, DbManager>` para `State<'_, Arc<DbManager>>`, resolvendo o erro 500 no botão de *Optimize Library*.

### 3. Loop Infinito na Geração de Thumbnails
- **Arquivo**: `src-tauri/src/processing/workers/thumbnail_worker.rs`
- **Problema**: O arquivo de log estava sendo inundado por erros constantes de extração. O worker de thumbnails rodava infinitamente sobre os mesmos arquivos caso o provider não fosse resolvido, a imagem retornasse header inválido ou um erro ocorresse no decoder de imagem. Isso acontecia pois o campo `thumbnail_path` da base continuava como `NULL`, qualificando-o novamente para processamento na próxima rodada do worker.
- **Solução**: Foi implementada uma marcação de fallback (inserção do path como string vazia `""`) no banco de dados na transação de `UpdateThumbnail` durante o tratamento de erro (`Err`), header nulo e erro na resolução de formato (`format_registry.resolve`), removendo a flag `NULL` que causava o comportamento cíclico e consequentemente reduzindo a carga do sistema a zero para arquivos problemáticos.

### 4. Implementação de Reparo Diferencial (`repair_library`)
- **Arquivos**: `core/repository/asset.rs`, `infra/database/queries.rs`, `feature/library/indexer.rs`, `core/ledger/command.rs`, `infra/database/ledger.rs`
- **Solução**: Um novo fluxo seguro foi desenhado e implementado via `LibraryIndexer::repair_library()` onde assets identificados via nova query `get_assets_needing_repair` (que não possuem formato ou thumbails mapeados) passam por uma redetecção usando o payload nativo Hexagonal de transação do banco (`LedgerCommand::UpdateFormat`), evitando um rescaneamento recursivo nocivo (`SQLITE_BUSY`).

### 5. Verificação de Saúde do FFmpeg no Setup
- **Arquivos**: `core/events/payloads.rs`, `src-tauri/src/lib.rs`
- **Solução**: O inicializador do Tauri foi injetado com um check automático de `crate::processing::transcoding::check_transcoding_availability()`. Quando indisponível, agora despacha nativamente a notificação assíncrona `DomainEvent::SystemHealthIssue` para o barramento do EventBus, deixando visível ao Client a limitação da reprodução de vídeos.

### 6. Limpeza de Débitos Técnicos (Warnings do Compilador)
- **Arquivos**: `coreldraw.rs`, `debouncer.rs`, `server.rs`, `probe.rs`, `xcf.rs`
- **Solução**: Realizada a limpeza de diversos *warnings* apontados pelo cargo:
  - `coreldraw.rs`: Atualizado o uso de `image::io::Reader` (obsoleto) para `image::ImageReader`.
  - `debouncer.rs`: Removida variável `parent_match` que não era mais utilizada após as otimizações das heurísticas de renomeação.
  - `server.rs`: Removidos os imports não utilizados `serde::Deserialize` e `HlsManager`, além de limpar a struct `StreamQuery` e referências órfãs de estado.
  - `probe.rs`: Removida a função não utilizada `is_hls_problematic`.
  - `xcf.rs`: Removida a função não utilizada `skip_properties`.
- **Resultado**: O build da aplicação V2 agora é feito de forma limpa, sem mensagens de atenção, assegurando que o código esteja polido e fácil de auditar de acordo com as diretrizes `clean-code`.

### 7. Throttling de Eventos (Front-end)
- **Arquivos**: `src/core/store/metadata/locationActions.ts`
- **Problema**: O backend emite eventos `library:batch-change` com alta frequência durante o processo de indexação ou quando o watcher detecta múltiplas mudanças rápidas. Isso causava uma sobrecarga no front-end, que tentava recarregar estatísticas e metadados instantaneamente, resultando em "jitter" visual e lentidão na interface.
- **Solução**: Implementado um mecanismo de *debounce* de 500ms no método `handleBatchChange`. Agora, as notificações de mudança de lote são acumuladas e as requisições de atualização global (`refreshAll`) são disparadas apenas uma vez após o período de estabilização, reduzindo drasticamente o tráfego de rede e o processamento no cliente.

### 8. Restauração da Integridade da Hierarquia de Pastas
- **Arquivos**: `core/repository/asset.rs`, `infra/database/queries.rs`, `infra/database/ledger.rs`, `feature/library/indexer.rs`
- **Problema**: Na arquitetura V2, ao adicionar uma pasta pai (`/A`) após uma pasta filha já monitorada (`/A/B`), a hierarquia não era reconstruída automaticamente, mantendo ambas como raízes independentes. Além disso, arquivos descobertos pelo watcher em subpastas profundas podiam ficar órfãos se a estrutura intermediária de pastas não existisse no banco.
- **Solução**:
  - **Adoção Automática**: O comando `LedgerCommand::CreateFolder` foi atualizado para um padrão `UPSERT`. Sempre que uma pasta é criada ou atualizada, o sistema executa a lógica de `adopt_orphaned_children`, vinculando qualquer pasta raiz existente que seja fisicamente uma subpasta da nova pasta.
  - **Garantia de Hierarquia**: Implementado o método recursivo `ensure_folder_hierarchy` no `LibraryIndexer`. Durante a descoberta de arquivos via watcher, o sistema agora garante que toda a cadeia de diretórios pais exista no banco de dados antes de registrar o asset, mantendo a fidelidade com o sistema de arquivos do computador.
  - **Dependency Addition**: Adicionada a crate `async-recursion` ao backend para suportar a resolução eficiente da árvore de diretórios de forma assíncrona.
