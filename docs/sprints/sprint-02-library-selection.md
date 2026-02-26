# Sprint 2: Biblioteca Base e Seleção

**Data:** 2026-02-26  
**Status:** Concluída ✅  
**Objetivo:** Consolidar as mutações centrais de dados (pastas e arquivos) e a lógica de seleção, preparando o terreno para busca e tags.

---

## 🏗️ 1. Interações Abrangidas

### Interação 2: Navegação e Gerenciamento de Biblioteca
- [x] **Desacoplamento de Pastas:**
    - Criar `libraryActions.addLocation()` e `libraryActions.removeLocation()`.
    - Mover diálogos `tauri.open` e comandos `invoke('remove_location')` dos modais para estas Actions.
- [x] **Gestão Atômica de Refresh:**
    - Garantir que a Action de remoção dispare sozinha o refresh de `stats` e `locations`.
- [x] **Persistência de UI:**
    - Mover estado de expansão de pastas (`mundam_tree_expanded`) do `localStorage` direto em componentes para uma sub-store persistente (`treeStore`).

### Interação 1: Seleção de Ativos
- [x] **Refatoração da SelectionStore:**
    - Implementar `selectionActions.toggle`, `selectionActions.selectRange`, e `selectionActions.clear`.
    - Criar `SelectionPayloadSchema`.
- [x] **Limpeza de AssetCard (Parte I):**
    - Remover verificações de `selectedIds.includes()` do `AssetCard` e hooks de UI, delegando para a Store.
- [x] **Seletores de Performance:**
    - Criar seletor `isItemSelected(id)` para evitar re-renderizações em massa quando a seleção muda.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/library/FolderTreeSidebarPanel.tsx`, `src/components/features/library/FolderDeleteModal.tsx`, `src/components/features/viewport/AssetCard.tsx`, `src/components/features/viewport/VirtualMasonry.tsx`, `src/components/features/viewport/VirtualGridView.tsx`, `src/components/features/viewport/VirtualListView.tsx`.
- **Core:** `src/core/store/libraryStore.ts`, `src/core/store/selectionStore.ts`, `src/core/store/treeStore.ts`, `src/core/store/selection/schemas.ts`.
- **Hooks:** `src/core/hooks/useSelection.ts`, `src/core/hooks/useLibrary.ts`, `src/core/hooks/useTree.ts`, `src/core/hooks/useAssetCardActions.ts`, `src/core/hooks/useGridKeyboardNav.ts`, `src/core/hooks/gridNavHelpers.ts`.

## 📋 3. Critérios de Aceite (DoD)

1. [x] Exclusão de pastas orquestrada atomicamente pela `libraryStore`.
2. [x] `FolderDeleteModal` não contém lógica de I/O (apenas emite confirmação).
3. [x] Seleção múltipla (Shift+Click) funcional via `selectionActions`.
4. [x] Nenhuma mutação direta de `selectedIds` na camada de Viewport (delegado para `useAssetCardActions`).
5. [x] Estado de expansão da árvore de pastas sincronizado sem `localStorage` manual nos componentes.

## 🛠️ 4. Detalhes de Implementação

- **Performance de Seleção:** Implementado `createSelector` no `selectionStore.ts`. O seletor `isItemSelected(id)` garante que cada `AssetCard` só re-renderize se o seu PRÓPRIO estado de seleção mudar, evitando o custo $O(N)$ em cada clique de seleção em grades grandes.
- **Seleção por Range:** Adicionado suporte a `Shift+Click` e `Shift+Arrow/Space` para selecionar faixas de itens. A lógica usa um `lastSelectedId` como âncora para determinar o início da seleção.
- **Gestão de Pastas:** `libraryActions.addLocation` e `removeLocation` agora centralizam toda a lógica, incluindo diálogos do Tauri e refreshes atômicos de metadados (`loadLocations`, `loadStats`).
- **Trava de Refresh:** Adicionado `isRefreshing` no `libraryStore` para evitar race conditions em operações rápidas de adição/remoção.
- **Persistência desacoplada:** criada `treeStore` para gerenciar o estado de expansão da árvore, removendo lógica de `localStorage` dos componentes de UI.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Gargalo em Seleção de Milhares de Itens** | Otimizar a Store para usar `Set` internamente e seletores de comparação rápida. |
| **Race Condition em Refresh de Biblioteca** | Implementar trava de estado (`isRefreshing`) na Action para evitar chamadas duplicadas. |
