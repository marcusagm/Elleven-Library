# Sprint 7.3: Smart Folders CRUD e Contadores de Biblioteca

**Status:** Concluído
**Data e hora de inicio:** 2026-03-10 01:10
**Data da conclusão:** 2026-03-11 08:56

**Fase 7:** Paridade IPC — Taxonomia e Organização
**Objetivo:** Implementar o CRUD completo de Smart Folders (pastas virtuais salvas com query JSON), o contador geral `get_asset_count_filtered` e o `get_library_stats` — funcionalidades core que alimentam a sidebar e badges do frontend.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. [x] O frontend consegue criar, editar, deletar e listar Smart Folders salvas.
2. [x] O frontend consegue obter o count total de assets com os mesmos filtros usados em `get_assets`.
3. [x] O frontend consegue obter estatísticas gerais da biblioteca (total assets, total folders, total tags, tamanho total em bytes).
4. [x] `cargo build` compila sem warnings.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Verificar/Criar tabela `smart_folders` no schema V2
- [x] Verificar se a tabela `smart_folders` já existe nas migrations V2 (`src-tauri/migrations/`).
- [x] Se **NÃO** existir, criar nova migration:
  ```sql
  CREATE TABLE IF NOT EXISTS smart_folders (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      query_json TEXT NOT NULL,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  );
  ```
- [x] **Referência V1:** `Mundam-main/src-tauri/src/db/smart_folders.rs` — schema e queries.

### 2. Criar modelo `SmartFolder` no V2
- [x] Em `src-tauri/src/core/models/` (ou onde estiver o modelo adequado), adicionar:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct SmartFolder {
      pub id: String,
      pub name: String,
      pub query_json: String,
  }
  ```

### 3. Adicionar Smart Folders ao QueryHandler
- [x] Em `core/repository/asset.rs` (trait `AssetQueryHandler`):
  ```rust
  async fn list_smart_folders(&self) -> AppResult<Vec<SmartFolder>>;
  async fn get_asset_count(&self, filter: &AssetFilter) -> AppResult<i64>;
  async fn get_library_stats(&self) -> AppResult<LibraryStats>;
  ```
- [x] Criar struct `LibraryStats`:
  ```rust
  #[derive(Debug, Serialize)]
  pub struct LibraryStats {
      pub total_assets: i64,
      pub total_folders: i64,
      pub total_tags: i64,
      pub total_size_bytes: i64,
  }
  ```

### 4. Adicionar Smart Folders ao Ledger (mutações)
- [x] Novos `LedgerCommand` variants:
  ```
  CreateSmartFolder { name: String, query_json: String }
  UpdateSmartFolder { id: String, name: String, query_json: String }
  DeleteSmartFolder { id: String }
  ```
- [x] Implementar no `SqliteAssetLedger` com transação.

### 5. Implementar queries no SqliteAssetQueries
- [x] `list_smart_folders`: SELECT * FROM smart_folders.
- [x] `get_asset_count`: reutilizar o `SearchBuilder` já existente, retornando COUNT(*) em vez de rows.
- [x] `get_library_stats`: Query agregada com UNION ou múltiplas SELECTs.
- [x] **Referência V1:** `Mundam-main/src-tauri/src/db/smart_folders.rs` e `db/assets.rs` → `get_asset_count_filtered`.

### 6. Criar IPC Commands
- [x] Em `delivery/tauri/commands/queries.rs`:
  ```rust
  get_smart_folders() -> AppResult<Vec<SmartFolder>>
  get_asset_count_filtered(filter, page) -> AppResult<i64>
  get_library_stats() -> AppResult<LibraryStats>
  ```
- [x] Em `delivery/tauri/commands/mutations.rs`:
  ```rust
  save_smart_folder(name, query) -> AppResult<SmartFolder>
  update_smart_folder(id, name, query) -> AppResult<()>
  delete_smart_folder(id) -> AppResult<()>
  ```

### 7. Registrar no `lib.rs`
- [x] Adicionar 6 novos commands ao `invoke_handler`.

### 8. Verificar Frontend
- [x] Confirmar que o frontend usa `get_smart_folders`, `save_smart_folder`, etc.
- [x] Verificar se `get_asset_count_filtered` no frontend envia parâmetros compatíveis com V2 `AssetFilter`.

---

## 📁 Arquivos de Referência V1

| Funcionalidade       | Arquivo V1 (Mundam-main)                          | Notas                    |
| -------------------- | ------------------------------------------------- | ------------------------ |
| Smart Folders CRUD   | `src-tauri/src/library/commands/smart_folders.rs` | 4 commands simples       |
| Smart Folders DB     | `src-tauri/src/db/smart_folders.rs`               | SQL queries              |
| SmartFolder model    | `src-tauri/src/db/models.rs`                      | Struct SmartFolder       |
| Asset count filtered | `src-tauri/src/library/commands/tags.rs`          | get_asset_count_filtered |
| Library stats        | `src-tauri/src/library/commands/tags.rs`          | get_library_stats        |
| Stats DB query       | `src-tauri/src/db/assets.rs`                      | get_library_stats()      |

## 📁 Arquivos a Modificar no V2

| Arquivo V2 (Mundam)                                  | Ação                                                   |
| ---------------------------------------------------- | ------------------------------------------------------ |
| `src-tauri/migrations/`                              | Nova migration para `smart_folders` (se não existir)   |
| `src-tauri/src/core/models/`                         | `SmartFolder`, `LibraryStats` structs                  |
| `src-tauri/src/core/ledger/command.rs`               | 3 novos LedgerCommands                                 |
| `src-tauri/src/core/repository/asset.rs`             | Novas trait fns                                        |
| `src-tauri/src/infra/database/ledger.rs`             | Handlers smart folder                                  |
| `src-tauri/src/infra/database/queries.rs`            | list_smart_folders, get_asset_count, get_library_stats |
| `src-tauri/src/delivery/tauri/commands/queries.rs`   | 3 novos IPC                                            |
| `src-tauri/src/delivery/tauri/commands/mutations.rs` | 3 novos IPC                                            |
| `src-tauri/src/lib.rs`                               | Registrar 6 commands                                   |

---

## 💡 Notas para o Desenvolvedor / Agente
> Smart Folders são **read-heavy**: o frontend carrega a lista no boot. As mutações são raras (o usuário cria/edita smart folders de vez em quando). Mesmo assim, mutações devem passar pelo Ledger para consistência e audit-log.

> O `get_asset_count_filtered` é chamado pelo frontend para exibir badges de contagem na sidebar. Ele usa os mesmos filtros do `get_assets` mas retorna apenas o COUNT. Reutilize o `SearchBuilder` existente.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- **Inconsistência de Tipos (ID Mismatch):** Identificada uma discrepância crítica onde o backend V2 utiliza UUIDs (`String`), mas partes do frontend e IPC em TS ainda esperavam `number` (comportamento herdado da V1). Isso causava erros de `null` e `undefined` durante o parse de JSON.
- **Pânico no Worker de Thumbnails:** Conflito entre o pool de threads do `Rayon` (síncrono/paralelo) e operações assíncronas do `Tokio`. Resolvido capturando o `Handle` do runtime antes da execução paralela para garantir o contexto adequado do executor Tokio.

### Melhorias Realizadas
- **Padronização de IDs:** Todos os IDs de Tags e Smart Folders foram convertidos para `string` em toda a stack (Frontend -> IPC -> Backend), garantindo consistência com o padrão UUID estabelecido na V2.
- **Refatoração de Stats:** A struct `LibraryStats` foi expandida e padronizada com `snake_case`, incluindo novos campos para contagem de assets sem tag e detalhamento por pastas.
- **Qualidade de Código:** Limpeza completa de warnings de compilação e linting (`clippy` no Rust e `prettier` no TypeScript).

### 📄 Arquivos Criados ou Modificados
**Backend (Rust):**
- `src-tauri/migrations/20260311020000_create_smart_folders.sql`
- `src-tauri/src/core/models/smart_folder.rs`
- `src-tauri/src/core/models/mod.rs`
- `src-tauri/src/core/models/asset.rs`
- `src-tauri/src/core/repository/asset.rs`
- `src-tauri/src/core/ledger/command.rs`
- `src-tauri/src/infra/database/ledger.rs`
- `src-tauri/src/infra/database/queries.rs`
- `src-tauri/src/infra/database/models.rs`
- `src-tauri/src/delivery/tauri/commands/queries.rs`
- `src-tauri/src/delivery/tauri/commands/mutations.rs`
- `src-tauri/src/feature/assets/queries.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/processing/workers/thumbnail_worker.rs`

**Frontend (TypeScript):**
- `src/lib/tags.ts`
- `src/core/store/metadata/tagActions.ts`
- `src/core/store/viewportStore.ts`
- `src/core/viewport/ViewportController.ts`
- `src/core/viewport/layout.worker.ts`
- `src/core/hooks/useVirtualViewport.ts`
- `src/core/hooks/useGridKeyboardNav.ts`
- `src/components/features/search/AdvancedSearchModal.tsx`
- `src/components/features/search/useAdvancedSearch.ts`
