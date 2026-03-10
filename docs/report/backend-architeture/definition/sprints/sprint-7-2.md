# Sprint 7.2: Folders Avançados, Indexação Manual e Contadores

**Status:** Pendente
**Data e hora de inicio:** -  
**Data da conclusão:** -

**Fase 7:** Paridade IPC — Taxonomia e Organização
**Objetivo:** Restaurar as operações avançadas de pastas: remoção de location (com limpeza de thumbnails e stop do watcher), listagem de subpastas, contadores por pasta, e o comando `start_indexing` que permite ao usuário disparar manualmente um re-scan.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. O frontend consegue remover uma location, parando o watcher e excluindo assets + thumbnails associados.
2. O frontend consegue listar toda a árvore de subfolders (hierárquica).
3. O frontend consegue obter contadores de assets por subfolder.
4. O frontend consegue disparar manualmente um `start_indexing` para uma pasta específica.
5. `cargo build` compila sem warnings.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Implementar `remove_location` no Ledger
- [ ] Adicionar `LedgerCommand::RemoveFolder { folder_id: String }` em `core/ledger/command.rs`.
- [ ] No handler em `infra/database/ledger.rs`:
  1. Buscar o path da pasta pelo `folder_id`.
  2. Obter lista de thumbnails dos assets associados (para limpeza de FS).
  3. Deletar assets da pasta (cascade: metadata, tags, colors, thumbnails registry).
  4. Deletar a pasta do banco.
  5. Emitir `DomainEvent::FolderRemoved { id, path }`.
- [ ] **Após** o Ledger retornar sucesso, o IPC command deve:
  1. Limpar arquivos de thumbnail do filesystem (`thumbnails_dir/`).
  2. Parar o watcher para o path removido.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/library/commands/folders.rs` → `remove_location()` (linhas 100-154) — lógica completa com limpeza de thumbs + stop watcher.

### 2. Adicionar queries de subfolders e contadores
- [ ] Em `core/repository/asset.rs` (trait `AssetQueryHandler`), adicionar:
  ```rust
  async fn list_all_subfolders(&self) -> AppResult<Vec<Folder>>;
  async fn get_subfolder_asset_counts(&self) -> AppResult<Vec<(String, i64)>>;
  async fn get_location_root_counts(&self) -> AppResult<Vec<(String, i64)>>;
  ```
- [ ] Implementar em `infra/database/queries.rs`:
  - `list_all_subfolders`: SELECT * FROM folders ORDER BY path (retorna toda hierarquia).
  - `get_subfolder_asset_counts`: COUNT de assets agrupados por folder_id, recursivamente incluindo subpastas.
  - `get_location_root_counts`: COUNT de assets por root folder.
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/db/folders.rs` → `get_folder_hierarchy()`, `get_folder_counts_recursive()`.

### 3. Implementar `start_indexing` IPC command
- [ ] Criar IPC command em `delivery/tauri/commands/mutations.rs`:
  ```rust
  #[tauri::command]
  pub async fn start_indexing(path: String, ...) -> AppResult<()> { ... }
  ```
- [ ] Lógica: Instanciar o `LibraryIndexer` existente e chamar `scan_directory_tree(&path)`.
- [ ] Deve spawnar a task assincronamente (não bloquear o IPC).
- [ ] **Referência V1:** `Mundam-main/src-tauri/src/library/commands/indexing.rs` → `start_indexing()` (40 linhas).
- [ ] **V2 já possui:** `feature/library/indexer.rs` → `LibraryIndexer::scan_directory_tree()`.

### 4. Criar IPC Commands no Delivery Layer
- [ ] Em `delivery/tauri/commands/mutations.rs`:
  ```rust
  remove_location(folder_id: String, ...) -> AppResult<()>
  start_indexing(path: String, ...) -> AppResult<()>
  ```
- [ ] Em `delivery/tauri/commands/queries.rs`:
  ```rust
  get_all_subfolders(...) -> AppResult<Vec<Folder>>
  get_subfolder_counts(...) -> AppResult<Vec<(String, i64)>>
  get_location_root_counts(...) -> AppResult<Vec<(String, i64)>>
  ```

### 5. Registrar commands no `lib.rs`
- [ ] Adicionar os 5 novos commands ao `invoke_handler`.

### 6. Verificar integração com WatcherService
- [ ] A remoção de location deve chamar `watcher.unwatch(path)` para parar o monitoramento.
- [ ] Verificar se `processing/watcher/sensor.rs` tem método `unwatch()`. Se não, implementar.

---

## 📁 Arquivos de Referência V1

| Funcionalidade                   | Arquivo V1 (Mundam-main)                     | Notas                        |
| -------------------------------- | -------------------------------------------- | ---------------------------- |
| Remove location + thumbs cleanup | `src-tauri/src/library/commands/folders.rs`  | Linhas 100-154               |
| Add location + start scan        | `src-tauri/src/library/commands/folders.rs`  | Linhas 19-98                 |
| Start indexing                   | `src-tauri/src/library/commands/indexing.rs` | Lógica completa              |
| Folder DB queries                | `src-tauri/src/db/folders.rs`                | get_folder_hierarchy, counts |
| Indexer                          | `src-tauri/src/indexer/scan.rs`              | Scan recursivo               |
| Indexer Types                    | `src-tauri/src/indexer/types.rs`             | Structs auxiliares           |
| Indexer Watcher                  | `src-tauri/src/indexer/watcher.rs`           | WatcherRegistry              |

## 📁 Arquivos a Modificar no V2

| Arquivo V2 (Mundam)                                  | Ação                                   |
| ---------------------------------------------------- | -------------------------------------- |
| `src-tauri/src/core/ledger/command.rs`               | Novo `RemoveFolder`                    |
| `src-tauri/src/core/events/payloads.rs`              | `FolderRemoved` event                  |
| `src-tauri/src/core/repository/asset.rs`             | Novas trait fns para subfolders/counts |
| `src-tauri/src/infra/database/ledger.rs`             | Handler `RemoveFolder`                 |
| `src-tauri/src/infra/database/queries.rs`            | Queries subfolders/counts              |
| `src-tauri/src/delivery/tauri/commands/mutations.rs` | `remove_location`, `start_indexing`    |
| `src-tauri/src/delivery/tauri/commands/queries.rs`   | `get_all_subfolders`, counts           |
| `src-tauri/src/processing/watcher/sensor.rs`         | Verificar/adicionar `unwatch()`        |
| `src-tauri/src/lib.rs`                               | Registrar 5 novos commands             |

---

## 💡 Notas para o Desenvolvedor / Agente
> A lógica de `remove_location` no V1 é complexa: ela obtém thumbnails antes de deletar o folder, depois limpa os arquivos de thumb do filesystem, e finalmente para o watcher. No V2, a limpeza de thumbnails e parada do watcher devem acontecer FORA do Ledger (depois que o Ledger confirma a deleção no BD). O Ledger cuida apenas da mutação de banco.

> O `start_indexing` no V2 já possui a infraestrutura (`LibraryIndexer`). A sprint é sobre EXPOR isso via IPC e garantir a integração.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- 
