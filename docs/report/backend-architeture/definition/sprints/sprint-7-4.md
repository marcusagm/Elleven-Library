# Sprint 7.4: Ratings, Notes, Metadata EXIF e Cores

**Status:** Concluído
**Data e hora de inicio:** 2026-03-10 09:00
**Data da conclusão:** 2026-03-11 10:43

**Fase 7:** Paridade IPC — Taxonomia e Organização
**Objetivo:** Restaurar as operações de edição de `rating` e `notes` por asset, a consulta de metadados EXIF via FormatProvider, e a exposição via IPC dos dados de cor extraídos pelo ColorWorker.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. O frontend consegue atribuir/alterar um rating (0-5 estrelas) a um asset.
2. O frontend consegue escrever e recuperar notas de texto livre por asset.
3. O frontend consegue consultar dados EXIF/técnicos de um asset por path.
4. O frontend consegue obter a paleta de cores extraída de um asset.
5. O frontend consegue forçar re-extração de cores para um asset.
6. `cargo build` compila sem warnings.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Adicionar Rating e Notes ao Ledger
- [x] Em `core/ledger/command.rs`, adicionar:
  ```
  UpdateAssetRating { asset_id: String, rating: i32 }
  UpdateAssetNotes { asset_id: String, notes: String }
  ```
- [x] Em `infra/database/ledger.rs`, implementar:
  - `UpdateAssetRating` → `UPDATE assets SET rating = ? WHERE id = ?` + emit `DomainEvent::AssetMetadataUpdated`.
  - `UpdateAssetNotes` → `UPDATE assets SET notes = ? WHERE id = ?` (verificar se coluna `notes` existe no schema V2, se não, adicionar via migration).
- [x] **Referência V1:** `Mundam-main/src-tauri/src/db/assets.rs` → `update_asset_rating()`, `update_asset_notes()`.

### 2. Verificar colunas `rating` e `notes` no schema V2
- [x] Verificar as migrations em `src-tauri/migrations/`. Se `rating` ou `notes` não existirem na tabela `v2_assets` ou `assets`:
  ```sql
  ALTER TABLE assets ADD COLUMN rating INTEGER DEFAULT 0;
  ALTER TABLE assets ADD COLUMN notes TEXT DEFAULT '';
  ```
- [x] Criar nova migration se necessário.

### 3. Implementar `get_asset_exif` via MetadataCapability
- [x] No V2, **ao invés** de usar o `metadata_reader.rs` do V1, utilizar o `FormatRegistry`:
  1. Resolver o provider for the path do asset.
  2. Invocar `provider.metadata()?.extract_technical(path)`.
  3. Retornar o `serde_json::Value` como `HashMap<String, String>` (ou `serde_json::Value` diretamente).
- [x] Criar IPC command:
  ```rust
  #[tauri::command]
  pub async fn get_asset_exif(
      registry: State<'_, Arc<FormatRegistry>>,
      path: String
  ) -> AppResult<serde_json::Value> { ... }
  ```
- [x] **Referência V1:** `Mundam-main/src-tauri/src/library/commands/metadata.rs` → `get_asset_exif()`, e `Mundam-main/src-tauri/src/media/metadata_reader.rs`.

### 4. Implementar Color Query IPC
- [x] Em `core/repository/asset.rs`, adicionar:
  ```rust
  async fn get_asset_colors(&self, asset_id: &str) -> AppResult<Vec<AssetColor>>;
  ```
- [x] Em `infra/database/queries.rs`, implementar SELECT na tabela `asset_colors`.
- [x] Verificar modelo `AssetColor` no V2 (pode já existir em `feature/analysis/colors.rs`).
- [x] **Referência V1:** `Mundam-main/src-tauri/src/db/colors.rs` → `get_asset_colors()`.

### 5. Implementar Re-extração de Cores (opcional, se existir infraestrutura)
- [x] O V2 já possui `processing/workers/color_worker.rs`. Verificar se é possível forçar re-extração de um asset individual.
- [x] Criar `LedgerCommand::ReextractColors { asset_id: String }` que limpa as cores existentes e agenda nova extração (ou executa imediatamente).
- [x] **Referência V1:** `Mundam-main/src-tauri/src/library/commands/colors.rs` → `reextract_asset_colors()` (113 linhas de lógica).

### 6. Criar IPC Commands
- [x] Em `delivery/tauri/commands/mutations.rs`:
  ```rust
  update_asset_rating(asset_id, rating) -> AppResult<()>
  update_asset_notes(asset_id, notes) -> AppResult<()>
  reextract_asset_colors(asset_id) -> AppResult<Vec<AssetColor>>
  ```
- [x] Em `delivery/tauri/commands/queries.rs`:
  ```rust
  get_asset_exif(path) -> AppResult<serde_json::Value>
  get_asset_colors(asset_id) -> AppResult<Vec<AssetColor>>
  ```

### 7. Registrar no `lib.rs`
- [x] Adicionar 5 novos commands ao `invoke_handler`.

---

## 📁 Arquivos de Referência V1

| Funcionalidade           | Arquivo V1 (Mundam-main)                           | Notas                 |
| ------------------------ | -------------------------------------------------- | --------------------- |
| update_asset_rating      | `src-tauri/src/library/commands/tags.rs` L149-152  | Simples update        |
| update_asset_notes       | `src-tauri/src/library/commands/tags.rs` L154-157  | Simples update        |
| get_asset_exif           | `src-tauri/src/library/commands/metadata.rs`       | Via metadata_reader   |
| Metadata reader EXIF     | `src-tauri/src/media/metadata_reader.rs`           | Extração bruta        |
| get_asset_colors         | `src-tauri/src/library/commands/colors.rs` L13-23  | Simples query         |
| reextract_asset_colors   | `src-tauri/src/library/commands/colors.rs` L25-113 | Lógica pesada         |
| Color palette extraction | `src-tauri/src/thumbnails/color_analysis.rs`       | Algoritmo k-means     |
| Colors DB queries        | `src-tauri/src/db/colors.rs`                       | Insert/select colores |

## 📁 Arquivos a Modificar no V2

| Arquivo V2 (Mundam)                                  | Ação                                                       |
| ---------------------------------------------------- | ---------------------------------------------------------- |
| `src-tauri/migrations/`                              | Nova migration para `rating`/`notes` se ausentes           |
| `src-tauri/src/core/ledger/command.rs`               | `UpdateAssetRating`, `UpdateAssetNotes`, `ReextractColors` |
| `src-tauri/src/core/repository/asset.rs`             | `get_asset_colors` trait fn                                |
| `src-tauri/src/infra/database/ledger.rs`             | Handlers rating/notes/reextract                            |
| `src-tauri/src/infra/database/queries.rs`            | `get_asset_colors` query                                   |
| `src-tauri/src/delivery/tauri/commands/mutations.rs` | 3 novos IPC                                                |
| `src-tauri/src/delivery/tauri/commands/queries.rs`   | 2 novos IPC                                                |
| `src-tauri/src/lib.rs`                               | Registrar 5 commands                                       |

---

## 💡 Notas para o Desenvolvedor / Agente
> O `get_asset_exif` no V2 deve usar o FormatRegistry que é a arquitetura correta. O V1 usava um `metadata_reader.rs` genérico. No V2, cada FormatProvider implementa `MetadataCapability` com `extract_technical()` — use isso.

> Para cores, o V2 já possui o `ColorWorker` reativo (sprint 4.3). O IPC `get_asset_colors` é apenas a PORTA de leitura. A re-extração pode ser simplificada: limpar os dados do BD e re-agendar o asset no color worker via event.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- Sincronização de tipos de ID (UUID string vs int i64 do V1) em todas as camadas do sistema (Database, Core, Frontend).
- Implementação da re-extração de cores mantendo a reatividade do `ColorWorker` baseada em eventos.
- Alinhamento de propriedades de DTOs (`AssetSummaryDto`) para evitar campos `undefined` no frontend.

### Melhorias Realizadas
- Arquitetura de extração de metadados técnica centralizada no `FormatRegistry`, eliminando o leitor genérico centralizado do V1.
- Uso de `DomainEvent` para notificar mudanças de metadados, permitindo que outros componentes respondam a alterações de `rating` e `notes`.
- Expansão do `AssetSummaryDto` para incluir dimensões e caminho da thumbnail, melhorando a performance do grid.

### 📄 Arquivos Criados ou Modificados
- `src-tauri/migrations/20250310000000_add_rating_notes.sql`
- `src-tauri/src/core/ledger/command.rs`
- `src-tauri/src/core/repository/asset.rs`
- `src-tauri/src/infra/database/ledger.rs`
- `src-tauri/src/infra/database/queries.rs`
- `src-tauri/src/delivery/tauri/commands/mutations.rs`
- `src-tauri/src/delivery/tauri/commands/queries.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/core/models/asset.rs`
- `src-tauri/src/infra/database/models.rs`
