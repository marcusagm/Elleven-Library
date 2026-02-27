# Sprint 5: ItemView e Viewport Engine

**Data:** 2026-02-26  
**Status:** Planejado  
**Data e hora da conclusão:** 
**Objetivo:** Finalizar a refatoração nos componentes de maior risco de performance, garantindo que a engine de renderização e o visualizador imersivo sigam os padrões de segurança arquitetural.

---

## 🏗️ 1. Interações Abrangidas

### Interação 5: ItemView e Renderers
- [x] **Desacoplamento de Visualização:**
    - Mover navegação (next/prev) para `viewportActions.navigateToAsset`.
    - Mover controle de zoom/fit para Actions no `viewportStore` ou Context.
- [~] **Padronização de Renderers:**
    - Garantir que `ImageViewer`, `FontRenderer`, etc., usem payloads tipados para suas configurações.
    - `ImageViewer` concluído.
    - `FontRenderer` pendente.
    - `ModelRenderer` pendente.
    - `AudioRenderer` pendente.
    - `VideoRenderer` pendente.

### Interação 9: Viewport Engine (Workers)
- [x] **Segurança de Comunicação (Main-Worker):**
    - Implementar validação de mensagens de entrada/saída do `LayoutWorker` via Zod Schemas.
- [x] **Refatoração do Controller:**
    - Transformar `ViewportController` em um **Domain Service** puro.
    - Sinais reativos oficiais residirão na `viewportStore`.
- [x] **System Scheduler:**
    - Padronizar uso de `requestAnimationFrame` em um utilitário centralizado para evitar contenção de performance.

## 📦 2. Arquivos Afetados

- **UI:** `src/components/features/itemview/`.
- **Core Viewport:** `src/core/viewport/ViewportController.ts`, `src/core/viewport/layout.worker.ts`.
- **Core Store:** `src/core/store/viewportStore.ts` (ou equivalente).

## 📋 3. Critérios de Aceite (DoD)

1. [x] Navegação entre itens no `ItemView` orquestrada isoladamente da UI.
2. [x] Workers de Layout validados (Schemas de entrada e saída).
3. [x] Nenhuma regressão de performance (FPS estável em scroll de 10k+ itens).
4. [x] Memória de Workers gerenciada (terminar workers órfãos).
5. [x] `ActionResult` usado em todas as operações de carregamento de mídia (quando aplicável no backend proxy).

---

## 📈 4. Riscos e Mitigações

| Risco | Mitigação |
| :--- | :--- |
| **Latência por Validação de Schemas nos Workers** | Validar apenas mensagens de configuração (raras) e samples das mensagens de posição (frequentes). |
| **Interrupção de Playback em Refactoring** | Manter o estado de media player isolado e persistente durante a navegação. |
