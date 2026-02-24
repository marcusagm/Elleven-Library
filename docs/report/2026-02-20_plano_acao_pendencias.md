# Plano de Ação: Otimizações e Funcionalidades Pendentes (Mundam)

**Data:** 20 de Fevereiro de 2026
**Status:** Planejamento (parcialmente em andamento)  
**Última atualização:** 2026-02-23 — Progresso parcial na seção 5.1 (tipagem e modularização) via sprint de qualidade frontend. Ver `docs/plans/2026-02-23_15:09-frontend-code-quality-refactoring.md`.
**Baseado em:** `pendencias_consolidadas.md`

Este documento elabora um plano detalhado de implementação para todas as pendências ativas mapeadas nos relatórios recentes do projeto **Mundam**. Para manter a manutenibilidade, o plano foi segmentado em partes menores, de modo que cada tópico representa uma evolução lógica, focada em recursos específicos, melhorias de arquitetura e otimizações de performance.

---


## 2. Robustez do Backend e Pipeline de Dados

**Motivação:** Alguns gargalos e riscos de loop em processamento massivo podem degradar o app. A manutenção da verdade única sobre detecção de arquivos está defasada no indexador.

### [ ] 2.1 Integração do Processo de Detecção (UMDS)
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

### [ ] 3.3 Batch Tagging (*Operações em Massa*)
*   **Ação:** Melhorar o `MultiInspector.tsx` e integrar suporte à interface (UI) e ao SQLite para renomear, excluir, mover agrupamentos e aplicar ou revogar **Tags de milhares de instâncias simultaneamente**.

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

### [~] 5.1 Refatoração Reativa: Actions e Store *(progresso parcial — 2026-02-24)*
*   **Concluído:** Modularização de god files concluída (incluindo `hls-player.ts`, `dispatcher.ts`, `metadataStore.ts`, `useVideoPlayer.ts` e `AdvancedSearchModal.tsx`). Eliminação de `any` em stores críticos. Remoção de `console.log` de debug, rewrite do `TagDropStrategy.ts`. Build TypeScript com 0 erros.
*   **Pendente:** Adequar inteiramente os patterns de `actions` exportadas dos Stores para mutações visuais exclusivas com tipagem segura, deixando componentes puramente UI. Restam `any` apenas em `DropdownMenu.tsx`, `Input.tsx`, `TreeView.tsx`, `Table.tsx`.
*   *Detalhes: `docs/plans/2026-02-23_15:09-frontend-code-quality-refactoring.md` e `docs/plans/2026-02-24_00:36-advanced-search-component-registry-architecture.md`*



### [ ] 5.3 Globalização e Customizações Exclusivas (UX Premium)
*   **Ação:** Integrar o ambiente arquitetural da Localização (`i18n`) para suportar instâncias multilíngue no layout Solid.
*   **Ação:** Finalizar estrutura complexa nativa de UI Customizada com suporte a **Temas Avançados** estendendo o `tokens.css` de forma que os usuários salvem paletas próprias no app e não somente limitem a "Light/Dark mode".
