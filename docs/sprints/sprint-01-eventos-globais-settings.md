# Sprint 1: Eventos Globais e Configurações

**Data:** 2026-02-26 - **Status:** Concluída ✅
**Objetivo:** Remover acoplamentos óbvios da Status Bar e do painel de Configurações, estabelecendo o padrão de `actions` em domínios de baixo risco.

---

## 🏗️ 1. Interações Abrangidas

### Interação 7: Status Bar e Eventos Globais
- [x] **Ação 1: Desacoplamento do Status Bar**
    - Criar `systemActions.openSettings()` e `systemActions.openDesignSystem()`.
    - Substituir `window.dispatchEvent` em `StatusSystem.tsx` por chamadas diretas às novas ações.
    - Mover WebviewWindow logic para o Core (systemStore).
- [x] **Ação 2: Seletores Reativos para Contadores**
    - Implementar memos reativos em `useLibrary` e `useSelection` para `loadedCount` e `selectedCount`.
    - Limpar `StatusCounts.tsx` para ser puramente apresentacional.
- [x] **Ação 3: Listeners de Progresso Real**
    - Conectar `listen('thumbnail:queue-status')` no `systemStore`.
    - Expor sinal `thumbnailProgress` para o `StatusSystem.tsx`.
- [x] **Ação 4: Isolamento de I/O em Settings**
    - Mover `runDbMaintenance`, `clearCache` e `cleanupCache` para `systemActions` no `systemStore`.
- [x] **Ação 5: Sincronização e Validação de Preferências**
    - Criar `settingsStore.ts` para gerenciar threads, retenção e estatísticas de cache.
    - Implementar `AppearancePayloadSchema` e `SettingsPayloadSchema` em `src/core/store/settings/schemas.ts`.
    - Refatorar `GeneralPanel.tsx` para usar `useSettings` e `useSystem`, removendo sinalizadores locais.
- [x] **Ação 6: Refatoração do Metadata Store (Isolamento Radical)**
    - Remover dependências de UI (`toast`) do `metadataStore.ts`.
    - Adaptar ações para retornar `ActionResult`, permitindo que a UI trate as notificações.
- [x] **Ação 7: Sistema de Notificações Desacoplado**
    - Criar hook `useMetadataNotifications.ts` para capturar eventos de sync do backend e exibir toasts na camada de UI.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/statusbar/`, `src/components/features/settings/`, `src/components/features/search/`.
- **Core:** `src/core/store/systemStore.ts`, `src/core/store/appearanceStore.ts`, `src/core/store/metadataStore.ts`, `src/core/store/settingsStore.ts`.
- **Hooks:** `src/core/hooks/useMetadataNotifications.ts`, `src/core/hooks/useSettings.ts`.
- **API:** `src/core/tauri/services.ts` (consumo via actions).

## 📋 3. Critérios de Aceite (DoD)

1. [x] Nenhum `window.dispatchEvent` usado para controle de UI nos componentes tocados. ✅
2. [x] `GeneralPanel` e `StatusSystem` não importam `tauriService`. ✅
3. [x] Todas as mutações de preferência passam por schemas Zod. ✅
4. [x] Progresso de thumbnails refletido na UI via eventos reais do backend. ✅
5. [x] Refactoring não quebra a persistência do tema (Light/Dark/System). ✅
6. [x] Core (Stores) livre de dependências de componentes (toasts/modais). ✅

### Detalhes de Implementação
- **Store Centralizada:** Criada `settingsStore.ts` para isolar configurações de aplicação das de aparência.
- **Hooks Reativos:** `useLibrary` e `useSelection` agora expõem contadores memoizados (`loadedCount`, `selectedCount`).
- **Segurança:** `AppearancePayloadSchema` e `SettingsPayloadSchema` garantem que apenas dados válidos sejam persistidos.
- **Isolamento de UI no Core:** `metadataStore.ts` refatorado para usar `ActionResult`. O feedback visual agora é responsabilidade dos componentes ou do hook `useMetadataNotifications`.
- **Resolução de Tipagem:** Eliminado o uso de `any` em callbacks de salvamento, substituindo por tipos genéricos de `ActionResult`.
- **Refatoração de App.tsx:** Removido gerenciamento de estado local de modais e listeners de eventos globais legados.

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Inconsistência entre Abas de Configuração** | Garantir que o `appearanceActions.initialize` carregue todas as preferências atômicas. |
| **Perda de Reatividade em Sinais Derivados** | Utilizar `createMemo` ou funções reativas estáveis para os contadores da Status Bar. |
