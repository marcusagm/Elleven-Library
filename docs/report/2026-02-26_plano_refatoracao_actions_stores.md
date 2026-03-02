# Plano de Implementação: Padronização de Actions e Desacoplamento UI/Store

**Data:** 2026-02-26  
**Status:** Concluído (Sprints 0-8 Executadas com Sucesso)  
**Objetivo:** Isolar a lógica de negócio da camada de visão (Solid.js), garantindo que os componentes sejam puramente UI (Presentational) e que as mutações de estado ocorram exclusivamente através de `actions` tipadas e validadas por schemas.

---

## 1. Visão Geral da Arquitetura Alvo

Desejamos migrar do modelo atual (mutações ad-hoc e acoplamento direto) para um modelo de fluxo unidirecional rígido:

1.  **UI Component:** Dispara um evento (ex: `onDelete`).
2.  **Action/Adapter:** O componente chama uma `action` da Store enviando um `payload`.
3.  **Validation:** A `action` valida o `payload` contra um **Schema** (Zod).
4.  **Domain Logic:** A Store processa a lógica de negócio e comunica-se com o backend (Tauri) se necessário.
5.  **State Mutation:** Apenas a Store altera seu próprio estado reativo.
6.  **Reactivity:** A UI reflete a mudança automaticamente via sinal/store reativa.

## 2. Cronograma de Análise e Implementação (Interações)

### Interação 1: Viewport e Seleção
A base da visualização dos ativos e gestão de estado de foco.

**Componentes e Vínculos:**
- **Feature:** `AssetCard.tsx` (Individual), `VirtualGridView.tsx` (Container).
- **Hook:** `useAssetCardActions.ts`.
- **Store:** `selectionStore.ts`.
- **DnD:** `assetDragSource` (directive).

**Problemas Identificados:**
1.  **Lógica Drag-to-Select Misturada:** A lógica de seleção em lote e arrasto está entranhada no `AssetCard.tsx`, dificultando a reciclagem de componentes na virtualização.
2.  **Payload de Seleção Não Validado:** As mutações de seleção aceitam IDs diretamente sem verificar a existência no domínio.
3.  **Acoplamento DnD:** O `AssetCard` gerencia sinais locais de `dragCounter` e consulta o `dndRegistry` diretamente para decidir se é um `dropTarget`.

**Plano de Ação para Interação 1:**
- [x] Criar `src/core/store/selection/schemas.ts` com schemas para `SelectionPayload`.
- [x] Mover a lógica de DnD (handlers de eventos) do `AssetCard.tsx` para um utilitário ou hook especializado, deixando o card apenas com a responsabilidade de "sinalizar" ser um drop target.
- [x] Centralizar mutações de seleção (toggle, range, clear) na `selectionStore`, garantindo que a UI apenas dispare intenções.

### Interação 2: Navegação e Gerenciamento de Biblioteca
Foco na estrutura de arquivos, pastas e sincronização com o backend.

**Componentes e Vínculos:**
- **Feature:** `LibrarySidebarPanel.tsx`, `FolderTreeSidebarPanel.tsx`, `FolderDeleteModal.tsx`, `FolderContextMenu.tsx`.
- **Hook:** `useLibrary.ts`, `useMetadata.ts`, `useFilters.ts`.
- **Store:** `libraryStore.ts`, `metadataStore.ts`.

**Problemas Identificados:**
1.  **I/O e APIs Tauri na UI:** O `FolderTreeSidebarPanel` e `FolderDeleteModal` realizam chamadas diretas ao `invoke` e `open` (diálogos do sistema), além de manipularem `localStorage` para salvar o estado de expansão.
2.  **Orquestração de Mutações Manual:** Ao adicionar ou remover pastas, os componentes chamam manualmente múltiplas funções de refresh (`loadLocations`, `loadStats`), em vez de disparar uma única "Action" atômica.
3.  **Acoplamento em Batch Changes:** `libraryStore.handleBatchChange` mistura lógica de sincronização com regras de negócio de hierarquia de pastas.
4.  **Lógica Oculta em Callbacks:** O `onDelete` em `FolderContextMenu` transporta objetos pesados (`TreeNode`) para a UI, violando a tipagem estrita de payloads.

**Plano de Ação para Interação 2:**
- [x] Criar `src/core/store/library/schemas.ts` com schemas para `AddLocationPayload`, `RemoveLocationPayload` e `BatchChangePayload`.
- [x] Mover todas as chamadas `invoke` e lógica de `localStorage` para Actions nas stores correspondentes.
- [x] Implementar uma Action `addLocation` na `metadataStore` que orquestre internamente o I/O, o refresh de dados e o disparo de notificações.
- [x] Refatorar `FolderTreeSidebarPanel` para utilizar apenas IDs simples em seus eventos, delegando a busca de dados para a Store.
- [x] Centralizar a lógica de construção da hierarquia (Tree View) em um seletor especializado ou helper de domínio.

### Interação 3: Busca Avançada e Smart Folders
O motor de inteligência de busca e persistência de consultas.

**Componentes e Vínculos:**
- **Feature:** `src/components/features/search` (Toda a subpasta, incluindo `AdvancedSearchModal`, `SmartFoldersSidebarPanel`, e os campos em `fields/`).
- **Hook:** `useAdvancedSearch.ts` (Lógica complexa de estado de busca) e `useFilters.ts`.
- **Store:** `filterStore.ts` (Persistência) e `metadataStore.ts` (Gestão de Smart Folders).

**Problemas Identificados:**
1.  **Hook Inflado (`useAdvancedSearch.ts`):** O hook possui quase 400 linhas de lógica de processamento, validação e mapeamento de critérios que deveriam estar na Store ou em um Domain Service.
2.  **Construção Manual de Queries:** O `AdvancedSearchModal.tsx` reconstrói o objeto `SearchGroup` manualmente e gera IDs via `lib/primitives`, expondo detalhes internos à UI.
3.  **Fragmentação de Handlers:** A lógica de validação e formatação está em um `criterionHandlerRegistry` que mistura renderização com processamento de dados.
4.  **Orquestração Manual de Smart Folders:** Componentes como `SmartFolderDeleteModal` orquestram manualmente disparos de notificação e refresh de metadados após chamadas às actions.

**Plano de Ação para Interação 3:**
- [x] Criar `src/core/store/filter/schemas.ts` com suporte recursivo para `SearchGroupSchema` e `CriterionSchema`.
- [x] Migrar a lógica de processamento de critérios de `useAdvancedSearch.ts` para Actions na `filterStore`.
- [x] Implementar um **Search Domain Service** para centralizar a geração de IDs de grupos e a normalização de critérios antes de chegarem à store.
- [x] Refatorar a gestão de Smart Folders na `metadataStore` para utilizar o **Domain Event Dispatcher** (emitindo `SMART_FOLDER_CREATED`, etc), removendo a necessidade de orquestração manual na UI.
- [x] Desacoplar os componentes em `src/components/features/search/fields` para que sejam puramente apresentacionais, recebendo valores e emitindo mudanças.

### Interação 4: Inspetor e Edição de Propriedades
O painel de controle de metadados e visualização técnica dos ativos.

**Componentes e Vínculos:**
- **Feature:** `src/components/features/inspector` (Toda a subpasta, incluindo `ImageInspector`, `VideoInspector`, `MultiInspector` e `InspectorTags`).
- **Feature:** `CommonMetadata.tsx` (Edição de rating e notas).
- **Hook:** `useMetadata.ts` e `useLibrary.ts`.
- **Store:** `metadataStore.ts` e `libraryStore.ts`.

**Problemas Identificados específicos do Inspector:**
1.  **Lógica de Negócio em `InspectorTags.tsx`:** O componente realiza cálculos de diff (adições vs remoções) e dispara múltiplas chamadas individuais ao `tagService`, violando o fluxo unidirecional.
2.  **I/O Direto no Componente:** `AdvancedMetadata.tsx` e `InspectorTags.tsx` utilizam `createResource` com chamadas diretas ao `tagService` (camada `lib`), ignorando a camada de persistência/actions das stores.
3.  **Agressão às Camadas (UI na Store):** A `metadataStore` dispara `toast` diretamente, impedindo testes isolados.
4.  **Inconsistência de Reatividade:** Alguns componentes do inspetor usam `refetch()` local em vez de reagir a mudanças no estado global da Store.

**Plano de Ação para Interação 4:**
- [x] Criar `TagMutationSchema` em `metadataStore` para lidar com adições/remoções em lote de forma atômica.
- [x] Mover lógicas de EXIF e tags de `createResource` em componentes para `actions` assíncronas na Store, mantendo o cache de metadados centralizado.
- [x] Implementar um **Domain Event Dispatcher** para que a Store apenas emita "TAG_UPDATED" e outros domínios reajam a isso sem acoplamento direto.
- [x] Mover disparos de `toast` para a camada de Hooks ou um **Notification Service** desacoplado.
- [x] Transformar `InspectorTags` em um componente puramente apresentacional que apenas emite eventos `onAddTags` e `onRemoveTags`.

### Interação 5: ItemView e Visualizadores de Mídia
O ambiente de visualização imersiva para diferentes formatos de ativos.

**Componentes e Vínculos:**
- **Feature:** `src/components/features/itemview` (Toda a subpasta, incluindo `ItemView.tsx` e `renderers/`).
- **Context:** `ItemViewContext.tsx` (Estado local do visualizador: zoom, flip, slideshow).
- **Hook:** `useViewport.ts` e `useLibrary.ts`.
- **Store:** `videoStore.ts`, `audioStore.ts`, `libraryStore.ts`.

**Problemas Identificados:**
1.  **Lógica de Navegação na UI:** `ItemView.tsx` calcula manualmente o próximo/anterior asset via índices de array, em vez de delegar para uma Action da Store/Hook.
2.  **Mutações Diretas de Contexto:** Toolbars e Renderers (ex: `FontToolbar`, `ImageViewer`) manipulam o estado do `ItemViewContext` diretamente, sem camadas de validação ou schemas.
3.  **Uso de Event Bus Global:** O sistema utiliza `window.dispatchEvent('viewport:fit')` para comunicação interna, ignorando os canais reativos do Solid.js.
4.  **Efeitos Colaterais de Playback:** Lógicas de temporizador (Slideshow) residem no componente `ItemView.tsx`, tornando difícil o controle externo ou testes.

**Plano de Ação para Interação 5:**
- [x] Criar `ViewerActions` no `viewportStore` ou um hook dedicado que encapsule lógicas de `navigateNext`, `navigatePrev` e `toggleSlideshow`.
- [x] Padronizar o `ItemViewContext` para aceitar apenas **Actions** e não setters diretos, validando mudanças de zoom/rotação via schemas.
- [x] Substituir eventos `window` por sinais ou métodos expostos pelo Context/Store.
- [x] Mover a lógica de temporização do Slideshow para uma Store (ex: `systemStore` ou `viewportStore`) para garantir consistência.
- [x] Garantir que renderers de mídia (Font, Model3D) utilizem payloads tipados para suas configurações específicas.

### Interação 6: Configurações e Preferências
O painel de controle e customização do aplicativo.

**Componentes e Vínculos:**
- **Feature:** `src/components/features/settings` (Toda a subpasta, incluindo `GeneralPanel`, `AppearancePanel`, `KeyboardShortcutsPanel`).
- **Store:** `appearanceStore.ts`, `shortcutStore.ts` (parte do `inputStore`), `transcodeStore.ts`.
- **API:** `tauriService.ts` (Persistência no banco de dados SQLite).

**Problemas Identificados:**
1.  **I/O Direto no Componente (`GeneralPanel.tsx`):** O componente invoca métodos do `tauriService` (ex: `runDbMaintenance`, `clearCache`) diretamente em seus handlers, ignorando a camada de Actions.
2.  **Duplicação de Estado:** `GeneralPanel` mantém sinais locais (`threads`, `cacheRetentionDays`) para valores que deveriam ser sincronizados via Store a partir das configurações do backend.
3.  **UI em Lógica de Negócio:** Assim como nas outras stores, o uso de `toast` ocorre dentro de try/catch nos componentes, em vez de ser gerenciado por um serviço de notificação baseado em eventos de domínio.
4.  **Acoplamento em Atalhos:** `KeyboardShortcutsPanel` acessa métodos brutos do `shortcutStore` para detecção de conflitos e persistência, sem validação de schema para os novos atalhos gravados.

**Plano de Ação para Interação 6:**
- [x] Criar `src/core/store/settings/schemas.ts` com schemas para `AppearancePayload`, `CacheCleanupPayload` e `ShortcutEditPayload`.
- [x] Mover todas as chamadas ao `tauriService` para Actions na `systemStore` ou em uma nova `settingsStore`.
- [x] Implementar um **Settings Domain Service** para lidar com a lógica de gravação individual das preferências no backend.
- [x] Padronizar a detecção de conflitos de teclado para que retorne um `Result` (Success/Error) tipado, facilitando a exibição na UI.

### Interação 7: Status Bar e Eventos Globais
A camada de feedback passivo e indicadores de saúde do sistema.

**Componentes e Vínculos:**
- **Feature:** `src/components/features/statusbar` (incluindo `StatusCounts`, `StatusSystem`).
- **Hook:** `useSystem.ts`, `useLibrary.ts`, `useSelection.ts`.
- **Event Bus:** Uso de `window.dispatchEvent` para comunicação entre componentes.

**Problemas Identificados:**
1.  **Eventos Globais Brutos:** Uso de `window.dispatchEvent(new CustomEvent(...))` no `StatusSystem.tsx` para abrir modais (Settings, Design System), ignorando o estado centralizado ou Actions.
2.  **Lógica de Apresentação em Props Reativas:** O cálculo de contadores (Selecionados vs Filtrados) está espalhado entre as funções de renderização do `StatusCounts.tsx`.
3.  **Mocks de Progresso:** Existem sinais locais "Mocados" no `StatusSystem` que deveriam estar ouvindo eventos reais do Tauri (ex: `thumbnail:queue-status`).

**Plano de Ação para Interação 7:**
- [x] Criar Actions na `systemStore` para gerenciar a abertura/fechamento de modais globais, removendo os `CustomEvent` brutos.
- [x] Implementar Seletores Reativos (derived signals) na Store para contagens complexas, permitindo que a Status Bar seja apenas um consumidor de dados puros.
- [x] Conectar o listener de eventos do Tauri (`listen`) dentro do ciclo de vida da `systemStore` para reportar progresso de I/O em tempo real.

### Interação 8: Sistema de Tags e Hierarquia
A gestão de taxonomia e organização lógica.

**Componentes e Vínculos:**
- **Feature:** `src/components/features/tags` (incluindo `TagTreeSidebarPanel`, `TagDeleteModal`, `TagContextMenu`).
- **Service:** `tagService.ts` (Abstração de I/O).
- **Store:** `metadataStore.ts` (onde reside a lista de tags e estatísticas).

**Problemas Identificados:**
1.  **Lógica Recursiva na UI:** O `TagDeleteModal` contém funções para calcular descendentes e realiza loops de exclusão (`deleteTag`) sequencialmente na UI.
2.  **Undo Ad-hoc:** A funcionalidade de "Desfazer" exclusão de tags está implementada como um callback dentro de uma notificação na UI, realizando novas chamadas de I/O complexas fora da Store.
3.  **Gerenciamento manual de LocalStorage:** O estado de expansão da árvore de tags é gerenciado por `localStorage` diretamente no componente, similar ao problema visto em pastas.
4.  **Construção de Nomes Únicos na UI:** A lógica `getUniqueTagName` valida contra o estado do componente/store, mas a regra de negócio de unicidade deveria ser garantida pela Store/Service.

**Plano de Ação para Interação 8:**
- [x] Criar `src/core/store/metadata/tag-schemas.ts` para validar payloads de criação, renomeação e deleção (incluindo deleção recursiva).
- [x] Migrar a lógica de "Deleção Recursiva" para uma Action atômica na Store que utilize transações ou orquestração segura no backend.
- [x] Centralizar a lógica de "Snapshots" para Undo em um serviço de **Domain Events** ou no histórico da Store.
- [x] Unificar a gestão de estados de expansão (Folders e Tags) em uma sub-store de `uiState` persistente (`treeStore`).

### Interação 9: Engine de Viewport e Layout Workers
O motor de alta performance para virtualização e renderização.

**Componentes e Vínculos:**
- **Core:** `src/core/viewport` (incluindo `ViewportController.ts`, `layout.worker.ts`).
- **Feature:** `VirtualGridView.tsx` (que consome o controller).
- **Store:** `viewportStore.ts` (ou a store que gerenciará a instância do controller).

**Problemas Identificados:**
1.  **Estado Híbrido Reativo:** O `ViewportController.ts` gerencia sinais SolidJS (`_visibleItems`, `_totalHeight`) de forma manual via getters, o que pode causar inconsistências se o controller for recriado ou compartilhado inadequadamente.
2.  **Comunicação Worker-Main Thread sem Validação:** As mensagens trocadas com o `LayoutWorker` (mais de 11.000 bytes de lógica) são baseadas em tipos simples de TS. Se o worker falhar ou retornar dados inesperados, a UI pode quebrar sem uma camada de proteção (Zod).
3.  **Lógica de "RAF" e Throttling Manual:** O controller implementa seu próprio sistema de `requestAnimationFrame` para scroll e `setTimeout` para resize. Essa lógica deveria ser centralizada em utilitários de sistema ou integrada ao ciclo de vida de uma Store.
4.  **Dificuldade de Testabilidade:** Por depender de Workers nativos do navegador e I/O assíncrono, a lógica de layout é difícil de testar em isolamento.

**Plano de Ação para Interação 9:**
- [x] Criar `src/core/viewport/schemas.ts` para validar o fluxo de dados entre o Worker e a Main Thread (especialmente `ItemPosition` e `LAYOUT_COMPLETE`).
- [x] Refatorar o `ViewportController` para ser um **Domain Service** puro, retornando dados que a Store então utiliza para atualizar sinais reativos oficiais.
- [x] Centralizar as instâncias de Workers em uma `serviceRegistry` para evitar vazamentos de memória ou múltiplas threads desnecessárias.
- [x] Padronizar os mecanismos de `RAF` e `Debounce` utilizando um padrão de "System Scheduler" (`scheduler.ts`) para garantir que a performance do viewport não compita com outras animações do sistema.

### Interação 10: Arquitetura de Drag and Drop (DnD)
O sistema de interação física entre diferentes domínios (Ativos ↔ Tags ↔ Pastas).

**Componentes e Vínculos:**
- **Core DnD:** `src/core/dnd` (incluindo `dnd-core.ts`, `ImageDropStrategy.ts`, `TagDropStrategy.ts`).
- **Feature Source:** `AssetCard.tsx`, `TagTreeSidebarPanel.tsx`.
- **Feature Target:** Viewports (`VirtualGridView`, `VirtualListView`, `VirtualMasonry`).

**Problemas Identificados:**
1.  **Tipagem "Fraca" em `DragItem`:** O uso de `Record<string, unknown>` no payload do `DragItem` impede a validação estrita em tempo de compilação (necessidade de Discriminated Union).
2.  **Regras de Negócio em Estratégias:** A lógica de "se o ativo está selecionado, aplique a tag a todos" está dentro de `ImageDropStrategy.ts`, em vez de ser uma Action da Store.
3.  **Hemorragia de UI na Lógica:** Estratégias DnD chamam `toast` e refreshes de metadados (`notifyTagUpdate`) diretamente, criando acoplamento circular e dificultando testes.
4.  **Complexidade de Reordenamento:** A lógica de cálculo de `order_index` e hierarquia em `TagDropStrategy.ts` é densa e deveria ser abstraída em um **Tag Domain Service**.

**Plano de Ação para Interação 10:**
- [x] Converter `DragItem` para uma **Discriminated Union** (ex: `{ type: 'IMAGE'; payload: { ids: number[] } } | { type: 'TAG'; payload: TagDragPayload }`).
- [x] Criar Actions dedicadas na `libraryStore`/`filterStore` para processar resultados de Drop (ex: `applyTagToSelection`, `reorderTags`).
- [x] Remover todas as referências a `toast` e disparos manuais de refresh das `DropStrategy`. Elas devem retornar apenas uma "Intenção de Mudança" ou disparar um evento de domínio.
- [x] Encapsular a lógica de `dragCounter` e lookup de registro em um helper reativo para simplificar `AssetCard` e `TagTreeSidebarPanel`.

---

## 3. Fases de Implementação

### Fase 1: Infraestrutura e Contratos (Base)
*   [x] **Definição de Tipos Globais:** Criar `src/core/types/actions.ts` para definir interfaces padrão de resposta e erro.
*   [x] **Padrão de Payload:** Estabelecer que todo payload complexo deve ser um objeto nomeado (ex: `DeleteAssetPayload`).
*   [x] **Integração Zod:** Adicionar `zod` e `zod-validation-error` ao projeto para validações de tempo de execução nos Payloads.
*   [x] **Factory de Actions:** (Opcional) Criar um utilitário `createSecureAction` para automatizar log de erros e validação de schema.

### Fase 2: Refatoração do "Core" (Stores Críticas)
Refatorar as stores que possuem maior interdependência e complexidade:
*   [x] **`systemStore.ts`**: Remover lógicas de inicialização complexas de dentro dos componentes e centralizar em `systemActions.initialize`.
*   [x] **`libraryStore.ts`**: Padronizar as ações de gestão de arquivos e cache. Implementar Schemas para o `BatchChangePayload`.
*   [x] **`metadataStore.ts`**: Desacoplar a lógica de busca e filtros do componente `AdvancedSearchModal`.

### Fase 3: Refatoração de Domínios Periféricos
Aplicar o mesmo padrão nas stores de suporte:
*   [x] **`selectionStore.ts`**: Garantir tipagem estrita nos IDs e remover qualquer lógica de UI de dentro da store.
*   [x] **`appearanceStore.ts`**: Centralizar troca de temas e persistência.
*   [x] **`audioStore.ts` / `videoStore.ts`**: Isolar controle de playback dos elementos de mídia da UI.

### Fase 4: Desacoplamento Total da UI (Pure Components)
Revisar e limpar componentes que ainda realizam lógica de negócio:
*   [x] **Componentes de Lista (`Table`, `TreeView`):** Garantir que eles apenas recebam dados e emitam eventos.
*   [x] **Sidebar Panels:** Transformar painéis em consumidores passivos de estado.
*   [x] **Modais de Ação:** Remover chamadas diretas a APIs Tauri dos componentes, movendo-as para as stores.

### Fase 5: Verificação, Segurança de Tipos e Limpeza Final (Sprints 6-8)
*   [x] **Eliminação total de `any` (Sprint 6):** Substituir todos os `any` remanescentes (atualmente 13) no diretório `src/core` por tipos derivados de Schemas Zod ou uniões discriminadas.
*   [x] **Refatoração de Complexidade (Sprint 6):** Reduzir a complexidade de `dispatcher.ts` e `normalizer.ts` para < 10.
*   [x] **Divisão de Arquivos God-Files (Sprint 7):** Decompor `libraryStore.ts`, `metadataStore.ts` e `filter/index.ts` em módulos de ações e estado < 300 linhas.
*   [x] **Remoção de `eslint-disable` (Sprint 7):** Corrigir as causas raízes de todos os linters ignorados.
*   [x] **Audit de Dependências Cíclicas (Sprint 8):** Utilizar ferramentas de análise estática para garantir que stores não importem umas às outras de forma circular.

---

## 4. Gestão de Riscos e Segurança

| Risco | Estratégia de Mitigação |
| :--- | :--- |
| **Quebra de Reatividade** | Nunca utilizar desestruturação de `props` ou de `state` da store sem `splitProps` ou getters. |
| **Regressão na Busca** | Realizar testes de fumaça manuais no `AdvancedSearchModal` após cada alteração no metadataStore. |
| **Performance (Zod)** | Utilizar validação de schema apenas em ações disparadas por input do usuário ou eventos externos (Tauri). |
| **Dependência Circular** | Mover interfaces e schemas para arquivos separados (`types.ts` / `schemas.ts`) para que stores possam compartilhá-los sem importar a store vizinha. |

---

## 5. Critérios de Aceitação (Definition of Done)

1.  [x] Nenhuma store realiza mutação de estado fora de uma função exportada em `actions`.
2.  [x] Nenhum componente UI importa `setStore` ou sinais de escrita (setters) diretamente.
3.  [x] Todos os payloads de actions possuem uma interface TypeScript e um Schema de validação associado.
4.  [x] O número de ocorrências de `any` no diretório `src/core` é ZERO.
5.  [x] O build e o lint passam sem avisos de complexidade ou imports circulares.
