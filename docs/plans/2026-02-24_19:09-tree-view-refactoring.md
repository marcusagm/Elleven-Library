# 🌲 Plan: TreeView Component Refactoring

This plan outlines the refactoring of the `TreeView` component to improve code quality, maintainability, and alignment with project guidelines (SOLID, Documentation, and Input System).

## 🎯 Objectives
- **Generification**: Decouple `TreeView` from specific domain types (Tag, Image, Folder).
- **Modularization**: Break down the ~570-line "god file" into smaller, single-responsibility modules.
- **Type Safety**: Eliminate all `any` usages, especially in DnD logic.
- **Input System Integration**: Use `src/core/input` (Shortcut system) for keyboard navigation.
- **Naming & Docs**: Enforce descriptive naming (no abbreviations) and complete TSDoc.

## 🏗️ Architecture

### Folder Structure
```
src/components/ui/TreeView/
├── index.ts                 # Public API (TreeView, TreeNode, etc.)
├── types.ts                 # Shared interfaces and generic types
├── TreeView.tsx             # Root orchestrator component
├── TreeViewItem.tsx         # Item orchestrator & Row decomposition
├── tree-view.css            # Styles
├── components/              # Atomic UI components
│   ├── TreeViewToggle.tsx
│   ├── TreeViewIcon.tsx
│   ├── TreeViewLabel.tsx
│   ├── TreeViewBadge.tsx
│   └── TreeViewInput.tsx
└── hooks/                   # Logic encapsulation
    ├── useTreeNavigation.ts # Keyboard Shortcuts
    └── useTreeDragDrop.ts   # DnD Logic & Validation
```

## 📋 Task Breakdown

### Phase 1: Infrastructure & Types
- [x] Create folder structure.
- [x] Move and update `tree-view.css`.
- [x] Define `TreeNode<T>` and `TreeViewProps` in `types.ts`.
- [x] Remove `any` from type definitions.

### Phase 2: Logic Extraction (Hooks)
- [x] **`useTreeNavigation`**: Implement keyboard navigation with `core/input`.
- [x] **`useTreeDragDrop`**: Encapsulate generic DnD logic and cross-domain validation.

### Phase 3: Atomic Component Decomposition
- [x] Create `TreeViewToggle`: Handles expansion/collapse state icons.
- [x] Create `TreeViewIcon`: Renders dynamic components with theme-aware coloring.
- [x] Create `TreeViewLabel`: Handles text truncation and display.
- [x] Create `TreeViewBadge`: Wrapper for slot-based badges.
- [x] Create `TreeViewInput`: Specialized input for renaming, using `createShortcut` for internal control.

### Phase 4: Assembly & Core Refactoring
- [x] Implement `TreeViewItem.tsx` with row decomposition to manage complexity.
- [x] Implement generic root drop logic in `TreeView.tsx`.
- [x] **Fix Bug**: Ensure local reordering (move) vs. external assignment (copy) is correctly handled via `dropEffect`.

### Phase 5: Verification & Cleanup
- [x] Replace usage in `TagTreeSidebarPanel.tsx`.
- [x] Replace usage in `FolderTreeSidebarPanel.tsx`.
- [x] **Fix**: Resolve SolidJS reactivity warnings by extracting async handlers into named functions.
- [x] Run linting and type checks (All Clear).
- [x] Delete legacy `src/components/ui/TreeView.tsx`.

## 🧪 Verification Plan
- [x] **Keyboard Navigation**: Arrows, Enter, Escape, and Focus management.
- [x] **Drag & Drop**:
    - [x] Move Tag A to Tag B (Reorder/Nest).
    - [x] Drag Image to Tag (Assignment).
    - [x] Validate circular drop prevention.
- [x] **Reactivity**: Expansion and selection state persistence.
- [x] **Code Quality**: No abbreviations, no `any`, 100% TSDoc coverage.
