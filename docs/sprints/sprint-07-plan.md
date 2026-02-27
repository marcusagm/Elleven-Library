# Sprint 7 Plan: UI Component Excellence & God-File Splitting

**Objective:** Systematic removal of `eslint-disable` and refactoring of massive store files into modular subunits, achieving 100% architectural compliance.

---

## 🏗️ 1. Interaction Breakdown

### Modular Store Refactoring (God-File Disposal)
- **[ ] Splitting `libraryStore.ts` (>400 lines):**
  - [ ] Extract `libraryActions.ts`.
  - [ ] Extract `libraryState.ts` and `schemas.ts`.
- **[ ] Splitting `metadataStore.ts` (>500 lines):**
  - [ ] Extract `tagActions.ts`.
  - [ ] Extract `searchActions.ts`.
  - [ ] Extract `locationActions.ts`.
- **[ ] Splitting `filter/index.ts` (>400 lines):**
  - [ ] Extract `filterActions.ts` and `filterState.ts`.

### UI Quality & Reactivity
- **[ ] Component Cleanup:**
  - [ ] Fix `StarRating.tsx` (reactive props).
  - [ ] Fix `GlyphsTab.tsx` (CSS units).
  - [ ] Fix `EmptyState.tsx` (reactive icon).
  - [ ] Fix `AudioPlayerContext.tsx` (reactive playerProps).
- **[ ] `KeyboardShortcutsPanel` Refactoring:**
  - [ ] Reduce complexity (currently 12).
  - [ ] Wrap event handlers in functions for reactivity.

### Directive Cleanup
- **[ ] `assetDirective.ts` & `assetDragSource.ts`:**
  - [ ] Remove `eslint-disable` by refactoring to pure SolidJS patterns.

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

1. [ ] **Zero `eslint-disable`:** All massive files split to <300 lines.
2. [ ] **Reactivity Correctness:** All warnings about reactive variables fixed.
3. [ ] **Final Build:** App builds and runs without any lint warnings.
4. [ ] **Architecture Check:** All components follow the feature-colocation and hook-facade rules.

---

## 🛑 Socratic Gate Questions

1. **Splitting Stores:**
   - How to manage shared context? Use a proxying `index.ts` in the store folder to maintain API compatibility for hooks.
2. **Complexity Reduction:**
   - Strategy? Break massive `switch` or `if/else` chains into early-return functions or strategy maps.
