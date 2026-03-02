# Sprint 6 Plan: Core Engine & Quality Refactoring

**Objective:** Finalize High-Performance Viewport Engine and begin systematic cleanup of architectural debt (lint, `any`, `eslint-disable`) in the core layer.

---

## 🏗️ 1. Interaction Breakdown

### Interaction 9: Viewport Engine (Conclusion)
- **[ ] System Scheduler Integration:**
  - [x] Implement `src/core/utils/scheduler.ts` (RAF/Throttling manager).
  - [x] Refactor `ViewportController.ts` to use `scheduler.ts` instead of direct `rAF`.
  - [x] Standardize `lastReportedWidth` and resize logic in `VirtualGridView.tsx` and `VirtualMasonry.tsx` using the scheduler.

### Architectural Excellence (Batch 1)
- **[x] Core Type Safety:**
  - [x] Eliminate 13 `any` usages across `eventBus.ts`, `createKeyState.ts`, `useFilters.ts`, etc.
- **[x] Core Lint Cleanup:**
  - [x] Fix complexity/max-lines in `src/core/input/dispatcher.ts`.
  - [x] Fix complexity/max-lines in `src/core/input/normalizer.ts` (Split into sub-utils).
  - [x] Fix complexity in `src/core/input/providers/GestureProvider.ts`.
  - [x] Fix complexity/max-lines in `src/core/input/store/shortcutStore.ts` (Split defaults).

---

## 📦 2. Technical Tasks & Files

### Files to Modify/Create
- `src/core/utils/scheduler.ts`
- `src/core/viewport/ViewportController.ts`
- `src/core/utils/eventBus.ts`
- `src/core/input/dispatcher.ts`
- `src/core/input/normalizer.ts`
- `src/core/input/store/shortcutStore.ts`
- `src/core/store/metadataStore.ts`

---

## 📋 3. Verification & DoD

1. [x] **Zero `any` in core utils:** EventBus and keyState are strictly typed.
2. [x] **Complexity < 10:** All touched functions meet the complexity limit.
3. [x] **Running App:** Viewport rendering remains fluid and accurate.
4. [x] **Build Check:** `npm run lint` shows 0 errors for core files.

---

## 🛑 Socratic Gate Questions

1. **Scheduler vs rAF:**
   - Why centralize it? To prevent frame-drop competition between smooth-scrolling and UI animations.
2. **Splitting Large Files:**
   - How to split `shortcutStore.ts` without breaking reactivity? Move actions to separate files and export them, or use partial stores with internal bridge.
