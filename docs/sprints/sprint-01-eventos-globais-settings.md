# Sprint 1: Eventos Globais e Configurações

**Data:** 2026-02-26  
**Status:** Planejado  
**Objetivo:** Remover acoplamentos óbvios da Status Bar e do painel de Configurações, estabelecendo o padrão de `actions` em domínios de baixo risco.

---

## 🏗️ 1. Interações Abrangidas

### Interação 7: Status Bar e Eventos Globais
- [ ] **Desacoplamento de Modais:**
    - Criar `systemActions.openSettings()` e `systemActions.openDesignSystem()`.
    - Remover `window.dispatchEvent` e `CustomEvent` de `StatusSystem.tsx`.
- [ ] **Seletores Reativos:**
    - Mover cálculo de `totalLoaded`, `totalFiltered` e `selectedCount` para seletores reativos na `systemStore` ou `libraryStore`.
    - `StatusCounts.tsx` deve ser puramente apresentacional.
- [ ] **Listeners Reais:**
    - Conectar `listen('thumbnail:queue-status')` na `systemStore` para atualizar o progresso real no indicador de sistema.

### Interação 6: Configurações e Preferências
- [ ] **Isolamento de I/O em Settings:**
    - Mover chamadas `runDbMaintenance` e `clearCache` de `GeneralPanel.tsx` para `systemActions`.
- [ ] **Sincronização de Estado:**
    - Remover sinais locais (`threads`, `cacheRetention`) do `GeneralPanel`.
    - Sincronizar via `appearanceStore` ou uma nova `settingsStore`.
- [ ] **Validação de Preferências:**
    - Criar `AppearancePayloadSchema` e `SettingsPayloadSchema` em `src/core/store/settings/schemas.ts`.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/statusbar/`, `src/components/features/settings/`.
- **Core:** `src/core/store/systemStore.ts`, `src/core/store/appearanceStore.ts`.
- **API:** `src/core/tauri/services.ts` (consumo via actions).

## 📋 3. Critérios de Aceite (DoD)

1. [ ] Nenhum `window.dispatchEvent` usado para controle de UI nos componentes tocados.
2. [ ] `GeneralPanel` e `StatusSystem` não importam `tauriService`.
3. [ ] Todas as mutações de preferência passam por schemas Zod.
4. [ ] Progresso de thumbnails refletido na UI via eventos reais do backend.
5. [ ] Refactoring não quebra a persistência do tema (Light/Dark/System).

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Inconsistência entre Abas de Configuração** | Garantir que o `appearanceActions.initialize` carregue todas as preferências atômicas. |
| **Perda de Reatividade em Sinais Derivados** | Utilizar `createMemo` ou funções reativas estáveis para os contadores da Status Bar. |
