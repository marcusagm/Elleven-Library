# 🏗️ Refactoring Plan: Simple UI Components

**Date:** 2026-02-26  
**Status:** ✅ Completed  
**Priority:** High  

## 🎯 Objective

Refactor foundational UI components (`Separator`, `Loader`, `CountBadge`, `Badge`, `Alert`) to use a modular folder-based architecture, following the Atomic / Compound Component Pattern where applicable, and adhering to strict naming and documentation standards.

---

## 🧱 Architectural Transformations

### 1. 📂 Module Structure Upgrade
Move all components and their respective `.css` files into dedicated directories within `src/components/ui/`.
- `Alert/` ✅
- `Badge/` ✅
- `CountBadge/` ✅
- `Loader/` ✅
- `Separator/` ✅

### 2. ⚡ Alert: Compound Pattern
Refactor the current multi-export file into a cohesive compound component structure.
- `Alert.Root`: Main container and status handling. ✅
- `Alert.Title`: Accessible heading. ✅
- `Alert.Description`: Body content. ✅

### 3. 🏷️ CountBadge: Tooltip Integration
Eliminate the custom `Portal` and `getBoundingClientRect` logic.
- Integrate the existing `Tooltip` component to handle the exact count display on hover. ✅
- Improves code reuse and consistency across the design system. ✅

### 4. 🧼 Code Quality & Guidelines
- **Naming**: `Props` → `Properties`, `props` → `properties`, `local` → `localProperties`. ✅
- **No Abbreviations**: Ensure every variable name is explicit (e.g., `isDisabled`). ✅
- **Documentation**: Comprehensive TSDoc for every component and property, including `@module` headers and `@example` code blocks. ✅

---

## 🛠️ Step-by-Step Implementation

### Phase 1: Alert Component Refactoring
- [x] Create `src/components/ui/Alert/` directory.
- [x] Define `types.ts` with `AlertVariant` and `AlertProperties`.
- [x] Implement `Root.tsx`, `Title.tsx`, and `Description.tsx`.
- [x] Migrate `alert.css` and update class names if necessary.
- [x] Create `index.ts` for public exports.

### Phase 2: Badge Component Refactoring
- [x] Create `src/components/ui/Badge/` directory.
- [x] Define `types.ts` with `BadgeVariant`, `BadgeSize`, and `BadgeProperties`.
- [x] Implement `Badge.tsx` with clean naming.
- [x] Migrate `badge.css`.
- [x] Create `index.ts`.

### Phase 3: CountBadge Component Refactoring
- [x] Create `src/components/ui/CountBadge/` directory.
- [x] Define `types.ts` with `CountBadgeVariant` and `CountBadgeProperties`.
- [x] Refactor `CountBadge.tsx` to use the `Tooltip` component.
- [x] Migrate `count-badge.css`.
- [x] Create `index.ts`.

### Phase 4: Loader Component Refactoring
- [x] Create `src/components/ui/Loader/` directory.
- [x] Define `types.ts` with `LoaderProperties`.
- [x] Implement `Loader.tsx` with optimized structure.
- [x] Migrate `loader.css`.
- [x] Create `index.ts`.

### Phase 5: Separator Component Refactoring
- [x] Create `src/components/ui/Separator/` directory.
- [x] Define `types.ts` with `SeparatorOrientation` and `SeparatorProperties`.
- [x] Implement `Separator.tsx`.
- [x] Migrate `separator.css`.
- [x] Create `index.ts`.

### Phase 6: Final Integration & Cleanup
- [x] Update `src/components/ui/index.ts` exports.
- [x] Remove old files: `Alert.tsx`, `Badge.tsx`, `CountBadge.tsx`, `Loader.tsx`, `Separator.tsx` and their corresponding `.css` files.
- [x] Run lint and verify project compilation.

---

## ✅ Verification Criteria
- [x] All components follow the folder structure: `index.ts`, `types.ts`, `[Component].tsx`, `[component-name].css`.
- [x] `CountBadge` hover behavior uses the shared `Tooltip` component.
- [x] `Alert` can be used as `<Alert.Title>` and `<Alert.Description>`.
- [x] No variable name abbreviations (no `i`, `props`, `local`, `attr`, etc.).
- [x] TSDoc is present for everything.
