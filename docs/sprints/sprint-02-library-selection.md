# Sprint 2: Biblioteca Base e Seleção

**Data:** 2026-02-26  
**Status:** Planejado  
**Objetivo:** Consolidar as mutações centrais de dados (pastas e arquivos) e a lógica de seleção, preparando o terreno para busca e tags.

---

## 🏗️ 1. Interações Abrangidas

### Interação 2: Navegação e Gerenciamento de Biblioteca
- [ ] **Desacoplamento de Pastas:**
    - Criar `libraryActions.addLocation()` e `libraryActions.removeLocation()`.
    - Mover diálogos `tauri.open` e comandos `invoke('remove_location')` dos modais para estas Actions.
- [ ] **Gestão Atômica de Refresh:**
    - Garantir que a Action de remoção dispare sozinha o refresh de `stats` e `locations`.
- [ ] **Persistência de UI:**
    - Mover estado de expansão de pastas (`mundam_tree_expanded`) do `localStorage` direto em componentes para uma sub-store persistente.

### Interação 1: Seleção de Ativos
- [ ] **Refatoração da SelectionStore:**
    - Implementar `selectionActions.toggle`, `selectionActions.selectRange`, e `selectionActions.clear`.
    - Criar `SelectionPayloadSchema`.
- [ ] **Limpeza de AssetCard (Parte I):**
    - Remover verificações de `selectedIds.includes()` do `AssetCard` e hooks de UI, delegando para a Store.
- [ ] **Seletores de Performance:**
    - Criar seletor `isItemSelected(id)` para evitar re-renderizações em massa quando a seleção muda.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/library/`, `src/components/features/viewport/AssetCard.tsx`.
- **Core:** `src/core/store/libraryStore.ts`, `src/core/store/selectionStore.ts`.
- **Hooks:** `src/core/hooks/useSelection.ts`, `src/core/hooks/useLibrary.ts`.

## 📋 3. Critérios de Aceite (DoD)

1. [ ] Exclusão de pastas orquestrada atomicamente pela `libraryStore`.
2. [ ] `FolderDeleteModal` não contém lógica de I/O (apenas emite confirmação).
3. [ ] Seleção múltipla (Shift+Click) funcional via `selectionActions`.
4. [ ] Nenhuma mutação direta de `selectedIds` na camada de Viewport.
5. [ ] Estado de expansão da árvore de pastas sincronizado sem `localStorage` manual nos componentes.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Gargalo em Seleção de Milhares de Itens** | Otimizar a Store para usar `Set` internamente e seletores de comparação rápida. |
| **Race Condition em Refresh de Biblioteca** | Implementar trava de estado (`isRefreshing`) na Action para evitar chamadas duplicadas. |
