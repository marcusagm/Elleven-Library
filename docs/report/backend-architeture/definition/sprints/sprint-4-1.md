# Sprint 4.1: Thumbnail Worker Pool & Fila de Prioridade

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

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
- [ ] No `core/workflows/thumbnails/priority.rs` ou `feature/thumbnails/state.rs`, desenhe a struct `ThumbnailPriorityState` contendo um `Mutex<VecDeque<i64>>` que armazenará os Assets que o front gritou pedindo.
- [ ] Implementar as rotinas de `push_front()` limitadas (não permitir enfileirar mais de N IDs prioritários para evitar estourar limites se o usuário girar o mouse feito louco).

### 2. O Motor do Processo (Tokio/Rayon)
- [ ] Em `infra/workers/thumbnail_worker.rs`, orquestre o loop principal infinito ancorado por um `CancellationToken` (Para Graceful Shutdown).
- [ ] A lógica central do loop obrigatoriamente fará:
  - Condição 1: Puxar do `PriorityState` primeiro (itens LIFO da UI).
  - Condição 2: Se o `PriorityState` estiver vazio, puxe do SQLite (Query veloz `SELECT id FROM assets WHERE has_thumb = 0 LIMIT 20`).
- [ ] Distribuir o "Job" invocando a porta `FormatRegistry.resolve(asset).thumbnail().generate()`. 

### 3. Integração na Carga do App
- [ ] No `main.rs`, defina os limites (CPU Cores limit - 1) na thread pool para que a maquina do usuário não trave.
- [ ] Spawne o `ThumbnailWorker::start(...)` atrelando-o à arquitetura.

### 4. O Atalho de Demanda Tauri
- [ ] Crie e exponha o comando `#[tauri::command] pub async fn prioritize_thumbnails(ids: Vec<i64>)`. Ele deve simplesmente pegar o state injetado no método 1 e dar o append na fila de RAM.

---

## 💡 Notas para o Desenvolvedor / Agente
> Em Rust Async, misturar processamento de imagem CPU-Bound direto no `tokio::spawn` causa Starvation no runtime inteiro. Use imperativamente o `rayon::ThreadPoolBuilder` englobado por `tokio::task::spawn_blocking` ao acionar os Codecs de imagem do `FormatProvider`. Preste atenção brutal na Concorrência do banco: o Worker reporta ao Banco o fim da conversão via comandos de Update separados, e se a conversão falha por arquivo corrompido, grave o estado "ERRO" para o sistema parar de tentar extraí-lo infinitamente.
