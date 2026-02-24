# Table Component Refactoring

## Context
The `Table.tsx` component needs to be thoroughly refactored to conform to Solid.js and TypeScript standards. Currently, it stands at ~450 lines, violating max line limits and the Single Responsibility Principle, and has multiple `any` type casts that bypass strong generic typing. The UI and logic are highly interwoven, particularly the virtualization and keyboard navigation handlers.

## Objectives
1. **Directory Structure**: [DONE] Transitioned from a single `Table.tsx` file to a modular `src/components/ui/Table/` folder with clear separations.
2. **Type Safety**: [DONE] Established strong `T extends Record<string, unknown>` boundaries.
3. **Externalized Logic**: [DONE] Extracted complex logic into individual hooks (`useTableVirtualization.ts`, `useTableNavigation.ts`).
4. **Integration with `src/core/input`**: [DONE] Refactored internal custom shortcut mapping to use the standard core input primitives correctly.
5. **Architectural Purity**: [DONE] Removed visual separation comments and decomposition into functional sub-components.

## Plan Outline
### Phase 1: Structuring & Types
- [x] Create `src/components/ui/Table/types.ts` defining rigorous types instead of inline definitions.
- [x] Move `table.css` to `src/components/ui/Table/table.css`.

### Phase 2: Logic Extraction (Hooks)
- [x] Create `useTableVirtualization.ts` for calculations related to scroll ranges and visible viewport index boundaries.
- [x] Create `useTableNavigation.ts` mapped with `src/core/input` to manage viewport navigation, space selection and enter triggers using events instead of raw DOM keyboard logic.

### Phase 3: Presentation UI Components
- [x] Create `TableHeader.tsx` for sorting behavior and column flex calculations.
- [x] Create `TableRow.tsx` isolating item-rendering.
- [x] Create `EmptyState.tsx` isolating fallback logic to render empty inbox components from `lucide-solid`.

### Phase 4: Assembly
- [x] Construct the slimline `Table.tsx` orchestrator to wire the hooks to the subcomponents.
- [x] Reconfigure exports over `src/components/ui/Table/index.tsx`.
- [x] Remove `src/components/ui/Table.tsx`.
- [x] Verify exports within `src/components/ui/index.ts` and ensure clean lint check.
