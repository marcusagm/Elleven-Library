# Sprint 3: Tags e Drag and Drop (DnD)

**Data:** 2026-02-26  
**Status:** Planejado  
**Objetivo:** Modernizar o sistema de taxonomia e unificar a lógica de interação física (DnD), removendo regras de negócio de dentro dos motores de arrasto.

---

## 🏗️ 1. Interações Abrangidas

### Interação 8: Sistema de Tags e Hierarquia
- [ ] **Ações Atômicas de Tags:**
    - Criar `metadataActions.createTag`, `metadataActions.deleteTagRecursive`, e `metadataActions.reorderTags`.
    - Mover a lógica de cálculo de descendentes do `TagDeleteModal` para a Store.
- [ ] **Tag Domain Service:**
    - Isolar a lógica de ordenação e limpeza de nomes em um serviço puro.

### Interação 10: Arquitetura de Drag and Drop
- [ ] **Discriminated Union de DragItem:**
    - Refatorar `DragItem` em `src/core/dnd/dnd-core.ts` para usar tipos explícitos (`IMAGE` | `TAG`).
- [ ] **Desacoplamento de Estratégias:**
    - Remover `toast` e acessos diretos à `selectionStore` de `ImageDropStrategy.ts`.
    - `onDrop` deve apenas emitir uma intenção: `libraryActions.applyTagToSelection(tagId)`.
- [ ] **Limpeza de AssetCard (Parte II):**
    - Remover `dragCounter` e lógica de DnD do `AssetCard`, substituindo por um hook especializado de Drop Zone.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/tags/`, `src/components/features/viewport/AssetCard.tsx`.
- **Core DnD:** `src/core/dnd/strategies/`, `src/core/dnd/dnd-core.ts`.
- **Core Store:** `src/core/store/metadataStore.ts`.

## 📋 3. Critérios de Aceite (DoD)

1. [ ] Tags deletadas recursivamente em uma única operação de Store.
2. [ ] Sistema de DnD 100% tipado (sem `Record<string, unknown>`).
3. [ ] Estratégias de Drop não disparam Toasts (a Action que recebe o drop faz isso).
4. [ ] `AssetCard` 100% livre de lógica de decisão de DnD.
5. [ ] Reordenamento de tags preserva integridade sem chamadas redundantes ao backend.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Complexidade no Reordenamento de Árvores** | Utilizar um algoritmo de ordenação estável (ex: ordem em milhares `1000, 2000`) para evitar colisões. |
| **Hemorragia de Tipos (DragItem)** | Realizar a migração da união discriminada em uma única etapa para evitar erros de compilação em cadeia. |
