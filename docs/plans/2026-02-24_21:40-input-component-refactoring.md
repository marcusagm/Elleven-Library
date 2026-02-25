# Plan: Input Component Refactoring

Refactor the `Input` UI component to improve code quality, maintainability, and adherence to project guidelines (SOLID, documentation, naming conventions).

## Goals
- [x] Modularize the component into a dedicated directory (`src/components/ui/Input/`).
- [x] Eliminate all `any` types, specifically in event handlers.
- [x] Improve naming conventions (no abbreviations like `others` or `local`).
- [x] Enhance documentation with complete TSDoc and clear examples.
- [x] Isolate keyboard/focus logic into a custom hook.
- [x] Ensure full compatibility with the existing `core/input` system.
- [x] Update all project imports to the new location.

## Architecture
- **Location**: `src/components/ui/Input/`
- **Files**:
    - `index.tsx`: Public API export.
    - `Input.tsx`: Main component logic and structure.
    - `InputLabel.tsx`: Sub-component for the label.
    - `InputIcon.tsx`: Sub-component for icons.
    - `InputErrorMessage.tsx`: Sub-component for error messages.
    - `types.ts`: Interface definitions.
    - `useInputEvents.ts`: Logic hook for focus, blur, and keyboard events.
    - `input.css`: Scoped component styles.

## Proposed Changes

### 1. Type Definitions (`types.ts`)
- Use specialized event handler types from SolidJS to avoid `as any`.
- Define clear interfaces for props and variants.

### 2. Logic Hook (`useInputEvents.ts`)
- Manage `PushScope` and `PopScope` for the `editing` scope.
- Handle `KeyDown` propagation logic to prevent input keys from triggering global shortcuts.

### 3. Component Structure (`Input.tsx`)
- Implement the "No Single/Two-letter names" rule.
- Use sub-components to reduce file complexity.
- Maintain existing props signature for backward compatibility.

## Verification Plan
- [x] Run `npm run lint` to check for style issues.
- [x] Verify that the `editing` scope is correctly pushed/popped during focus/blur.
- [x] Ensure keyboard navigation (Enter, Arrows) works within the input without bubbling to global shortcuts.
- [x] Verify the component renders correctly with icons, labels, and error messages.
