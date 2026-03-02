# Core Architecture Guidelines

This document defines the patterns, constraints, and best practices for the Mundam core layer, ensuring long-term maintainability, type safety, and decoupling.

---

## 🛡️ 1. Action Pattern

All state mutations and side effects must occur through standardized **Actions**. This ensures a unidirectional data flow and predictable state transitions.

### ActionResult and BaseError
Every action must return a `Promise<ActionResult<TData, TError>>`. This pattern avoids throwing exceptions for expected business errors and forces consumers to handle both success and failure.

```typescript
import { ActionResult } from '../types/actions';

export async function processMetadata(id: string): Promise<ActionResult<Metadata>> {
    try {
        // Validation and logic...
        return { success: true, data: result };
    } catch (error) {
        return { 
            success: false, 
            error: { code: 'INTERNAL_ERROR', message: 'Failed to process' } 
        };
    }
}
```

### Error Codes
System-wide error segments are defined in `ErrorCode`. Use specific codes so the UI can react appropriately (e.g., showing a specific error message for 404 vs 409).

### Batch Actions
When an operation affects multiple items (e.g., tagging 100 images), use **atomic batch commands**. Avoid looping through single-item actions in the UI. Batch actions ensure database consistency and performance by utilizing a single transaction or specialized backend commands (e.g., `metadataActions.updateAssetsTags`).

---

## 🔍 2. Zod Integration & Validation

We use Zod for all "at-the-edge" validations—data coming from the UI, LocalStorage, or Tauri backend.

### Schema Ownership
- Each store/domain has a `schemas.ts` file (e.g., `src/core/store/filter/schemas.ts`).
- **Suffix Rule:** All schemas must end with `Schema` (e.g., `CriterionSchema`).
- **Derivation Rule:** Always derive TypeScript types from schemas using `z.infer`.

### Validation Strategy
- **Fail Fast:** Actions must validate their input payloads as the first step using `Schema.safeParse()`.
- **Transformation:** Use Zod's `.transform()` to normalize data (e.g., trimming strings) during validation.

---

## 📡 3. Domain Events & Event Bus

The `eventBus` is the primary mechanism for decoupling stores and notifying the UI about background changes without direct imports.

### Decoupling Principle
Stores should not import or call each other directly to avoid circular dependencies. Instead:
1. **Emit:** Store A performs an action and calls `eventBus.emit('domain:changed', payload)`.
2. **Listen:** Store B or a UI Component subscribes via `eventBus.on('domain:changed', (data) => { ... })`.

### Typesafety
All events must be registered in the `DomainEvents` interface within `src/core/utils/eventBus.ts` to ensure type-safe payloads.

### LifecycleManager & Backend Communication
For events that are emitted by the Rust Backend (Tauri IPC), **never** call `listen` directly inside scattered UI components without cleanup.
- **Centralized Handlers**: Use `LifecycleManager.ts` to connect critical App events and gracefully unsubscribe them `onCleanup()`.
- **Telemetry Bridge**: The `LifecycleManager` also serves as the IPC bridge to send critical frontend `tracing` logs (e.g., render times, UI crashes) to the Rust backend, ensuring a single unified timeline.

---

## 📦 4. Store & State Management

Mundam uses **Solid.js Stores** for reactive state. As domains grow, it is strictly forbidden to create "God-Files" (files exceeding 300-400 lines). Stores must be split into modular subunits to guarantee separation of concerns.

### Modular Store Anatomy
When organizing a store (e.g., `src/core/store/metadata/`), follow this directory structure:
1. **State Definition (`*State.ts`)**: Contains the `createStore` initialization, interfaces, and the private setter (`setInternalState`). Must not contain business logic.
2. **Domain Actions (`*Actions.ts`)**: Split actions logically into sub-domains (e.g., `tagActions.ts`, `locationActions.ts`, `searchActions.ts`) to keep files small and focused.
3. **Schemas & Constants**: Keep validation rules and static lists isolated.
4. **Proxy Index (`index.ts`)**: Aggregate and re-export the state and all actions to maintain a unified public API for consumer hooks without introducing breaking changes.

### Resolving Circular Dependencies
Splitting massive stores can introduce cyclic imports. To solve this safely:
- Avoid importing `storeA/actions.ts` directly into `storeB/actions.ts` at the top level.
- Use **late initialization/Dependency Injection** (e.g., exporting an `initRefs` function in the action file that the `index.ts` calls to link dependencies) or rely on the `eventBus` to orchestrate effects across boundaries.

### Design Principles
1. **Atomic Mutations:** Actions should modify the store in discrete, logical steps.
2. **Reactivity Control:** Use `untrack()` inside actions to read current state without creating unnecessary dependencies.
3. **Derived State:** Prefer `createMemo` or simple getters for computed values instead of storing redundant data in the state.
4. **Encapsulation:** Export only the State (via accessor) and the Actions. Never export `setStore`.

---

## 🪝 5. Public API (Hooks Layer)

The `src/core/hooks` directory acts as the **Public API facade** for the UI.

- Components should only import from hooks (e.g., `useMetadata`, `useLibrary`).
- Hooks encapsulate multiple store accessors and actions into a cohesive interface.
- **Rule:** If a component needs more than 3 different actions from a store, consider adding a simplified facade method to the hook.

---

## 🧱 6. Layer Constraints (Guardrails)

To prevent architectural decay, the following constraints are strictly enforced:

1.  **UI Isolation:** UI components (`src/components/`) must never call `invoke` or `tauriService` directly. They must use Hooks -> Actions.
2.  **State Protection:** No code outside of the action layer should modify a store's state.
3.  **Cross-Store Communication:** Use the **Event Bus** for communication between different store domains.
4.  **No UI in Core:** Stores and Actions must remain "pure" from UI concerns. Never import `toast`, `modal`, or DOM-specific objects (like `window` or `document`) into actions unless explicitly abstracted.
5.  **Service Abstraction:** Use the `tauriService` wrapper for all backend calls to provide a single place for error logging and type mapping.

---

## 🌐 7. Backend (Tauri) Abstraction

The `tauriService` (`src/core/tauri/services.ts`) acts as our Anti-Corruption Layer (ACL) between the frontend and Rust.

- Standardizes argument names (e.g., converting camelCase to snake_case for Rust).
- Centralizes error logging for bridge failures and forwards frontend performance traces via IPC to the Rust `tracing` context.
- Provides a mockable interface for potential unit testing of the business logic.
