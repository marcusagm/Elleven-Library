# 🏗️ Refactoring Plan: Form UI Components (Batch 1)

**Date:** 2026-02-26  
**Status:** ✅ Completed  
**Priority:** High  

## 🎯 Objective

Refactor core form components (`Checkbox`, `RadioGroup`, `Select`) to a modular folder-based architecture. Implement the Compound Component Pattern for `Select` and `RadioGroup` to increase flexibility and maintainability, while ensuring strict adherence to project naming and documentation standards.

---

## 🧱 Architectural Transformations

### 1. 📂 Folder-Based Modularity
Each component will move from a single file to a dedicated directory:
- `Checkbox/`: Simple modularization.
- `RadioGroup/`: Compound component cleanup.
- `Select/`: Major refactoring into a full Compound Component suite.

### 2. 🧩 Compound Component Pattern
- **RadioGroup**: Separate `Root` and `Item` into their own files, sharing state via `context.tsx`.
- **Select**: Transform the monolithic `Select.tsx` into:
    - `Select.Root`: Logic & State Context.
    - `Select.Trigger`: Interactive button.
    - `Select.Value`: Selected label display.
    - `Select.Content`: Portal-based dropdown list.
    - `Select.Item`: Selectable option.
    - `Select.Search`: Filter input.

### 3. 🧼 Code Quality & Clean Code
- **Naming**: `Props` → `Properties`, `local` → `localProperties`, `others` → `remainingProperties`.
- **No Abbreviations**: Ensure every variable describes its responsibility exactly (e.g., `itemValue` instead of `val`).
- **Documentation**: TSDoc `@module` headers, clear parameter descriptions, and `@example` blocks for every exported item.

---

## 🛠️ Step-by-Step Implementation

### Phase 1: Checkbox Refactoring
- [x] Create `src/components/ui/Checkbox/` directory.
- [x] Define `types.ts` with `CheckboxSize` and `CheckboxProperties`.
- [x] Implement `Checkbox.tsx` with cleaned naming.
- [x] Migrate `checkbox.css`.
- [x] Create `index.ts` with TSDoc module description.

### Phase 2: RadioGroup Refactoring
- [x] Create `src/components/ui/RadioGroup/` directory.
- [x] Define `types.ts` for all RadioGroup-related properties.
- [x] Implement `context.tsx` for state sharing.
- [x] Implement `Root.tsx` and `Item.tsx`.
- [x] Migrate `radio-group.css`.
- [x] Create `index.ts` exporting compound parts.

### Phase 3: Select (Pro) Refactoring
- [x] Create `src/components/ui/Select/` directory.
- [x] Define `types.ts` with expansive options for the compound pattern.
- [x] Implement `context.tsx` to handle open state, selection, and highlighted items.
- [x] Implement `Root.tsx`, `Trigger.tsx`, `Value.tsx`, `Content.tsx`, `Item.tsx`, and `Search.tsx`.
- [x] Migrate `select.css`.
- [x] Create `index.ts` exporting compound parts + a High-level `Select` component for backward compatibility.

### Phase 4: Final Integration
- [x] Update `src/components/ui/index.ts` exports.
- [x] Remove old files: `Checkbox.tsx`, `RadioGroup.tsx`, `Select.tsx` and their CSS files.
- [x] Update usages in `DesignSystemGuide.tsx`.
- [x] Run lint and verify project compilation.

---

## ✅ Verification Criteria
- [x] `Select` works both as a High-level component and via Compound parts.
- [x] All types end in `Properties`.
- [x] Zero abbreviations in variables (`local`, `props`, `attr`, `val`, `i`, etc., are BANNED).
- [x] All exported items have comprehensive TSDoc documentation.
- [x] `npm run build` or `npm run dev` completes without errors.
