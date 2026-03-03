# Plano de Ação: Otimizações e Funcionalidades Pendentes (Mundam)

**Data:** 20 de Fevereiro de 2026
**Status:** Concluído (Foco de Interface/Ações)
**Última atualização:** 2026-03-01 — Conclusão de todas as Sprints de 1 a 8, abrangendo padronização completa de Tipagens, Componentes, Viewport Engine, Limpeza de Lint e Otimização Arquitetural.
**Baseado em:** `pendencias_consolidadas.md`

Este documento elabora um plano detalhado de implementação para todas as pendências ativas mapeadas nos relatórios recentes do projeto **Mundam**. Para manter a manutenibilidade, o plano foi segmentado em partes menores, de modo que cada tópico representa uma evolução lógica, focada em recursos específicos, melhorias de arquitetura e otimizações de performance.

---


## 2. Robustez do Backend e Pipeline de Dados

**Motivação:** Alguns gargalos e riscos de loop em processamento massivo podem degradar o app. A manutenção da verdade única sobre detecção de arquivos está defasada no indexador.

### [✓] 2.1 Integração do Processo de Detecção (UMDS)
*   **Ação:** O ecossistema de formatos (`FileFormat::detect`) já foi criado em `definitions.rs`, mas o `indexer/metadata.rs` ainda utiliza a heurística simples (linha 17: `path.extension()?.to_string_lossy().to_string().to_lowercase()`). O objetivo aqui é apenas fazer a **ligação final** dessa nova estrutura robusta de detecção no mecanismo raiz de ingestão e indexação volumosa.

### [✓] 2.2 Tratamento de Erros e Controle de Loops (`Poison Pill`)
*   **Ação (Concluída):** Atualizar o `thumbnail_worker.rs` adicionando suporte explícito à contagem de repetições e sinalização de "Poison Pill", garantindo que falhas contínuas de compressão ou arquivos corrompidos não causem sobrecarga da CPU ou travem o pool Rayon num ciclo passivo.

### [ ] 2.3 Melhorias nos Extratores Auxiliares
*   **Ação:** **SVG:** Embora o `svg.rs` exista, deve-se validar e aprofundar se a renderização por intermédio de webview ou `resvg/librsvg` nativo está capturando perfeitamente as pranchas complexas.
*   **Ação:** Melhorar os extratores desenvolvidos recentemente de vetores brutos (`.ai`, `.eps`) garantindo o scan prioritário aos *PDF Streams* (alta qualidade) em contrapartida ao ícone genérico fallback.

---

## 3. Pesquisa e Organização Semântica (Busca Contextual e Em Massa)

**Motivação:** Bibliotecas grandes precisam de ferramentas de manipulação de metadados em lote e motores de busca avançados orientados tanto a contexto quanto a propriedades estéticas.

### [ ] 3.1 Implementação de Análise Cromática e Busca por Cor
*   **Ação DB:** Expandir o Schema do SQLite (requer SQLx migrations caso ainda não configurado para essa entidade) para englobar uma tabela/módulo `image_colors` e uma coluna para `dominant_color`.
*   **Ação Backend:** Atrelar à pipeline de thumbnailing as métricas extraídas usando FFmpeg/ImageMagick para definir a paleta dominante ou k-means cluster do arquivo.
*   **Ação Frontend:** Criar um componente isolado `ColorPicker` agregado no `FilterStore` da Sidebar para submissões de cor hex/código no motor de busca interno.

### [ ] 3.2 Melhorias do Core do Motor de Busca
*   **Ação:** Instaurar a lógica robusta de **Fuzzy Search** na tipagem textual permitindo que o SQLite (ou lógicas do Rust em memória usando distâncias de edição, ex. Levenshtein) tolere erros de digitação (typos) na pesquisa.

### [✓] 3.3 Batch Tagging (*Operações em Massa*)
*   **Ação (Concluída):** Melhorar o `MultiInspector.tsx` e integrar suporte à interface (UI) e ao SQLite para renomear, excluir, mover agrupamentos e aplicar ou revogar **Tags de milhares de instâncias simultaneamente**.
*   *Nota (2026-03-02): O sistema agora utiliza ativamente a variável reativa global `tagUpdateVersion` em conjunto ao `EventBus` para refletir alterações massivas efetuadas via Drag and Drop instantaneamente no Inspector UI.*

---

## 4. Integrações de Fluxo, Ingestão Remota e Empacotamento

**Motivação:** A arquitetura precisa suportar a introdução veloz de assets sem fricção por meio de extensões e de áreas de transferência.

### [ ] 4.1 Ingestão Web Clipper (Extensão) e Clipboard
*   **Ação:** Desenvolver o endpoint no protocolo stream/interno (como `/ingest`) exposto pelo servidor Web local (via `axum` ou `tauri-plugin-localhost`) recebendo payloads rest (como base64 string, URL remotas).
*   **Ação:** Criar estrutura inicial no navegador para envio da imagem, atrelando à lógica local de download e parse e aceitar interceptação por Deep OS Integration + Ctrl+V Universal do ecossistema.

### [ ] 4.2 Empacotamento Portátil e Cloud Sync Estrutural
*   **Ação:** Implementação de uma UI de "Exportação Inteligente".
*   **Ação:** Desenvolver no backend scripts em Rust em `commands/` encarregados de encapsular as mídias requeridas juntamente a um arquivo de base de dados/manifest com metadados isolando o formato (`.eaglepack` / `.mundampack`) para fins de backup ágil e compartilhamento de metadados limpos.
*   **Ação Genérica:** Investigar soluções nativas e fluxos padronizados para amparo básico de versionamento de Cloud Sync (e.g., GDrive e Dropbox wrappers / watcher de conflitos).

### [ ] 4.3 Suporte Básico à Arquitetura de Plugins e Scripts
*   **Ação:** Planejar o ecossistema base e API exposta do Tauri com permissões escaladas que viabilize em um futuro a integração por scripts customizados.

---

## 5. UI/UX: Componentizações e Especializações Variadas

**Motivação:** Interfaces inchadas podem dificultar a injeção reativa do Solid e as melhorias visuais do portfólio.

### [✓] 5.1 Refatoração Reativa: Actions e Store
*   **Concluído:** Modularização de god files concluída (incluindo `hls-player.ts`, `dispatcher.ts`, `metadataStore.ts`, `useVideoPlayer.ts`, `AdvancedSearchModal.tsx`, `Table.tsx`, `TreeView.tsx`, `Input.tsx`, `DropdownMenu.tsx`, `ContextMenu.tsx` e `Sonner.tsx`). Eliminação de `any` em stores e UI components críticos. Resolução de conflitos de atalhos (Meta+A). Refatoração do `TreeView` para componente 100% puro e agnóstico ao domínio.
   *Detalhes: `docs/plans/2026-02-23_15:09-frontend-code-quality-refactoring.md`, `docs/plans/2026-02-24_00:36-advanced-search-component-registry-architecture.md`, `docs/plans/2026-02-24_15:51-table-component-refactoring.md`, `docs/plans/2026-02-24_19:09-tree-view-refactoring.md`, `docs/plans/2026-02-24_21:40-input-component-refactoring.md`, `docs/plans/2026-02-25_02:22-dropdown-context-menu-refactoring.md` e `docs/plans/2026-02-26_14:45-refactor-simple-ui-components-batch-2.md`*
*   **Concluído Total:** Padronização completa e definitiva de patterns de `actions` exportadas dos Stores. Componentes agora são puramente UI e delegam mutações para o Core validado. Eliminação absoluta do uso de `any` em todo o projeto. Nenhuma supressão `eslint-disable` mantida injustificadamente. Viewport Engine refatorada para escalabilidade e `layout.worker` otimizado abaixo da complexidade limitadora. Pipeline estático perfeitamente limpo (Saída Zeros em Lint e Schema Check).

### [✓] 5.2 Refatoração de Componentes Base (Accordion e UI Library)
*   **Concluído:** O componente `Accordion` foi completamente refatorado para **Compound Components**. Além dele, todos os componentes base (`Badge`, `CountBadge`, `Alert`, `Separator`, `Loader`, `ProgressBar`, `SectionGroup`, `SidebarPanel`, `Sonner`) foram modularizados em pastas dedicadas, com nomenclatura descritiva e documentación TSDoc completa.
*   *Detalhes: `docs/plans/2026-02-25_01:28-refactor-accordion.md` e `docs/plans/2026-02-26_14:45-refactor-simple-ui-components-batch-2.md`*

### [ ] 5.3 Globalização e Customizações Exclusivas (UX Premium)
*   **Ação:** Integrar o ambiente arquitetural da Localização (`i18n`) para suportar instâncias multilíngue no layout Solid.
*   **Ação:** Finalizar estrutura complexa nativa de UI Customizada com suporte a **Temas Avançados** estendendo o `tokens.css` de forma que os usuários salvem paletas próprias no app e não somente limitem a "Light/Dark mode".
