# Plan: Refactor Accordion Component to Compound Pattern

Refactor the `Accordion` component into a compound component architecture to improve modularity, maintainability, and adherence to Mundam's coding excellence standards.

## 1. Preparation
- [x] Create directory `src/components/ui/Accordion/`.
- [x] Research specific usages in `Inspector` components to ensure a smooth transition.

## 2. Implementation (Phase 1: Structure)
- [x] **types.ts**: Define all interfaces with descriptive names.
- [x] **useAccordion.ts**: Implement the context and `useAccordion`/`useAccordionItem` hooks.
- [x] **Accordion.tsx**: Root component managing the controlled/uncontrolled state.
- [x] **AccordionItem.tsx**: Component for individual items, managing its own ID and providing item-level context if needed.
- [x] **AccordionTrigger.tsx**: The clickable header part of the accordion. Includes `AccordionHeader` and `AccordionChevron`.
- [x] **AccordionContent.tsx**: The collapsible content part.
- [x] **index.tsx**: Export all components.

## 3. Implementation (Phase 2: Assets & Cleanup)
- [x] Move `src/components/ui/accordion.css` to `src/components/ui/Accordion/accordion.css`.
- [x] Update CSS selectors if the HTML structure changed significantly.
- [x] Verify TSDoc documentation is complete and in English.
- [x] Ensure no abbreviated variable names (e.g., replace `v`, `i` with `value`, `index`).

## 4. Integration & Testing
- [x] Update `src/components/ui/index.ts` to export from the new folder.
- [x] Update all dependencies (Inspector components):
    - [x] `InspectorTags.tsx`
    - [x] `CommonMetadata.tsx`
    - [x] `AdvancedMetadata.tsx`
    - [x] `ImageMetadata.tsx`
    - [x] `FontInspector.tsx`
    - [x] `ModelInspector.tsx`
    - [x] `MultiInspector.tsx`
- [x] Run a quick manual check or linting to ensure everything is correct.
- [x] Delete legacy `src/components/ui/Accordion.tsx`.

## 5. Final Verification
- [ ] `son kontrolleri yap` / Final audit.
