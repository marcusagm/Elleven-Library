# Sprint 4: Busca Avançada e Inspetor

**Data:** 2026-02-27  
**Status:** Em Progresso (Aguardando Verificação UI Final)  
**Data e hora da conclusão:** 2026-02-27 12:30  
**Objetivo:** Refatorar os dois domínios mais densos em lógica de processamento de metadados, garantindo que buscas e edições sejam atômicas e validadas.

---

## 🏗️ 1. Interações Abrangidas

### Interação 3: Busca Avançada e Smart Folders
- [x] **Esvaziamento do `useAdvancedSearch`:**
    - Mover validação de critérios e geração de IDs para `filterActions`.
- [x] **Normalização de Critérios:**
    - Implementar Schemas Zod recursivos para `SearchGroup` e `Criterion` (`src/core/store/filter/schemas.ts`).
    - **Novidade:** Criado `criterionLogicRegistry` para isolar lógica de processamento por tipo de campo.
- [x] **Gestão de Smart Folders:**
    - Unificar a criação/edição em `metadataActions.saveSmartFolder`, removendo a orquestração manual do modal.

- [x] **Ações em Lote (Batch Edit):**
    - [x] Criar `metadataActions.updateAssetsTags` (batch add/remove/replace).
    - [x] Criar `metadataActions.updateAssetsMetadata` (rating, notas).
    - [x] **Backend:** Implementados comandos Rust atômicos para atualização em lote.
- [x] **Isolamento de Services:**
    - `InspectorTags.tsx` e `AdvancedMetadata.tsx` agora utilizam `metadataActions` via `useMetadata`. [x]
    - Nenhuma chamada direta a `tagService` nos componentes. [x]
- [x] **Cache de Metadados:**
    - [x] Centralizar a busca de EXIF/Technical Info em `actions` da store, evitando múltiplos `createResource` espalhados.
    - [x] Implementado em `src/core/store/metadata/cache.ts` e integrado na action `getAssetExif`.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/search/`, `src/components/features/inspector/`.
- **Core Store:** `src/core/store/filterStore.ts`, `src/core/store/metadataStore.ts`.
- **Hooks:** `src/core/hooks/useAdvancedSearch.ts`.

## 📋 3. Critérios de Aceite (DoD)

1. [x] `useAdvancedSearch` reduzido a menos de 100 linhas (focado apenas em estado de UI).
2. [x] Busca avançada validada por schema antes de disparar consulta ao backend.
3. [x] Edição de tags no Inspetor funciona corretamente para seleções múltiplas via Action atômica.
    - [x] Detecção inteligente de tags comuns (mostradas no input).
    - [x] Listagem de tags parciais com opção de "adicionar a todos" em um clique.
4. [x] Nenhuma chamada a `tagService` ou `tauriService` nos componentes de Inspetor.
5. [x] Smart Folders salvos e atualizados sem recarregamento manual da UI.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Performance em Filtros Complexos** | Garantir que o Zod valide apenas a estrutura, não a lógica da query, para evitar overhead. |
| **Inconsistência de Estado no Batch Edit** | Usar transações simuladas na Store (rollback local se a Action do backend falhar). |
