# Sprint 7.1: Tags CRUD Completo (Criar, Editar, Deletar, Listar por Asset)

**Status:** Pendente
**Data e hora de inicio:** -  
**Data da conclusão:** -

**Fase 7:** Paridade IPC — Taxonomia e Organização
**Objetivo:** Restaurar toda a gestão de Tags que o V1 possuía, agora passando pelo Ledger transacional. Isso inclui criação, edição, deleção de tags e operações de leitura por asset, além de batch operations.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. O frontend consegue criar uma tag com nome, cor e parent_id (tags hierárquicas).
2. O frontend consegue editar nome, cor, parent_id e order_index de uma tag existente.
3. O frontend consegue deletar uma tag, removendo-a automaticamente de todos os assets associados.
4. O frontend consegue consultar as tags associadas a um asset específico.
5. O frontend consegue aplicar tags em batch (múltiplos assets simultaneamente).
6. O frontend consegue remover tags em batch e substituir todas as tags de múltiplos assets.
7. `cargo build` compila sem warnings, `cargo clippy` passa limpo.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Adicionar novos `LedgerCommand` variants para Tags
- [ ] Em `src-tauri/src/core/ledger/command.rs`, adicionar:
  ```
  CreateTag { name: String, parent_id: Option<String>, color: Option<String> }
  UpdateTag { id: String, name: Option<String>, color: Option<String>, parent_id: Option<String>, order_index: Option<i64> }
  DeleteTag { id: String }
  ```
- [ ] Nota: No V1, tags usavam `i64` como ID. No V2, manter consistência com o modelo V2 (verificar se tags usam `String` UUID ou `i64`). A decisão deve seguir o schema do banco V2 em `src-tauri/migrations/`.

### 2. Implementar os handlers no `SqliteAssetLedger`
- [ ] Em `src-tauri/src/infra/database/ledger.rs`, implementar a execução dos 3 novos commands:
  - `CreateTag` → INSERT na tabela `tags` dentro de transação + emit `DomainEvent::TagCreated`
  - `UpdateTag` → UPDATE na tabela `tags` + emit `DomainEvent::TagUpdated`
  - `DeleteTag` → DELETE cascade da `asset_tags` + DELETE da `tags` + emit `DomainEvent::TagDeleted`
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/db/tags.rs` contém as queries SQLite originais.

### 3. Adicionar queries para Tags no `AssetQueryHandler`
- [ ] Em `src-tauri/src/core/repository/asset.rs` (trait `AssetQueryHandler`), adicionar:
  ```
  async fn get_tags_for_asset(&self, asset_id: &str) -> AppResult<Vec<Tag>>;
  ```
- [ ] Em `src-tauri/src/infra/database/queries.rs`, implementar a query com JOIN na `asset_tags`.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/db/tags.rs` → `get_tags_for_asset(asset_id: i64)`

### 4. Adicionar batch operations no Ledger
- [ ] Novos commands:
  ```
  AddTagsToAssetsBatch { asset_ids: Vec<String>, tag_ids: Vec<String> }
  RemoveTagsFromAssetsBatch { asset_ids: Vec<String>, tag_ids: Vec<String> }
  ReplaceTagsForAssetsBatch { asset_ids: Vec<String>, tag_ids: Vec<String> }
  ```
- [ ] Cada um deve rodar dentro de uma transação atômica no SQLite.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/db/tags.rs` → `add_tags_to_assets_batch`, `remove_tags_from_assets_batch`, `replace_tags_for_assets_batch`.

### 5. Adicionar novos DomainEvents
- [ ] Em `src-tauri/src/core/events/payloads.rs`, adicionar:
  ```
  TagCreated { id: String, name: String }
  TagUpdated { id: String }
  TagDeleted { id: String }
  ```
- [ ] Garantir que esses eventos tenham `#[derive(Clone, Debug, Serialize)]` e estejam na variante do enum.

### 6. Criar os IPC Commands no Delivery Layer
- [ ] Em `src-tauri/src/delivery/tauri/commands/mutations.rs`, adicionar:
  ```rust
  #[tauri::command]
  pub async fn create_tag(...) -> AppResult<Tag> { ... }
  
  #[tauri::command]
  pub async fn update_tag(...) -> AppResult<()> { ... }

  #[tauri::command]
  pub async fn delete_tag(...) -> AppResult<()> { ... }

  #[tauri::command]  
  pub async fn add_tags_to_assets_batch(...) -> AppResult<()> { ... }

  #[tauri::command]
  pub async fn remove_tags_from_assets_batch(...) -> AppResult<()> { ... }

  #[tauri::command]
  pub async fn replace_tags_for_assets_batch(...) -> AppResult<()> { ... }
  ```
- [ ] Em `src-tauri/src/delivery/tauri/commands/queries.rs`, adicionar:
  ```rust
  #[tauri::command]
  pub async fn get_tags_for_asset(...) -> AppResult<Vec<Tag>> { ... }
  ```

### 7. Registrar commands no `lib.rs`
- [ ] Adicionar os 7 novos commands ao `invoke_handler(tauri::generate_handler![...])`.

### 8. Verificar Frontend
- [ ] Verificar se o frontend chama esses comandos com os nomes corretos e tipos compatíveis.
- [ ] Os tipos do frontend devem esperar `String` IDs (UUIDs) e não `i64`.
- [ ] **Importante:** Se o frontend envia `i64` para tags, será necessário ajustar o frontend ou manter compatibilidade.

---

## 📁 Arquivos de Referência V1

| Funcionalidade    | Arquivo V1 (Mundam-main)                 | Notas                                           |
| ----------------- | ---------------------------------------- | ----------------------------------------------- |
| Tags SQL queries  | `src-tauri/src/db/tags.rs`               | Todas as queries de insert/update/delete/select |
| Tags IPC commands | `src-tauri/src/library/commands/tags.rs` | Assinaturas e lógica dos 16 comandos            |
| Tags model        | `src-tauri/src/db/models.rs`             | Struct `Tag` com campos                         |

## 📁 Arquivos a Modificar no V2

| Arquivo V2 (Mundam)                                  | Ação                                                |
| ---------------------------------------------------- | --------------------------------------------------- |
| `src-tauri/src/core/ledger/command.rs`               | Novos variants de LedgerCommand                     |
| `src-tauri/src/core/events/payloads.rs`              | Novos DomainEvents para tags                        |
| `src-tauri/src/core/repository/asset.rs`             | Nova trait fn `get_tags_for_asset`                  |
| `src-tauri/src/infra/database/ledger.rs`             | Handlers para CreateTag/UpdateTag/DeleteTag + batch |
| `src-tauri/src/infra/database/queries.rs`            | Query `get_tags_for_asset`                          |
| `src-tauri/src/delivery/tauri/commands/mutations.rs` | 6 novos IPC commands                                |
| `src-tauri/src/delivery/tauri/commands/queries.rs`   | 1 novo IPC command                                  |
| `src-tauri/src/lib.rs`                               | Registrar 7 novos commands                          |

---

## 💡 Notas para o Desenvolvedor / Agente
> **ATENÇÃO:** No V1, tags usam `i64` como IDs. No V2, pode ser que se use `i64` também (tags não são UUIDs, são auto-incremento). Verifique a tabela `tags` no schema V2 (migrations) antes de decidir o tipo. Se o schema V2 já usa `INTEGER PRIMARY KEY` para tags, mantenha `i64`. NÃO altere o schema do banco nesta sprint — apenas adicione funcionalidade sobre o que já existe.

> **REGRA:** Mutations SEMPRE via Ledger. Queries NUNCA pelo Ledger, vão direto ao QueryHandler.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- 
