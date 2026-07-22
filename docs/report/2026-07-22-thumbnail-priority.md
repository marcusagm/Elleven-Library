# 📄 Relatório de Engenharia: Refatoração da Prioridade de Geração de Thumbnails

**Data:** 22 de Julho de 2026
**Módulo:** `src-tauri/src/processing/workers/` & `core/workflows/thumbnails/priority.rs`

## 1. Contexto e Problema Original

Durante testes da V2, foi detectado que a prioridade de geração de thumbnails para os itens visíveis na viewport (UI) não estava funcionando conforme o esperado. O objetivo da aplicação é que a interface tenha sempre a mais alta prioridade, garantindo uma percepção de extrema rapidez e fluidez para o usuário — mesmo quando uma grande quantidade de arquivos está sendo indexada em background.

### Análise Causa-Raiz (Root Cause Analysis)

A investigação apontou duas falhas arquiteturais primárias no modelo anterior:

1. **Starvation por Bloqueio de Lote (Batch Blocking):**
   O `ThumbnailWorker` agrupava o processamento em blocos de 10 itens. Ele realizava o fetch desses 10 itens (prioritários ou do background), disparava 10 tarefas paralelas e usava `join_next().await` para aguardar que **todas** as 10 tarefas concluíssem antes de consultar a fila de prioridades novamente. 
   - *Impacto:* Se o worker capturasse 10 vídeos pesados, ele ficava bloqueado e cego para os pedidos da UI por vários segundos ou minutos.

2. **Inversão da Fila LIFO (LIFO Inversion):**
   A viewport (Solid.js) detecta os itens visíveis de cima para baixo (ex: `[A, B, C]`). Quando esses itens eram inseridos na fila do backend usando `push_front` em um loop iterativo direto, o array final ficava invertido (`[C, B, A]`). 
   - *Impacto:* O item do final da tela (`C`) era processado antes do primeiro item visível no topo (`A`).

3. **Demora no Feedback Visual (Batch Commit):**
   As atualizações no banco (`LedgerCommand::UpdateThumbnail`) só eram comitadas em lote após a conclusão de todas as 10 tarefas de um ciclo.
   - *Impacto:* Atritos na experiência do usuário, visto que a thumbnail de um item rápido demorava a aparecer na interface apenas por estar no mesmo lote de um arquivo demorado.

---

## 2. Solução Implementada

Para resolver esses problemas de maneira elegante e seguindo a Arquitetura Hexagonal adotada, o design do worker foi completamente reescrito para operar de forma contínua usando o padrão **Semaphore-based Continuous Pipeline**.

### 2.1. Pipeline Contínuo com Semáforo Dinâmico
- **Implementação:** Substituímos o bloqueio rígido do `join_set` por um `tokio::sync::Semaphore`.
- **Dinâmica:** O Semáforo limita o número máximo de processamentos concorrentes com base nos núcleos lógicos da CPU (`clamp(2, 8)`). O laço principal do `ThumbnailWorker` só tenta puxar novos IDs (da UI ou do Indexador) se houver um "slot" (permit) disponível no semáforo.
- **Resultado:** Assim que uma tarefa termina (por exemplo, uma imagem pequena), o slot é liberado e o worker instantaneamente vai à fila de prioridade ver se a UI pediu algo. Isso permite a intercalação imediata das demandas da UI em meio ao processamento de background pesado.

### 2.2. Graceful Finish (Preempção Suave)
- Ao invés de inserir uma complexidade insustentável de cancelamento via "tokens" que precisariam chegar nos extractors C++ e FFmpeg, adotou-se o modelo de *Graceful Finish*.
- Tarefas pesadas continuam executando até o fim sem serem mortas, porém ocupam apenas *1 slot* do semáforo. Os slots restantes continuam livres para atender a UI instantaneamente. Isso garante um código ultra limpo e isento de falhas fantasmas.

### 2.3. Correção do Padrão LIFO 
- A rotina `push_priorities` no arquivo `priority.rs` foi ajustada para fazer um loop com `.rev()`.
- *Resultado:* Se a UI manda `[A, B, C]`, o backend empurra invertido (empurra `C`, depois `B`, depois `A`), garantindo que a estrutura final no início da fila seja exatamente `[A, B, C]`, onde `A` é o item no topo da tela do usuário.

### 2.4. Commit Individual Imediato
- A função `process_batch` foi desmembrada. Agora temos o `process_single`.
- Assim que uma única thumbnail é gerada ou ignorada, o `ThumbnailWorker` despacha o `UpdateThumbnail` para o Ledger imediatamente.
- *Benefício:* Notificação instantânea (via eventos Tauri) para a VirtualList. Imagens pipocam na tela milissegundos após concluídas, trazendo a sensação "WOW" descrita nos preceitos do projeto.

---

## 3. Visão de Futuro (Em Direção ao Estado da Arte)

A solução atual consertou a pipeline base com extrema competência técnica e manutenibilidade. Para elevar a geração de Thumbnails ao estado da arte definitivo (tanto em performance bruta quanto na experiência "mágica" para o usuário), as seguintes melhorias futuras são fortemente recomendadas:

### A. Pooling e Caching de Extractors (Warm Start)
*Problema Atual:* O `webp::Encoder` e as chamadas via `FFmpeg/image-rs` criam e destroem contexto a cada execução isolada (especialmente em extrações baseadas em FFI ou Subprocessos).
*Solução Elegante:* Criar um "Object Pool" de instâncias persistentes no `FormatRegistry` que reaproveitem memória. Isso elimina o overhead de inicialização em arquivos muito curtos.

### B. Transcoding via GPU (Hardware Acceleration)
*Problema Atual:* Toda a conversão para WebP / processamento da imagem crua roda atrelada à CPU usando `tokio::task::spawn_blocking`.
*Solução Elegante:* Implementar extração de frames via aceleração de hardware (ex: `NVDEC`/`VideoToolbox` via ffmpeg) e resize gráfico com shaders (Vulkan/Metal via `wgpu`). Isso libertaria quase 100% da CPU, mantendo a responsividade do OS intocável.

### C. Abordagem "BlurHash" Imediata (Perceived Performance)
*Problema Atual:* O usuário rola a lista e vê placeholders genéricos ou "Loaders" até que o asset físico responda e a compressão WebP termine.
*Solução Elegante:* No instante em que um arquivo de imagem/vídeo for detectado durante o indexador principal de disco, podemos ler e computar os primeiros bytes em um minúsculo _BlurHash_ (string de ~20-30 caracteres) e injetar no banco. A UI (Solid.js) desenharia o canvas suavizado do blurhash num piscar de olhos, enquanto a verdadeira WebP da thumbnail roda no semáforo em background.

### D. Priorização Preditiva Dinâmica
*Problema Atual:* A priorização reage apenas ao que "cai" na tela (`visibleItems`). Se o usuário scrolla rápido, ele esvazia e enche a fila de prioridades no sobressalto.
*Solução Elegante:* Monitorar o _vetor e aceleração de Scroll_. O frontend calcula o que o usuário **vai ver** nos próximos milissegundos e sinaliza o backend preditivamente. Combinado ao LIFO perfeito, os itens chegam à tela antes mesmo do usuário encostar nelas, criando uma ilusão de latência zero ("Zero-Latency Illusion").
