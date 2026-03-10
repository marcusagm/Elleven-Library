# Sprint 8.1: Thumbnails Avançados, Formatos Suportados e DB Maintenance

**Status:** Pendente
**Data e hora de inicio:** -  
**Data da conclusão:** -

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
- [ ] Criar `LedgerCommand::RegenerateThumbnail { asset_id: String }`:
  - Limpar o registro de thumbnail no `asset_thumbnails_registry` (set `has_small = 0, has_medium = 0, has_large = 0`).
  - Emitir `DomainEvent::ThumbnailInvalidated { asset_id }`.
  - O ThumbnailWorker existente (sprint 4.1) reprocessará automaticamente assets sem thumbnail.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/thumbnails/commands.rs` → `request_thumbnail_regenerate()` simplesmente limpa o `thumbnail_path` e o worker reprocesa.

### 2. Implementar `get_library_supported_formats`
- [ ] Criar consulta que retorna todos os formatos registrados no `FormatRegistry`.
- [ ] O FormatRegistry V2 tem `by_extension` HashMap — iterar e construir a lista.
- [ ] Retornar struct compatível com o frontend:
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
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/library/commands/formats.rs` → `get_library_supported_formats()` retorna `Vec<FileFormat>` do `SUPPORTED_FORMATS`.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/formats/definitions.rs` → Contém o array estático `SUPPORTED_FORMATS`.

### 3. Implementar `run_db_maintenance`
- [ ] Criar IPC command que executa `VACUUM` e `ANALYZE` no pool SQLite.
- [ ] Implementar no `infra/database/manager.rs` ou no `queries.rs`:
  ```rust
  pub async fn run_maintenance(&self) -> AppResult<()> {
      sqlx::query("VACUUM").execute(&self.pool).await?;
      sqlx::query("ANALYZE").execute(&self.pool).await?;
      Ok(())
  }
  ```
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/settings/commands.rs` → `run_db_maintenance()`.

### 4. Implementar `send_telemetry_log`
- [ ] Simples IPC command que redireciona logs do frontend para o `tracing` do Rust:
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
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/settings/commands.rs` → `send_telemetry_log()` (8 linhas).

### 5. Implementar `get_audio_waveform_data`
- [ ] Este comando usa FFmpeg para gerar dados de waveform (array de floats).
- [ ] Avaliar duas abordagens:
  - **A) Migrar a lógica de `media/ffmpeg.rs`** do V1 diretamente para `processing/media/audio_format.rs` ou um módulo dedicado.
  - **B) Criar um módulo `feature/media/waveform.rs`** se a lógica for significativa.
- [ ] O comando invoca FFmpeg via subprocesso para analisar o áudio e retorna `Vec<f32>`.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/media/commands.rs` → `get_audio_waveform_data()` e `Mundam-main/src-tauri/src/media/ffmpeg.rs` → `get_audio_waveform()`.

### 6. Criar IPC Commands
- [ ] Em `delivery/tauri/commands/mutations.rs`:
  ```rust
  request_thumbnail_regenerate(asset_id) -> AppResult<()>
  run_db_maintenance() -> AppResult<()>
  send_telemetry_log(level, component, message) // void, não retorna Result
  ```
- [ ] Em `delivery/tauri/commands/queries.rs`:
  ```rust
  get_library_supported_formats() -> Vec<SupportedFormat>
  get_audio_waveform_data(path) -> AppResult<Vec<f32>>
  ```

### 7. Registrar no `lib.rs`
- [ ] Adicionar 5 novos commands ao `invoke_handler`.

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
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- 
