# Ledger: God Adapter Refactoring & Saga/Outbox Pattern

**Date:** 2026-06-13  
**Status:** Implemented & Validated  
**Author:** Engineering Session (AI-Assisted)

---

## 1. Executive Summary

This document describes the two-phase architectural refactoring performed on the `SqliteAssetLedger` infrastructure adapter in the Mundam V2 backend.

**Phase 1** resolved the "God Adapter" anti-pattern by decomposing all domain-specific SQL logic out of the Ledger into dedicated handler modules.

**Phase 2** implemented the **Saga/Outbox pattern** to close a critical architectural gap: the lack of real atomicity between SQLite database transactions and filesystem I/O operations (e.g., physically deleting a file from disk).

The result is a Ledger that acts as a pure transactional router and event emitter, with each domain concern fully encapsulated in its own handler module.

---

## 2. Context & Problem Statement

### 2.1 The God Adapter Problem

Before this refactoring, `SqliteAssetLedger` was a single file exceeding 1,500 lines that mixed:

- Raw SQL queries (INSERT, UPDATE, DELETE)
- Move detection logic (signature-based file recovery)
- Audit log writes (`asset_operations_log`)
- Domain event emission (`AppEventBus`)
- Path normalization (NFC/NFD Unicode handling)
- Business rules for each operation type (colors, metadata, thumbnails, tags, etc.)

This "God Adapter" made the file increasingly difficult to navigate, test in isolation, and extend without introducing regressions.

### 2.2 The Atomicity Gap

The Ledger's port contract (`TransactionalAssetLedger`) declared:

> "Ensures atomicity between database updates and filesystem operations."

This was **an unfulfilled promise**. The implementation wrapped only database operations in SQLite transactions. Filesystem operations (physical file deletion, file moves) happened **outside** any transactional boundary. A process crash between a `tx.commit()` and a `tokio::fs::remove_file()` would leave the system in an inconsistent state:

- The asset record deleted from the database ✅
- The physical file still present on disk ❌ (orphaned)

---

## 3. Phase 1 — Modular Handler Extraction

### 3.1 Architecture Decision

Rather than keeping the Ledger as a monolith or moving logic to a service layer, the chosen design was to create **internal infrastructure handlers**: pure functions (no `self`, no state) that receive a database transaction reference, perform their specific SQL mutations, write the audit log, and return the domain result.

This keeps the transaction boundary (SQLite `BEGIN`/`COMMIT`) entirely owned by the Ledger, while each handler is independently testable in isolation.

### 3.2 Handler Module Inventory

All handlers live in `src/infra/database/handlers/`.

| Module                 | File                      | Responsibility                                                                        |
| ---------------------- | ------------------------- | ------------------------------------------------------------------------------------- |
| `asset_handler`        | `asset_handler.rs`        | Asset creation (single + batch), V1 move recovery, logical/physical deletion          |
| `folder_handler`       | `folder_handler.rs`       | Folder creation (UPSERT + hierarchy adoption), cascade removal, recursive path rename |
| `smart_folder_handler` | `smart_folder_handler.rs` | SmartFolder CRUD (saved search queries)                                               |
| `tags_handler`         | `tags_handler.rs`         | Tag CRUD, incremental add/remove, batch operations                                    |
| `metadata_handler`     | `metadata_handler.rs`     | Colors (replace-all), rating, notes, format correction, technical metadata upsert     |
| `thumbnail_handler`    | `thumbnail_handler.rs`    | Thumbnail path persistence and invalidation                                           |

### 3.3 New Role of `SqliteAssetLedger`

After Phase 1, the Ledger's `execute_single` method became a pure dispatcher — a `match` with one arm per `LedgerCommand` variant, each delegating immediately to the corresponding handler:

```rust
// Before: 900+ lines of inline SQL
// After: a clean dispatch table
match command {
    LedgerCommand::CreateAsset(payload) =>
        asset_handler::handle_create(tx, payload).await,

    LedgerCommand::DeleteAsset { asset_id, path, physical_delete } =>
        asset_handler::handle_delete_asset(tx, asset_id, path, physical_delete).await,

    LedgerCommand::CreateFolder(payload) =>
        folder_handler::handle_create_folder(tx, payload).await,

    LedgerCommand::UpdateAssetColors(payload) =>
        metadata_handler::handle_update_asset_colors(tx, payload).await,

    // ... one line per command
}
```

---

## 4. Phase 2 — Saga/Outbox Pattern

### 4.1 The Core Insight

SQLite cannot participate in a two-phase commit with the filesystem. True atomicity between "delete from DB" and "delete file from disk" requires a different approach: **record the intent before acting, and verify the result after**.

The Outbox pattern solves this by:
1. Writing the intended operation to an audit log **inside** the same database transaction.
2. Committing the database transaction.
3. Executing the filesystem operation **after** the commit.
4. Updating the audit log status to `COMPLETED` or `FAILED`.

If the process crashes between steps 3 and 4, the operation log row remains in `PENDING` status. A recovery service, running at next startup, picks up these rows and completes or compensates them.

### 4.2 Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                    execute() — Ledger                            │
│                                                                  │
│  1. BEGIN TRANSACTION                                            │
│  │                                                               │
│  ├─► execute_single() ──► asset_handler::handle_delete_asset()  │
│  │       │                                                       │
│  │       ├─ DELETE FROM assets WHERE id = ?                      │
│  │       │                                                       │
│  │       └─ INSERT INTO asset_operations_log                     │
│  │              (status = 'PENDING', physical = true, path = ?) │
│  │                                                               │
│  2. COMMIT TRANSACTION  ◄──────────────────────────────────────  │
│  │   (DB is now consistent — asset removed from index)          │
│  │                                                               │
│  3. POST-COMMIT SAGA                                             │
│  │                                                               │
│  ├─► tokio::fs::remove_file(path)                               │
│  │       │                                                       │
│  │       ├─ Ok  ──► UPDATE log SET status = 'COMPLETED'         │
│  │       ├─ NotFound ► UPDATE log SET status = 'COMPLETED'      │
│  │       └─ Err ──► UPDATE log SET status = 'FAILED'            │
│  │                                                               │
│  4. EMIT DomainEvent::FsPathDeleted                              │
└──────────────────────────────────────────────────────────────────┘
```

### 4.3 Outbox Log Status Machine

```
┌──────────┐   commit succeeds   ┌─────────┐   fs ok   ┌───────────┐
│  (none)  │ ──────────────────► │ PENDING │ ─────────► │ COMPLETED │
└──────────┘                     └─────────┘            └───────────┘
                                      │
                                      │   fs fails
                                      ▼
                                 ┌──────────┐
                                 │  FAILED  │ ◄── Recovery can retry or
                                 └──────────┘     leave for manual action
```

The `asset_operations_log` table schema (relevant columns):

| Column           | Type        | Description                          |
| ---------------- | ----------- | ------------------------------------ |
| `id`             | TEXT (UUID) | Unique operation identifier          |
| `operation_type` | TEXT        | e.g. `DELETE_ASSET`, `CREATE_FOLDER` |
| `asset_id`       | TEXT        | Target entity ID                     |
| `payload`        | JSON        | Full payload for recovery context    |
| `status`         | TEXT        | `PENDING` / `COMPLETED` / `FAILED`   |
| `error_note`     | TEXT?       | Error message if status is `FAILED`  |
| `created_at`     | DATETIME    | When the intent was recorded         |

### 4.4 `SagaRecoveryService` — Startup Recovery

A new `SagaRecoveryService` (`src/infra/database/saga_recovery.rs`) runs **once at application startup**, injected via the `bootstrap/database.rs` composition root.

```
App Startup
    │
    ├─► DbManager::new()          — connection pool + migrations
    ├─► normalize_database_paths() — one-time Unicode cleanup
    └─► SagaRecoveryService::run_recovery()
            │
            └─► SELECT * FROM asset_operations_log WHERE status = 'PENDING'
                    │
                    └─► for each row:
                            process_operation()
                                │
                                ├─ 'DELETE_ASSET' with physical=true
                                │       └─► fs::remove_file() or skip if already gone
                                └─ unknown type → log & skip
```

The recovery service is deliberately **conservative**: it processes operations from oldest to newest, marks failures individually, and never crashes the boot sequence even if individual operations fail.

### 4.5 Code — Key Sections

**`asset_handler.rs` — Outbox registration on physical delete:**

```rust
// 3. Audit Log — Use Outbox pattern (PENDING if physical, COMPLETED otherwise)
let status = if physical_delete { "PENDING" } else { "COMPLETED" };
SqliteAssetLedger::log_operation(
    tx,
    "DELETE_ASSET",
    &resolved_id,
    serde_json::json!({
        "physical": physical_delete,
        "path": path.map(|p| p.to_string_lossy().to_string())
    }),
    status,
    None,
)
.await?;
```

**`ledger.rs` — Post-commit Saga execution:**

```rust
// 2.5 Post-commit Saga Execution (Filesystem Operations)
for (asset, command_item) in &results {
    match command_item {
        LedgerCommand::DeleteAsset {
            physical_delete: true,
            path: Some(path_reference),
            ..
        } => {
            let filesystem_result = tokio::fs::remove_file(path_reference).await;
            match filesystem_result {
                Ok(_) => { /* mark COMPLETED */ }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => { /* mark COMPLETED */ }
                Err(error) => { /* mark FAILED with error_note */ }
            }
        }
        _ => {}
    }
}
```

**`saga_recovery.rs` — Recovery loop:**

```rust
pub async fn run_recovery(&self) -> AppResult<()> {
    let pending_operations = sqlx::query!(
        "SELECT ... FROM asset_operations_log WHERE status = 'PENDING' ORDER BY created_at ASC"
    ).fetch_all(&self.pool).await?;

    for operation in pending_operations {
        match self.process_operation(...).await {
            Ok(_)  => self.mark_completed(&operation.id).await?,
            Err(e) => self.mark_failed(&operation.id, &e.to_string()).await?,
        }
    }
    Ok(())
}
```

---

## 5. Files Changed

| File                                                  | Phase | Change Type | Description                                                      |
| ----------------------------------------------------- | ----- | ----------- | ---------------------------------------------------------------- |
| `src/infra/database/ledger.rs`                        | 1 & 2 | Refactor    | Reduced to dispatcher + Saga executor. ~900 lines deleted.       |
| `src/infra/database/handlers/mod.rs`                  | 1     | New         | Module registry for all handlers.                                |
| `src/infra/database/handlers/asset_handler.rs`        | 1 & 2 | New         | Asset lifecycle — create, batch create, delete (with Outbox).    |
| `src/infra/database/handlers/folder_handler.rs`       | 1     | New         | Folder CRUD — create (UPSERT), cascade remove, recursive rename. |
| `src/infra/database/handlers/smart_folder_handler.rs` | 1     | New         | SmartFolder CRUD.                                                |
| `src/infra/database/handlers/tags_handler.rs`         | 1     | New         | Tag CRUD and batch operations.                                   |
| `src/infra/database/handlers/metadata_handler.rs`     | 1     | New         | Colors, rating, notes, format, technical metadata.               |
| `src/infra/database/handlers/thumbnail_handler.rs`    | 1     | New         | Thumbnail path persistence and invalidation.                     |
| `src/infra/database/saga_recovery.rs`                 | 2     | New         | Startup recovery service for PENDING Sagas.                      |
| `src/infra/database/mod.rs`                           | 1 & 2 | Modified    | Registered `handlers` and `saga_recovery` modules.               |
| `src/bootstrap/database.rs`                           | 2     | Modified    | Injected `SagaRecoveryService::run_recovery()` at startup.       |

---

## 6. Advantages of the New Implementation

### 6.1 Modular Testability

Each handler is a pure async function with a well-defined signature:

```rust
pub async fn handle_create(
    tx: &mut Transaction<'_, Sqlite>,
    payload: CreateAssetPayload,
) -> AppResult<Asset>
```

This makes unit testing straightforward: create an in-memory SQLite DB, begin a transaction, call the handler, assert the result. No Ledger, no EventBus, no mocking required.

### 6.2 Single Responsibility Principle

The `ledger.rs` file now has one job: orchestrate the transaction lifecycle and emit domain events. No business rule should ever live there.

### 6.3 Real Atomicity for Physical Operations

Before this change, the following scenario was possible:

1. `DELETE FROM assets` — DB commit succeeds ✅
2. Process crash (power loss, OS kill) ❌
3. File remains on disk — inconsistent state **forever**

After this change:

1. `DELETE FROM assets` + `INSERT INTO log (status='PENDING')` — commit ✅
2. Process crash ❌
3. On next boot: `SagaRecoveryService` detects `PENDING` row
4. Attempts `fs::remove_file()` — resolves consistency ✅

### 6.4 Audit Trail with Recovery Context

The `asset_operations_log` table now stores the **full JSON payload** of every mutating operation, not just the result. This enables:

- Post-mortem debugging of what happened and when
- Idempotent recovery (replay the same payload)
- Future support for undo operations

### 6.5 Extensibility

Adding support for a new command that touches the filesystem now follows a clear, documented pattern:

1. Write `status = 'PENDING'` in the handler inside the transaction
2. Add a `match` arm in the Ledger's post-commit Saga loop
3. Add a recovery case in `SagaRecoveryService::process_operation()`

### 6.6 Compliance with the Architecture Contract

The `TransactionalAssetLedger` port's guarantee of "atomicity between database updates and filesystem operations" is now real and enforced by code, not just by documentation.

---

## 7. Disadvantages & Known Limitations

### 7.1 Saga Status Update Is Not Itself Atomic

The post-commit step that marks a saga `COMPLETED` or `FAILED` is intentionally **fire-and-forget** (`let _ = ...`). This means:

- If *this* update fails (e.g., connection drop), the row remains `PENDING`.
- On next startup, `SagaRecoveryService` will attempt the filesystem operation again.
- For idempotent operations (like `remove_file`), this is safe.
- For non-idempotent operations (e.g., sending an external HTTP request), this would cause double-execution.

**Mitigation:** All currently implemented Saga operations (`DELETE_ASSET`) are idempotent. The `process_operation` handler treats `NotFound` as success.

### 7.2 No Support for Compensating Transactions

The current implementation only supports **forward recovery** (retry the intent). There is no implementation of compensating transactions (rollback the DB record if the filesystem operation is permanently impossible).

**Example gap:** If a file cannot be deleted due to a `EPERM` error, the Saga marks it `FAILED`. But the database record is already gone. The file remains an orphan on disk with no DB entry to reference it.

**Mitigation:** Acceptable for the current scope. A future sprint can implement a `COMPENSATE` action in the `SagaRecoveryService` that re-inserts the deleted DB record if the physical operation fails after a configured number of retries.

### 7.3 Recovery Service Is Sequential

`SagaRecoveryService::run_recovery()` processes operations one at a time in chronological order. For a library with thousands of pending operations (e.g., bulk delete before a crash), startup recovery will block on each operation serially.

**Mitigation:** Recovery is expected to be rare and the startup window is brief. A future optimization is to process operations in parallel bounded batches using `FuturesUnordered`.

### 7.4 No Saga Coverage for Move and Rename Operations

`RenameFolder` and `UpdateAsset` (path changes) are currently logged as `COMPLETED` immediately, even though they may involve filesystem renames not tracked in the Outbox.

**This is a known gap.** Filesystem renames are typically atomic at the OS level for same-volume operations, lowering the practical risk, but a rigorous implementation should still cover them.

### 7.5 `FAILED` Operations Have No User-Facing Alerting

A saga marked `FAILED` currently only appears in the `tracing` logs. There is no user-visible notification or admin panel to surface unresolved saga failures.

**Mitigation:** Future work. A `get_saga_failures` Tauri query command could expose this information to the frontend dashboard.

### 7.6 Handler Functions Are Not Truly Independent Modules

While the handlers are in separate files, they all depend on `SqliteAssetLedger::log_operation()` and `SqliteAssetLedger::fetch_asset_by_id()` as associated functions. This creates a compile-time coupling: handlers cannot be compiled without the parent `ledger` module. A future service layer could eliminate this by providing these utilities as standalone free functions.

---

## 8. Future Work (Backlog)

| Priority | Item                           | Description                                                                                           |
| -------- | ------------------------------ | ----------------------------------------------------------------------------------------------------- |
| 🔴 High   | Extend Saga coverage           | Add `RenameFolder`, `MoveFile`, `CreateThumbnailFile` to the Outbox pattern                           |
| 🔴 High   | Compensating transactions      | Re-insert DB record if physical operation permanently fails after N retries                           |
| 🟡 Medium | Parallel recovery              | Process pending sagas concurrently in bounded batches using `FuturesUnordered`                        |
| 🟡 Medium | User-facing failure surface    | Expose `FAILED` rows via a `get_saga_failures` Tauri query command                                    |
| 🟡 Medium | Service Layer                  | Extract `log_operation` + `fetch_asset_by_id` from `SqliteAssetLedger` to break compile-time coupling |
| 🟢 Low    | Retry count + backoff          | Track retry attempts in `asset_operations_log` and apply exponential backoff                          |
| 🟢 Low    | Saga coverage for external I/O | Extend the Outbox to cover AI extraction API calls, not just local fs                                 |

---

## 9. Architecture Diagram (Final State)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        delivery/ (Tauri)                            │
│  mutations.rs → calls ledger.execute(LedgerCommand::*)              │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ AppResult<Asset>
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│              infra/database/ledger.rs (SqliteAssetLedger)           │
│                                                                     │
│  1. BEGIN tx                                                        │
│  2. dispatch → execute_single() ──► handlers/*                      │
│  3. COMMIT tx                                                       │
│  4. Post-commit Saga (fs I/O + log update)                          │
│  5. emit_event_for_command() ──► AppEventBus                        │
└──────────────────────┬──────────────────────────────────────────────┘
                       │ delegates SQL to
                       ▼
┌──────────────────────────────────────┐
│      infra/database/handlers/        │
│                                      │
│  asset_handler.rs                    │
│  folder_handler.rs                   │
│  smart_folder_handler.rs             │
│  tags_handler.rs                     │
│  metadata_handler.rs                 │
│  thumbnail_handler.rs                │
└──────────────────────────────────────┘

On App Startup:
┌───────────────────────────────────────────────────────────┐
│  bootstrap/database.rs                                    │
│    └─► SagaRecoveryService                                │
│           └─► run_recovery()                              │
│                 └─► asset_operations_log WHERE PENDING    │
└───────────────────────────────────────────────────────────┘
```

---

## 10. Validation

The full implementation was validated with `cargo check`, yielding:

```
Finished `dev` profile [unoptimized + debuginfo] target(s)
0 errors, 0 warnings
```

Integration was also validated by running `npm run tauri dev` and confirming:

- Application boots cleanly:
  ```
  SagaRecovery: No pending operations found. System is clean.
  ```
- Indexing, thumbnail generation, and color extraction operate correctly
- Move detection (V1 signature-based recovery) continues to work:
  ```
  Ledger: MOVE DETECTED (V1 recovery). Updating asset 47349843-...
    from '.../800_0.0.DNG' to '.../8001_0.0.DNG'
  ```
