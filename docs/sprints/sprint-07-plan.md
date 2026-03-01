# Sprint 7 Plan: UI Component Excellence & God-File Splitting

**Objective:** Systematic removal of `eslint-disable` and refactoring of massive store files into modular subunits, achieving 100% architectural compliance.

---

## 🏗️ 1. Interaction Breakdown

### Modular Store Refactoring (God-File Disposal)
- **[x] Splitting `libraryStore.ts` (>400 lines):**
  - [x] Extract `libraryActions.ts`.
  - [x] Extract `libraryState.ts` and `schemas.ts`.
- **[x] Splitting `metadataStore.ts` (>500 lines):**
  - [x] Extract `tagActions.ts`.
  - [x] Extract `searchActions.ts`.
  - [x] Extract `locationActions.ts`.
- **[x] Splitting `filter/index.ts` (>400 lines):**
  - [x] Extract `filterActions.ts` and `filterState.ts`.

### UI Quality & Reactivity
- **[x] Component Cleanup:**
  - [x] Fix `StarRating.tsx` (reactive props).
  - [x] Fix `GlyphsTab.tsx` (CSS units).
  - [x] Fix `EmptyState.tsx` (reactive icon).
  - [x] Fix `AudioPlayerContext.tsx` (reactive playerProps).
- **[x] `KeyboardShortcutsPanel` Refactoring:**
  - [x] Reduce complexity (currently 12).
  - [x] Wrap event handlers in functions for reactivity.

### Directive Cleanup
- **[x] `assetDirective.ts` & `assetDragSource.ts`:**
  - [x] Remove `eslint-disable` by refactoring to pure SolidJS patterns.

---

## 📦 2. Technical Tasks & Files

### Files to Modify/Create
- `src/core/store/library/` (New folder)
- `src/core/store/metadata/` (New folder)
- `src/core/store/filter/` (Expansion)
- `src/components/features/settings/KeyboardShortcutsPanel.tsx`
- `src/core/dnd/assetDirective.ts`
- `src/core/dnd/assetDragSource.ts`

---

## 📋 3. Verification & DoD

1. [x] **Zero `eslint-disable`:** All massive files split to <300 lines.
2. [x] **Reactivity Correctness:** All warnings about reactive variables fixed.
3. [x] **Final Build:** App builds and runs without any lint warnings.
4. [x] **Architecture Check:** All components follow the feature-colocation and hook-facade rules.

---

## 🛑 Socratic Gate Questions

1. **Splitting Stores:**
   - How to manage shared context? Use a proxying `index.ts` in the store folder to maintain API compatibility for hooks.
2. **Complexity Reduction:**
   - Strategy? Break massive `switch` or `if/else` chains into early-return functions or strategy maps.
