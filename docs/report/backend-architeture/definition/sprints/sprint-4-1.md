# Sprint 4.1: Thumbnail Worker Pool & Fila de Prioridade

**Status:** Concluído ✅
**Data e hora de inicio:** 2026-03-09T06:29:34Z
**Data da conclusão:** 2026-03-09T13:45:00Z

**Fase 4:** O Músculo Operacional (Workflows) 
**Objetivo:** Implementar o motor de background do sistema. Este worker consumirá o *Format-Kit Registry* para gerar miniaturas continuamente. O foco crítico desta sprint é a implementação de um sistema sensível à interface: uma Fila de Prioridade (LIFO) que atropela a fila de background (FIFO) quando o usuário rola a galeria.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Background Indexing (FIFO):** O `JobScheduler` puxa do banco os arquivos recém-coletados que ainda não possuem thumbnail (coluna nula) e processa silenciosamente limitando a 2 ou 4 threads para não asfixiar o SO.
2. **Prioridade Visual (LIFO On-Demand):** Ao enviar um comando (via Tauri `invoke`) contendo os IDs das imagens que acabaram de brotar na tela do React/Solid.js, o `Worker` paralisa/pausa novos itens da fila lenta, engole esses IDs prioritários em memória RAM, e gera as thumbs deles imediatamente, devolvendo Eventos pro Front.
3. **Escrita Imune e Feedback:** A cada Thumb gerada com sucesso, um evento de conclusão sai pelo `EventBus` (`ThumbGeneratedEvent`) avisando o sistema de que o Cache X está pronto no disco, atualizando a coluna no banco via Handler do `Ledger`.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Sistema Filas Híbridas (State)
- [x] No `core/workflows/thumbnails/priority.rs` ou `feature/thumbnails/state.rs`, desenhe a struct `ThumbnailPriorityState` contendo um `Mutex<VecDeque<i64>>` que armazenará os Assets que o front gritou pedindo.
- [x] Implementar as rotinas de `push_front()` limitadas (não permitir enfileirar mais de N IDs prioritários para evitar estourar limites se o usuário girar o mouse feito louco).

### 2. O Motor do Processo (Tokio/Rayon)
- [x] Em `infra/workers/thumbnail_worker.rs`, orquestre o loop principal infinito ancorado por um `CancellationToken` (Para Graceful Shutdown).
- [x] A lógica central do loop obrigatoriamente fará:
  - Condição 1: Puxar do `PriorityState` primeiro (itens LIFO da UI).
  - Condição 2: Se o `PriorityState` estiver vazio, puxe do SQLite (Query veloz `SELECT id FROM assets WHERE has_thumb = 0 LIMIT 20`).
- [x] Distribuir o "Job" invocando a porta `FormatRegistry.resolve(asset).thumbnail().generate()`. 

### 3. Integração na Carga do App
- [x] No `main.rs`, defina os limites (CPU Cores limit - 1) na thread pool para que a maquina do usuário não trave.
- [x] Spawne o `ThumbnailWorker::start(...)` atrelando-o à arquitetura.

### 4. O Atalho de Demanda Tauri
- [x] Crie e exponha o comando `#[tauri::command] pub async fn prioritize_thumbnails(ids: Vec<i64>)`. Ele deve simplesmente pegar o state injetado no método 1 e dar o append na fila de RAM.

---

## 💡 Notas para o Desenvolvedor / Agente
> Em Rust Async, misturar processamento de imagem CPU-Bound direto no `tokio::spawn` causa Starvation no runtime inteiro. Use imperativamente o `rayon::ThreadPoolBuilder` englobado por `tokio::task::spawn_blocking` ao acionar os Codecs de imagem do `FormatProvider`. Preste atenção brutal na Concorrência do banco: o Worker reporta ao Banco o fim da conversão via comandos de Update separados, e se a conversão falha por arquivo corrompido, grave o estado "ERRO" para o sistema parar de tentar extraí-lo infinitamente.

---

## 🛠️ Informações da Implementação

### Dificuldades Encontradas
- **Tratamento de Macro Borrowing:** O uso de `sqlx::query!` com argumentos resultantes de casts ou expressões inline (ex: `limit as i64`) causou erros de "temporary value dropped while borrowed". Resolvido vinculando o valor a uma variável de escopo superior antes da macro.
- **Ciclo de Vida de Tarefas (JoinHandle):** A transição do worker legado (baseado em Tokio puro) para o novo worker integrado exigiu cuidado na definição do tipo de retorno de `JoinHandle` para o gerenciador de ciclo de vida do Tauri.
- **Concorrência de Estados:** Foi necessário separar logicamente os `ThumbnailPriorityState` da V1 e V2 para evitar conflitos de tipos e garantir que o comando Tauri roteasse para o motor correto durante a fase de transição (Strangler Fig pattern).

### Melhorias Realizadas
- **Rayon Integration:** Implementada uma Thread Pool dedicada do Rayon dentro do worker para garantir que o processamento pesado de imagens não bloqueie o reactor principal do Tokio.
- **Audit Logging:** Integrado ao sistema de logs de operações do `Ledger`, garantindo que cada atualização de miniatura seja rastreável e auditável.
- **Robustez na Resolução de Formatos:** O worker agora utiliza o `FormatRegistry` completo, permitindo suporte automático a qualquer novo formato adicionado ao sistema sem alteração no loop do worker.

### Pontos Fora do Escopo Inicial
- **Atualização Massiva de Projeções:** Devido à mudança no esquema (adição de `thumbnail_path`), foi necessário atualizar quase todas as queries de projeção no `ledger.rs` e `queries.rs` para manter a consistência do modelo `AssetDb`.

---

## 📂 Arquivos Modificados / Criados

- **Core/Domain:**
  - `src-tauri/src/core/workflows/thumbnails/priority.rs` (Novo/Implementado)
  - `src-tauri/src/core/ledger/command.rs` (Adicionado `UpdateThumbnail`)
  - `src-tauri/src/core/models/asset.rs` (Adicionado `thumbnail_path`)
- **Infrastructure:**
  - `src-tauri/migrations/20260309000000_sprint_4_1_thumbnail_path.sql` (Nova)
  - `src-tauri/src/infra/database/models.rs` (Atualizado `AssetDb`)
  - `src-tauri/src/infra/database/ledger.rs` (Implementado `UpdateThumbnail` + Projeções)
  - `src-tauri/src/infra/database/queries.rs` (Adicionado `get_assets_needing_thumbnails` + Projeções)
- **Processing/Workers:**
  - `src-tauri/src/processing/workers/thumbnail_worker.rs` (Novo/Implementado)
  - `src-tauri/src/processing/mod.rs` (Exposto `workers`)
- **Delivery/Tauri:**
  - `src-tauri/src/delivery/tauri/thumbnails.rs` (Novo/Implementado)
  - `src-tauri/src/lib.rs` (Orquestração e Registro)
- **Documentação:**
  - `docs/report/backend-architeture/definition/sprints/sprint-4-1.md` (Atualizado)
