# Sprint 4: Busca Avançada e Inspetor

**Data:** 2026-02-26  
**Status:** Planejado  
**Objetivo:** Refatorar os dois domínios mais densos em lógica de processamento de metadados, garantindo que buscas e edições sejam atômicas e validadas.

---

## 🏗️ 1. Interações Abrangidas

### Interação 3: Busca Avançada e Smart Folders
- [ ] **Esvaziamento do `useAdvancedSearch`:**
    - Mover validação de critérios e geração de IDs para `filterActions`.
- [ ] **Normalização de Critérios:**
    - Implementar Schemas Zod recursivos para `SearchGroup` e `Criterion`.
- [ ] **Gestão de Smart Folders:**
    - Unificar a criação/edição em `metadataActions.saveSmartFolder`, removendo a orquestração manual do modal.

### Interação 4: Inspetor e Metadados
- [ ] **Ações em Lote (Batch Edit):**
    - Criar `metadataActions.updateAssetsTags` (batch add/remove).
    - Criar `metadataActions.updateAssetsMetadata` (rating, notas).
- [ ] **Isolamento de Services:**
    - `InspectorTags.tsx` deve apenas emitir eventos, sem chamar `tagService` diretamente.
- [ ] **Cache de Metadados:**
    - Centralizar a busca de EXIF/Technical Info em `actions` da store, evitando múltiplos `createResource` espalhados.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/search/`, `src/components/features/inspector/`.
- **Core Store:** `src/core/store/filterStore.ts`, `src/core/store/metadataStore.ts`.
- **Hooks:** `src/core/hooks/useAdvancedSearch.ts`.

## 📋 3. Critérios de Aceite (DoD)

1. [ ] `useAdvancedSearch` reduzido a menos de 100 linhas (focado apenas em estado de UI).
2. [ ] Busca avançada validada por schema antes de disparar consulta ao backend.
3. [ ] Edição de tags no Inspetor funciona corretamente para seleções múltiplas via Action atômica.
4. [ ] Nenhuma chamada a `tagService` ou `tauriService` nos componentes de Inspetor.
5. [ ] Smart Folders salvos e atualizados sem recarregamento manual da UI.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Performance em Filtros Complexos** | Garantir que o Zod valide apenas a estrutura, não a lógica da query, para evitar overhead. |
| **Inconsistência de Estado no Batch Edit** | Usar transações simuladas na Store (rollback local se a Action do backend falhar). |
