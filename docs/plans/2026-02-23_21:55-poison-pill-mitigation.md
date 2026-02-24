# Plano de Mitigação: Erros no Worker de Thumbnails (*Poison Pill*)

**Data/Hora:** 2026-02-23 21:55  
**Objetivo:** Resolver o problema de "Busy Looping" e possíveis crashes infinitos causados por arquivos corrompidos ("Poison Pills") que encerram forçadamente as threads do visualizador / extrator sem permitir ao DB registrar a tentativa.

## Contexto e Problema

O `ThumbnailWorker` utiliza um ThreadPool do `Rayon` para a processamento massivo de *assets* para a geração de thumbnails. Existiam dois vetores de erro preocupantes:

1. **Cycle Passivo (Busy Looping):** Arquivos não suportados perfeitamente ou que retornam falha em milissegundos causavam um consumo intenso de IO/Database, esgotando todo o lote numa fração de segundo e reiniciando a máquina recursivamente.
2. **Crash Loop (Poison Pill):** Arquivos seriamente defeituosos ou não comportados que invocam *panics* em dependências C/C++ ou que esgotam a memória RAM num instante causam o `kill` do processo. Como a tentativa só era incrementada *após* a falha (se capturada no erro), reabrir a aplicação colocava esse arquivo problemático novamente na fila no mesmo instante, induzindo a um reinício de crash infinito.

## Solução Adotada (Option C + Backoff Progressivo)

A solução unificada implementada seguiu duas disciplinas principais: pre-commit e backoff local condicional.

### 1. Pre-Incrementation (Tratamento Pré-Voo)

No arquivo `src-tauri/src/db/images.rs`:
- Criada a função `increment_thumbnail_attempts_batch` dedicada a iterar todos os ids na esteira via `IN (.., .., ..)`.
- Requerida antes mesmo do despache para o bloco pesando do CPU `rayon` as tentativas de cada imagem daquele turno de batch são incrementadas massivamente.
- Caso ocorra um *Crash Fatal* e o app seja fechado forçado, a tentativa estará somada; ou seja, na 3º abertura o sistema pula esse arquivo nativamente evitando loops mortais.
- Em cenários perfeitos (Thumbnail finalizada e path atualizado via `update_thumbnail_path`), essa tentativa é convertida novamente a `0` como se ela estivesse imaculada.

### 2. Backoff Dinâmico por Percentual de Erro

No arquivo `src-tauri/src/thumbnails/worker.rs`:
- Implantadas linhas de estatísticas de contagem de falhas processados dentro de um mesmo Lote (Batch).
- Se a conta exceder *50%* de erro do lote inteiro, o Worker percebe o estresse das dependências geradoras ou um caminho mal-sucedido montado (sem permissões) e atrasa o reshare para `5s` em vez de `100ms`.
- Se a conta for de *100%* (desastre generalizado na leitura como disconnects USB drásticos), a pausa atinge pesados `10s` entre os lotes até estabilização natural, poupando bateria/CPU do device.

## Arquivos Modificados
- `src-tauri/src/db/images.rs`: Novos métodos `increment_thumbnail_attempts_batch`, adequações reset das tabelas nas queries e alteração de nomeação analítica.
- `src-tauri/src/thumbnails/worker.rs`: Lógica na esteira de extração de pre-increment list, backoff delays explícitos pós processamento de logs do DB.
