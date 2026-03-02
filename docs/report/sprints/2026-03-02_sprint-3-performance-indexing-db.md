# Sprint 3: Indexing Performance & UX Flow (DB & Solid.js)

**Data:** 2026-03-02
**Status:** Concluído
**Data e hora da conclusão:** 2026-03-02T19:28:27Z

## 📌 Objetivo
Elevar a fluidez contínua do DAM (Digital Asset Manager). Tratar engarrafamentos de UI em situações de altíssima escala via estratégias defensivas: chunking em backend, escalonamento em Viewport frontend e tunning minucioso dos índices relacionais de busca do banco de dados (SQLite).

## 🛠 Tarefas de Implementação

### 1. Priority-Queue & Cache de Thumbnails
- **Escopo:** Substituir o enfileiramento linear ("FIFO burro") pelo agendamento reativo ("Just-In-Time").
- **Ações (Rust Backend):**
  - Adotar múltiplas filas controladas sob escopo local do `ThumbnailStrategy` no arquivo definitions/mod.rs. (Filas concorrenciais isoladas por tipo nativo vs pesado FFMPEG).
  - **Otimização de Render (Priorização Dinâmica):** A demanda exigida na UI deve puxar blocos para o topo da fila (LIFO comportamental por interesse de foco), ignorando momentaneamente miniaturas muito recuadas da rolagem visível.
- **Validação:** Redução comprovada do tempo até o "*first thumbnail*" (< 2s) pós-abertura de pastas congestionadas com milhares de arquivos estáticos pesados.

### 2. ViewportController Total e Schedule RAF (Frontend DOM)
- **Escopo:** Garantir a taxa ideal visual de 60fps constantes sem instabilidades estruturais reativas (quebras de signal DOM).
- **Ações (TS e Solid):**
  - Consolidar a implementação final unificada de render loop `scheduler.ts` através de RAF (Request Animation Frame) que delega posições no ecossistema do scroll da UI.
  - Utilizar arquitetura estanque isolada de Domínio (`ViewportController`) via injeção `core/store/`. Utilizar as *escape hatches* eficientes do ecosistema Solid (`untrack()`, sinais derivativos puramente atrelados a leitura passiva) para abolir componentes refazendo árvore virtual desnecessariamente durante as atualizações mecânicas no mouse wheel.
- **Validação:** Componentes da infra UI (ex. `VirtualListView.tsx`) estritamente sem regras de layout no código visual (mantidos livres de magic numbers), tipados adequadamente, isentos de *destructuring properties* dos fluxos Solid (`splitProps` mandatório).

### 3. Chunking de Interface e DB Tuning
- **Escopo:** Resolução dos gargalos identificáveis por latência alta nas tabelas que bloqueiam a UI.
- **Ações:**
  - Atrelar indexações iniciais (ou recarregamentos totais massificados) forçosamente pelo padrão de chunking assíncrono na rota do Payload (ex.: envios limitados de N entidades seguidos por *await yield* mitigados em eventos no EventBus, ao invés do bloqueio fatal aguardando o processonivel backend de 100k dados cruze a ponte do Tauri).
  - Inspecionar consultas SQLite com "queries lentas e sub-otimizadas" reportadas via métricas OTLP de Dev (Sprint 1), implementando **índices pontuais** para dar match em *ORDER BY* cruciais frequentemente usados, otimizando *JOINs* severos do banco (`tags` e `locations`).
  - Atualizar via cache nativo `cargo sqlx prepare` no hook final mitigando vazamento relacional de checagens.

### 📝 Observações Adicionais (Logs Pós-Execução)

- **Backend / SQLite:** Criamos a nova migration `20260302000000_sprint3_performance_indices_tuning.sql` com índices compostos super eficientes (`idx_asset_tags_tag_id` e `idx_assets_filename_nocase`, focando nas checagens primárias inversas de `asset_tags` e na performance de ordenação com COLLATE NOCASE sem perda relacional). Em seguida, `cargo sqlx prepare` foi executado com sucesso validando offline as queries SQLite.
- **Backend / Thumbnails:** Um dispatcher robusto usando `tokio::mpsc::channel` isolado para operações pesadas computacionais e enfileiramento LIFO (`VecDeque` e `push_front`) foram introduzidos com total sucesso (nas etapas já concluídas da implementação subjacente).
- **Frontend / RAF & UI:** O `Table.tsx` e o `VirtualListView.tsx` agora usam integralmente o ecossistema `scheduler.ts` para capturar os eventos do usuário no Mouse Wheel de forma debouncada (isolada via escape hatches SolidJS e unificação por *RequestAnimationFrame*). Componente de lista mantido inteiramente puro sem *magic numbers* com `DEFAULT_COLS` unificado e isento de leaks reativos de renderização nos *scrolls* severos. Tudo implementado no padrão Strict Type/`splitProps`. A Sprint 3 foi um sucesso de estabilidade.
- **Frontend / Lint e Arquitetura:** Refinamos amplamente componentes base como `VirtualGridView.tsx`, `VirtualMasonry.tsx`, `AssetCard.tsx` e `GeneralPanel.tsx` silenciando estritamente problemas de hooks com uso de `untrack(() => ...)` e extirpando metadados de comentários sujos, alinhando com o Compliance de Clean Code/Documentação. Lint 100% verde (`npm run lint` Exit code 0).
- **Testes Manuais / FFmpeg Logs:** Logs finais da interface demonstram total absorção da nova fila de Worker, capturando corretamente as imagens (incluindo tratamento de RAW pesados como LEICA e NIKON) via priorização FFmpeg em concorrência isolada e balanceamento correto pelo `ThumbnailWorker` unificado com `tracing::info`.
