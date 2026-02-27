# Sprint 4 Implementation Plan: Search & Inspector

This plan outlines the refactoring of the Advanced Search and Inspector domains, focusing on atomic actions, validation, and domain isolation.

## 🏗️ 1. Infrastructure & Patterns

### 📡 1.1 Domain Event Bus
To solve circular dependencies and allow UI isolation (as per Guideline 4.4), we will implement a lightweight, typesafe Event Bus.
- **File:** `src/core/utils/eventBus.ts` [x]
- **Purpose:** Allow Stores to emit events (e.g., `metadata:changed`) that UI components or other stores can listen to without direct coupling.

### 💾 1.2 Metadata Cache System
A persistent cache for EXIF and technical information.
- **File:** `src/core/store/metadata/cache.ts` [x]
- **Implementation:** `localStorage` with TTL (Time To Live) and size management.
- **Integration:** Actions in `metadataStore` will check this cache before calling `tauriService`.

## 🔍 2. Advanced Search (Interaction 3)

### 🛡️ 2.1 Validation & Normalization
- **File:** `src/core/store/filter/schemas.ts` [x]
- **Task:** Implement Zod schemas for `SearchCriterion` and `SearchGroup`. 
- **Note:** Keep it simple for now (top-level AND/OR) as per user request, but use recursive types to allow future expansion.

### ⚙️ 2.2 Filter Store Refactoring
- **Move Logic:** Relocate criteria processing (ID generation, display value computation handles) to `filterActions`. [x]
- **Atomic Actions:** Add `filterActions.addCriterion`, `filterActions.removeCriterion`, `filterActions.updateCriterion`. [x]
- **Validation:** Actions will return `ActionResult` with validation details. [x]

### 🧹 2.3 Hook Cleanup
- **File:** `src/components/features/search/useAdvancedSearch.ts` [x]
- **Task:** Reduce to purely UI state (current builder values). Delegate all structure and validation to `filterActions`.

## 📦 3. Inspector & Metadata (Interaction 4)

### 🏗️ 3.1 Batch Actions
- **File:** `src/core/store/metadataStore.ts` [x]
- **Actions:**
    - `updateAssetsTags(assetIds, tagIds, mode: 'merge' | 'replace' | 'remove')` [x]
    - `updateAssetsMetadata(assetIds, metadata)` [x]
- **Atomic Execution:** Perform batch updates and emit domain events for UI refresh. [x]

### 🧱 3.2 Inspector UI Isolation
- **Task:** Refactor `InspectorTags.tsx` and related components to call Actions instead of `tagService` or `tauriService`.
- **Event Handling:** Components will react to `metadataStore` state or generic events.

## ✅ Verification Plan

### Automated Tests
- [ ] **Unit Tests:** Validate `SearchGroup` schema with valid and invalid payloads.
- [ ] **Action Tests:** Verify `updateAssetsTags` correctly handles all 3 modes (Merge/Replace/Remove).
- [ ] **Cache Tests:** Verify `localStorage` persistence and eviction logic.

### Manual Verification
- [x] Advanced search builder works correctly with validation feedback.
- [x] Batch editing tags in the Inspector updates the grid and sidebar stats.
- [x] Smart folder creation/editing persists correctly.
