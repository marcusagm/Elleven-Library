# Sprint 5 Plan: ItemView & Viewport Engine

**Objective:** Finalize refactoring of high-performance components, ensuring the rendering engine and immersive viewer follow architectural safety standards and performance benchmarks.

---

## 🏗️ 1. Interaction Breakdown

### Interação 5: ItemView e Renderers
Focus: Decoupling navigation and viewport control from UI state.

- **[x] Decouple Navigation:**
  - [x] Implement `viewportActions.navigateToAsset` in `viewportStore.ts`.
  - [x] Update `ItemView` and its sub-components to use `useViewport` hook for navigation.
  - [x] Ensure navigation state is synchronized across the app (selection vs. viewport).
- **[x] Viewport Controls (Zoom/Fit):**
  - [x] Move zoom, fit-to-screen, and panning state/logic to `viewportActions`.
  - [x] Implement `zoomIn`, `zoomOut`, `toggleFit`, `resetZoom`.
- **[x] Standardize Renderers:**
  - [x] Ensure `ImageViewer`, `FontRenderer`, etc., use strict typed payloads for config.
  - [x] Refactor media loading to use `ActionResult`.

### Interação 9: Viewport Engine (Workers)
Focus: Thread safety, validation, and performance optimization.

- **[x] Worker Security (Main-Worker):**
  - [x] Define Zod Schemas for `LayoutWorker` messages (in/out).
  - [x] Implement validation in `layout.worker.ts`.
- **[x] ViewportController Refactoring:**
  - [x] Transform `ViewportController` into a **Domain Service**.
  - [x] Move reactive signals to `viewportStore`.
- **[x] System Scheduler:**
  - [x] Implement `src/core/utils/scheduler.ts` as a central `requestAnimationFrame` manager.
  - [x] Replace ad-hoc `rAF` calls with the central scheduler.

---

## 📦 2. Technical Tasks & Files

### Files to Modify/Create
- `src/core/store/viewportStore.ts`: Central state for navigation and viewport properties.
- `src/core/hooks/useViewport.ts`: Public API for viewport actions.
- `src/core/viewport/ViewportController.ts`: Logic refactoring.
- `src/core/viewport/layout.worker.ts`: Message validation.
- `src/core/utils/scheduler.ts`: Performance utility.
- `src/components/features/itemview/`: UI refactoring to use new hooks.

---

## 📋 3. Verification & DoD

1. [ ] **Navigation Isolation:** Individual navigation logic removed from UI, handled by Actions.
2. [ ] **Worker Validation:** All worker messages are schema-validated.
3. [ ] **Performance Check:** Maintain stable 60FPS during viewport operations with large libraries.
4. [ ] **Memory Safety:** Verify workers are properly terminated on component unmount.
5. [ ] **Clean Code:** 0 complexity lint errors in viewport logic.

---

## 🛑 Socratic Gate Questions

1. **Navigation Context:**
   - **Answer:** It must synchronize and update the global selection in the library. This ensures that even if ItemView is open full-screen, closing it will show the library scrolled to the currently viewed item, improving UX.

2. **Worker Serialization:**
   - **Answer:** Avoid CPU overhead where possible. High-frequency updates (like scroll offsets) should use lightweight TypeGuards for performance, while initialization and configuration commands should be fully validated with Zod. Performance and safety combined.

3. **Zoom Logic:**
   - **Answer:** Zoom will be purely 2D (Canvas/CSS transforms). The 3D viewer is completely separate and uses its own rendering system (`src/components/features/itemview/renderers/model`).
