# Sprint 3: Tags e Drag and Drop (DnD)

**Data:** 2026-02-26  
**Status:** Concluído ✅  
**Data da conclusão:** 2026-02-27 11:05 (Refinamentos finais)
**Objetivo:** Modernizar o sistema de taxonomia e unificar a lógica de interação física (DnD), removendo regras de negócio de dentro dos motores de arrasto.

---

## 🏗️ 1. Interações Abrangidas

### Interação 8: Sistema de Tags e Hierarquia
- [x] **Ações Atômicas de Tags:**
    - Criados `metadataActions.createTag`, `metadataActions.updateTag`, `metadataActions.deleteTagRecursive`, e `metadataActions.moveTag`.
    - Lógica de cálculo de descendentes movida do `TagDeleteModal` para a Store (`deleteTagRecursive`).
- [x] **Tag Domain Service:**
    - Lógica de normalização de nomes centralizada em `TagDomainService`.

### Interação 10: Arquitetura de Drag and Drop
- [x] **Discriminated Union de DragItem:**
    - Refatorado `DragItem` em `src/core/dnd/dnd-core.ts` para usar tipos explícitos (`IMAGE` | `TAG`) com payloads estritos.
- [x] **Desacoplamento de Estratégias:**
    - Removidos `toast` e acessos diretos à `selectionStore` de `ImageDropStrategy.ts` e `TagDropStrategy.ts`.
    - Implementado `ActionResult` para retorno de estratégias.
    - Centralizada lógica de notificações no hook `useDndHandlers`.
- [x] **Limpeza de AssetCard (Parte II):**
    - Removida lógica de DnD do `AssetCard`, substituída pelo hook especializado `useAssetDropZone`.
- [x] **Refatoração Pure TreeView:**
    - `TreeView.tsx` e `useTreeDragDrop` desacoplados de `useDndHandlers`.
    - Implementado callback `onDrop` para delegação de lógica de negócio aos painéis de feature (`TagTreeSidebarPanel`).
    - Corrigido bug de movimentação para a raiz e para "gaps" dentro de grupos de filhos (gaps entre itens ou áreas de indentação).

---

## 🛠️ 2. Refinamentos de Estabilização (Pós-Sprint)

Após a implementação inicial, foram realizados os seguintes refinamentos críticos conforme detectado em uso:

### 🐛 Correções de Bugs
- [x] **Consistência de Parâmetros (Frontend/Backend):** Corrigido o mapeamento de parâmetros em `tagService` para garantir que `camelCase` no JS reflita corretamente no `snake_case` do Rust via Tauri `invoke`.
- [x] **Estabilidade do Menu de Contexto:** 
    - Corrigido fechamento prematuro do menu ao interagir com o `ColorPicker` (submenus em Portals).
    - Implementada proteção em `createClickOutside` para ignorar cliques dentro de containers de contexto ativos.
- [x] **Cleanup de DnD:** Corrigida a persistência visual do destaque de drop na raiz (`ui-tree-root-drop-active`) através de um listener global ao sinal de arraste.

### ⚡ Otimização e Performance
- [x] **Atualizações Granulares (Store Performance):**
    - Refatorado `updateTag` para realizar patches locais no Store em vez de disparar `loadTags()` (full reload) para mudanças estéticas (cores).
    - Notificação seletiva: Otimizado o fluxo para pular re-renders de estatísticas e imagens se a mudança for apenas visual.
- [x] **Eliminação de Flickering na Árvore:**
    - Implementado `tagTreeStructuralHash` em `TagTreeSidebarPanel` para evitar reconstruções totais da hierarquia DOM.
    - Uso de **Reactive Getters** nos `TreeNode` para que cores e nomes atualizem de forma fina, mantendo a árvore estável durante interações rápidas no ColorPicker.

---

## 📦 3. Arquivos Afetados

- **UI:** `src/components/features/tags/`, `src/components/features/viewport/AssetCard.tsx`, `src/components/ui/TreeView/`, `src/components/ui/ContextMenu/`.
- **Core DnD:** `src/core/dnd/strategies/`, `src/core/dnd/dnd-core.ts`, `src/core/hooks/useDndHandlers.ts`, `src/core/hooks/useAssetDropZone.ts`.
- **Core Store:** `src/core/store/metadataStore.ts`, `src/core/store/libraryStore.ts`.

## 📋 4. Critérios de Aceite (DoD)

1. [x] Tags deletadas recursivamente em uma única operação de Store.
2. [x] Sistema de DnD 100% tipado (sem `any` remanescente em payloads críticos).
3. [x] Estratégias de Drop não disparam Toasts (centralizado em `useDndHandlers`).
4. [x] `AssetCard` 100% livre de lógica de decisão de DnD.
5. [x] Reordenamento de tags preserva integridade e funciona em todos os níveis (raiz, reordenação e aninhamento em "gaps").
6. [x] `TreeView` é um componente puro, agnóstico ao domínio de "tags" ou "imagens".
7. [x] Mudança de cor no ColorPicker é fluida, sem desmontar menus ou causar "flicker" na árvore lateral.

---

## 📈 5. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Complexidade no Reordenamento de Árvores** | Implementado cálculo de `order_index` baseado em vizinhos, garantindo consistência. |
| **Hemorragia de Tipos (DragItem)** | Migrada de forma atômica com verificação completa de tipos via `tsc`. |
| **Flickering e Re-renders Excessivos** | Implementada estabilização estrutural via hashes e getters reativos no mapeamento da hierarquia. |
