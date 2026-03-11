# Sprint 8.1: Thumbnails Avançados, Formatos Suportados e DB Maintenance

**Status:** Concluído
**Data e hora de inicio:** 2026-03-11 10:43
**Data da conclusão:** 2026-03-11 11:03

**Fase 8:** Paridade IPC — Mídia, Manutenção e Utilidades
**Objetivo:** Restaurar as funcionalidades utilitárias restantes: regeneração de thumbnails, listagem de formatos suportados, manutenção de BD (VACUUM/ANALYZE), telemetria frontend e geração de audio waveforms.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. O frontend pode solicitar regeneração de thumbnail para um asset específico.
2. O frontend pode obter a lista completa de formatos suportados pelo backend (para UI de filtros).
3. O frontend pode disparar manutenção do BD (VACUUM/ANALYZE).
4. O frontend pode enviar logs de telemetria que aparecem no tracing do backend.
5. O frontend pode obter dados de waveform para assets de áudio.
6. `cargo build` compila sem warnings.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Implementar `request_thumbnail_regenerate`
- [x] Criar `LedgerCommand::RegenerateThumbnail { asset_id: String }`:
  - Limpar o registro de thumbnail no `asset_thumbnails_registry` (set `has_small = 0, has_medium = 0, has_large = 0`).
  - Emitir `DomainEvent::ThumbnailInvalidated { asset_id }`.
  - O ThumbnailWorker existente (sprint 4.1) reprocessará automaticamente assets sem thumbnail.
- [x] **Referência V1:** `Mundam-main/src-tauri/src/thumbnails/commands.rs` → `request_thumbnail_regenerate()` simplesmente limpa o `thumbnail_path` e o worker reprocesa.

### 2. Implementar `get_library_supported_formats`
- [x] Criar consulta que retorna todos os formatos registrados no `FormatRegistry`.
- [x] O FormatRegistry V2 tem `by_extension` HashMap — iterar e construir a lista.
- [x] Retornar struct compatível com o frontend:
  ```rust
  #[derive(Serialize)]
  pub struct SupportedFormat {
      pub extensions: Vec<String>,
      pub name: String,
      pub family: String,
      pub has_thumbnail: bool,
      pub has_metadata: bool,
  }
  ```
- [x] **Referência V1:** `Mundam-main/src-tauri/src/library/commands/formats.rs` → `get_library_supported_formats()` retorna `Vec<FileFormat>` do `SUPPORTED_FORMATS`.
- [x] **Referência V1:** `Mundam-main/src-tauri/src/formats/definitions.rs` → Contém o array estático `SUPPORTED_FORMATS`.

### 3. Implementar `run_db_maintenance`
- [x] Criar IPC command que executa `VACUUM` e `ANALYZE` no pool SQLite.
- [x] Implementar no `infra/database/manager.rs` ou no `queries.rs`:
  ```rust
  pub async fn run_maintenance(&self) -> AppResult<()> {
      sqlx::query("VACUUM").execute(&self.pool).await?;
      sqlx::query("ANALYZE").execute(&self.pool).await?;
      Ok(())
  }
  ```
- [x] **Referência V1:** `Mundam-main/src-tauri/src/settings/commands.rs` → `run_db_maintenance()`.

### 4. Implementar `send_telemetry_log`
- [x] Simples IPC command que redireciona logs do frontend para o `tracing` do Rust:
  ```rust
  #[tauri::command]
  pub fn send_telemetry_log(level: String, component: String, message: String) {
      match level.to_lowercase().as_str() {
          "error" => tracing::error!(component = %component, "{}", message),
          "warn" => tracing::warn!(component = %component, "{}", message),
          "debug" => tracing::debug!(component = %component, "{}", message),
          _ => tracing::info!(component = %component, "{}", message),
      }
  }
  ```
- [x] **Referência V1:** `Mundam-main/src-tauri/src/settings/commands.rs` → `send_telemetry_log()` (8 linhas).

### 5. Implementar `get_audio_waveform_data`
- [x] Este comando usa FFmpeg para gerar dados de waveform (array de floats).
- [x] Avaliar duas abordagens:
  - **A) Migrar a lógica de `media/ffmpeg.rs`** do V1 diretamente para `processing/media/audio_format.rs` ou um módulo dedicado.
  - **B) Criar um módulo `feature/media/waveform.rs`** se a lógica for significativa.
- [x] O comando invoca FFmpeg via subprocesso para analisar o áudio e retorna `Vec<f32>`.
- [x] **Referência V1:** `Mundam-main/src-tauri/src/media/commands.rs` → `get_audio_waveform_data()` e `Mundam-main/src-tauri/src/media/ffmpeg.rs` → `get_audio_waveform()`.

### 6. Criar IPC Commands
- [x] Em `delivery/tauri/commands/mutations.rs`:
  ```rust
  request_thumbnail_regenerate(asset_id) -> AppResult<()>
  run_db_maintenance() -> AppResult<()>
  send_telemetry_log(level, component, message) // void, não retorna Result
  ```
- [x] Em `delivery/tauri/commands/queries.rs`:
  ```rust
  get_library_supported_formats() -> Vec<SupportedFormat>
  get_audio_waveform_data(path) -> AppResult<Vec<f32>>
  ```

### 7. Registrar no `lib.rs`
- [x] Adicionar 5 novos commands ao `invoke_handler`.

---

## 📁 Arquivos de Referência V1

| Funcionalidade       | Arquivo V1 (Mundam-main)                     | Notas                     |
| -------------------- | -------------------------------------------- | ------------------------- |
| Thumbnail regenerate | `src-tauri/src/thumbnails/commands.rs` L6-11 | Limpa thumbnail_path      |
| Supported formats    | `src-tauri/src/library/commands/formats.rs`  | Retorna SUPPORTED_FORMATS |
| Format definitions   | `src-tauri/src/formats/definitions.rs`       | Array estático            |
| DB maintenance       | `src-tauri/src/settings/commands.rs` L23-26  | VACUUM + ANALYZE          |
| Telemetry log        | `src-tauri/src/settings/commands.rs` L28-36  | Redirect to tracing       |
| Audio waveform cmd   | `src-tauri/src/media/commands.rs`            | 15 linhas                 |
| Audio waveform logic | `src-tauri/src/media/ffmpeg.rs`              | get_audio_waveform()      |

## 📁 Arquivos a Modificar no V2

| Arquivo V2 (Mundam)                                  | Ação                              |
| ---------------------------------------------------- | --------------------------------- |
| `src-tauri/src/core/ledger/command.rs`               | `RegenerateThumbnail`             |
| `src-tauri/src/core/events/payloads.rs`              | `ThumbnailInvalidated` event      |
| `src-tauri/src/infra/database/ledger.rs`             | Handler regenerate                |
| `src-tauri/src/infra/database/manager.rs`            | `run_maintenance()`               |
| `src-tauri/src/core/formats/registry.rs`             | Método para listar formatos       |
| `src-tauri/src/feature/media/` (novo)                | `waveform.rs` para audio waveform |
| `src-tauri/src/delivery/tauri/commands/mutations.rs` | 3 novos IPC                       |
| `src-tauri/src/delivery/tauri/commands/queries.rs`   | 2 novos IPC                       |
| `src-tauri/src/lib.rs`                               | Registrar 5 commands              |

---

## 💡 Notas para o Desenvolvedor / Agente
> O `get_library_supported_formats` no V1 retorna um array estático definido em compilação. No V2, o FormatRegistry é dinâmico. **A vantagem V2 é que formatos adicionados via `register()` aparecem automaticamente.** Itere sobre `by_extension` do registry para construir a lista.

> O `get_audio_waveform_data` é uma funcionalidade que não tem relação direta com o Ledger ou Event Bus. É uma utilidade pura de processamento. Pode ir direto como async task com spawn_blocking + FFmpeg CLI.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- Orquestração de subprocessos FFmpeg com timeouts para evitar processos zumbis durante a extração de waveforms.
- Normalização e downsampling de grandes volumes de dados de áudio para um formato leve (500 pontos) consumível pelo frontend.
- **Incompatibilidade de Viewport:** Identificada divergência entre os nomes de propriedades do backend V2 (`name`, `format_type`, `file_size`) e as expectativas do frontend V1 (`filename`, `format`, `size`).
- **Mismatch de Comandos:** O frontend utilizava o comando `set_thumbnail_priority`, enquanto o backend V2 havia implementado como `prioritize_thumbnails`.
- **Lock de Compilação:** Durante a execução concorrente do Tauri, o `cargo check` foi bloqueado pelo lock do diretório de build, exigindo sincronização manual.
- **Gestão de Sessão:** Implementação de tokens de segurança UUID para o servidor de streaming HLS embutido.
- **Mapeamento de DTOs:** Necessidade de alinhar propriedades complexas (dimensões, caminhos de cache) entre Rust e Typescript.
- **Ambiguidade de Colunas (SQL):** Erro de `ambiguous column name: created_at` causado por joins entre as tabelas `assets` e `asset_metadata_envelope`, ambas contendo colunas com o mesmo nome (`created_at`, `updated_at`, `rating`, `notes`).
- **Protocolo de Assets Frontend:** Adoção do novo protocolo `asset://` em substituição aos protocolos legados (`thumb://`, `image://`, `font://`, `model://`).
- **Identificação por UUID:** Migração da carga de assets (thumbnails e originais) para uso exclusivo de `asset_id` (UUID) em vez de caminhos de arquivos físicos, garantindo isolamento da camada de dados e compatibilidade com o backend hexagonal (V2).
- **Suporte a GLB no Backend:** Extensão do handler do protocolo `asset` para suportar o parâmetro `type=glb`, permitindo a visualização de modelos 3D convertidos armazenados no cache de thumbnails.
- **Otimização de Pesquisa Avançada:** Corrigido erro de `DATABASE_ERROR: no column found for name: updated_at` no comando `search_assets` através da inclusão de todos os campos obrigatórios na projeção SQL.
- **Robustez no Gerenciamento de Tags:** Implementada lógica de "retry" sincronizado no frontend para garantir que tags recém-criadas ou movidas sejam corretamente localizadas no store antes do processamento de drag-and-drop, eliminando o erro `Dragged tag not found`.

### Melhorias Realizadas
- **Mapeamento Transparente:** Adicionado `#[serde(rename = "...")]` nos structs `Asset` e `AssetSummaryDto` para garantir paridade com o frontend sem alterar a lógica de domínio do Rust.
- **Restauração de Compatibilidade:** Renomeado o comando de priorização de thumbnails para `set_thumbnail_priority` para evitar alterações no código legível do frontend.
- Integração dinâmica do `FormatRegistry`, permitindo que novos providers de plugins sejam descobertos automaticamente pela UI de filtros do frontend.
- Centralização da lógica de manutenção do banco de dados no `DbManager` hexagonal.
- **Gestão de Estado:** Corrigido o gerenciamento de estado do `DbManager` no Tauri (`handle.manage`), permitindo que comandos como `run_db_maintenance` acessem o pool de conexão corretamente.
- **Abstração de Configurações:** Implementação de comandos de configuração genéricos (`get_setting`/`set_setting`) para suportar personalizações da UI (atalhos, aparência).
- **Consistência de Dados:** Expansão do `AssetSummaryDto` para incluir metadados essenciais para o grid, reduzindo roundtrips IPC.
- **Resolução de Ambiguidade SQL:** Implementação de prefixos de tabela (`a.` para assets e `m.` para metadata) de forma sistemática em todas as queries JOIN dinâmicas (usando `QueryBuilder`) e estáticas (macros `query!`), garantindo estabilidade nas chamadas `get_assets` e `search_assets`.
- **Padronização de Atribuição de Tags:** Refatoração do payload `UpdateTags` para utilizar UUIDs de tags em vez de nomes, eliminando falhas de associação e atribuições nulas no banco de dados.
- **Correção de Contagem de Itens em Pastas:** Implementação de resolução de `folder_id` lógico por caminho físico no `LibraryIndexer`, garantindo que assets descobertos via scan diferencial sejam corretamente vinculados à hierarquia da biblioteca e contabilizados nas estatísticas.
- **Sincronização de Batch IPC:** Atualização do frontend em `tags.ts` para utilizar chamadas IPC em lote (`batch`) padronizadas, otimizando a performance em operações massivas de tagging.

### 📄 Arquivos Criados ou Modificados
- `src-tauri/src/core/models/asset.rs`
- `src-tauri/src/delivery/tauri/thumbnails.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/delivery/tauri/commands/mutations.rs`
- `src-tauri/src/delivery/tauri/commands/queries.rs`
- `src-tauri/src/infra/database/manager.rs`
- `src-tauri/src/feature/media/waveform.rs`
- `src-tauri/src/core/ledger/command.rs`
- `src-tauri/src/infra/database/ledger.rs`
- `src-tauri/src/core/settings/model.rs`
- `src-tauri/src/feature/settings/service.rs`
- `src-tauri/src/delivery/tauri/commands/settings.rs`
- `src-tauri/src/infra/database/models.rs`
- `src-tauri/src/infra/database/queries.rs`
- `src-tauri/src/infra/database/search_builder.rs`
- `src-tauri/src/feature/assets/queries.rs`
- `src-tauri/src/delivery/protocols/asset.rs`
- `src-tauri/src/core/repository/asset.rs`
- `src-tauri/src/feature/library/indexer.rs`
- `src/components/features/viewport/assets/Thumbnail.tsx`
- `src/components/features/viewport/layouts/VirtualListView.tsx`
- `src/components/features/inspector/image/ImageInspector.tsx`
- `src/components/features/inspector/font/FontInspector.tsx`
- `src/components/features/inspector/model/ModelInspector.tsx`
- `src/components/features/inspector/multi/MultiInspector.tsx`
- `src/components/features/itemview/ItemView.tsx`
- `src/components/features/itemview/renderers/model/ModelViewer.tsx`
- `src/core/dnd/ghost.ts`
- `src/lib/tags.ts`
- `src/core/store/metadata/tagActions.ts`
- `src/core/store/library/itemActions.ts`
- `src/components/features/viewport/layouts/VirtualGridView.tsx`
