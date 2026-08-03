## Relatório — análise da arquitetura `backend-refactor` do Mundam

### Preparação do ambiente

Atualizei o repositório local para a branch `backend-refactor` a partir de `https://github.com/marcusagm/Mundam.git`. A branch local ficou em `backend-refactor`, no commit `85c4b9e4cb2affab6ab4586b947b21d948073ee6`. Não realizei alterações em arquivos, não fiz commit e não abri PR, conforme solicitado.

## 1. Visão geral da nova arquitetura

A nova arquitetura do backend do Mundam está organizada como uma arquitetura híbrida que combina:

- **Vertical slicing por escopo**, refletido na separação física em `core`, `delivery`, `feature`, `infra`, `processing` e `lifecycle`. Essa organização está documentada como a estrutura principal do backend.
- **Arquitetura Hexagonal / Ports & Adapters**, em que o domínio declara contratos e a infraestrutura implementa detalhes concretos. O próprio documento de arquitetura descreve essa intenção de isolar o `Core` e trocar implementações como banco ou bus sem impactar a lógica de negócio.
- **EDA — Event-Driven Architecture**, com eventos de domínio publicados em um barramento baseado em `tokio::sync::broadcast`.
- **CQRS**, separando comandos de mutação, que devem passar pelo `Asset Ledger`, de queries otimizadas para leitura.

Na prática, a estrutura real do `src-tauri/src/lib.rs` confirma essa direção: a aplicação inicializa protocolos de entrega, settings, event bus, lifecycle registry, registry de formatos, banco SQLite, query handler, ledger, serviços de busca, streaming, workers, indexer e watcher em camadas relativamente bem delimitadas.

## 2. Pontos fortes da arquitetura

### 2.1 Separação clara entre domínio, infraestrutura e entrega

A separação em módulos ajuda muito a legibilidade e manutenção. O `core` concentra contratos e modelos, enquanto `infra` implementa SQLite, eventos e configuração. A documentação descreve explicitamente essa divisão como forma de combinar navegação intuitiva com baixo acoplamento.

Exemplos concretos:

- O `TransactionalAssetLedger` é um port de domínio para mutações, exposto como trait.
- O `AssetQueryHandler` é um port de leitura, também como trait assíncrona.
- A implementação concreta do ledger está em SQLite, dentro de `infra/database/ledger.rs`, mantendo a persistência fora do contrato de domínio.

**Vantagem:** isso aproxima o Mundam de sistemas DAM maduros, nos quais indexação, extração de metadados, cache, streaming e UI precisam evoluir de forma independente.

### 2.2 Asset Ledger como ponto central de mutação

O conceito de `Asset Ledger` é uma das decisões arquiteturais mais importantes. A documentação o define como ponto único de verdade para mutações de estado e disco.

A implementação atual reforça isso:

-   Cada comando entra em uma transação SQLite.
-   Eventos de domínio são publicados apenas depois do commit.
-   Operações são registradas em `asset_operations_log`, criando base para auditoria, debug e futura recuperação.

**Vantagem:** isso reduz corrupção de estado, duplicações e inconsistências entre banco, indexador, workers e UI. Para um DAM desktop offline, esse é um pilar correto.

### 2.3 CQRS parcialmente bem aplicado

A separação entre query handler e ledger está presente:

-   Queries passam por `AssetQueryHandler`, com métodos específicos para paginação, busca, contagens, tags, smart folders, thumbnails pendentes e dados de comparação para indexação diferencial.
-   Mutations passam pelo `TransactionalAssetLedger`.

**Vantagem:** essa separação permite otimizar leitura e escrita de forma diferente, algo essencial para bibliotecas grandes com milhares ou dezenas de milhares de assets.

### 2.4 Format Registry extensível e performático

O `FormatRegistry` é outro ponto forte. Ele mantém:

- lookup O(1) por extensão;
- conjunto rápido de extensões suportadas;
- fallback por magic bytes / MIME para casos ambíguos.

O fluxo de resolução tenta primeiro extensão, depois validação de bytes e fallback por MIME/magic bytes.

**Vantagem:** isso é adequado para um gerenciador profissional de assets, especialmente porque formatos de design frequentemente usam contêineres genéricos como ZIP, TIFF ou formatos proprietários.

### 2.5 Indexador com pipeline paralelo e scan diferencial

O `LibraryIndexer` demonstra avanço importante em direção a uma arquitetura escalável:

-   Usa `WalkDir` em `spawn_blocking` para percorrer a árvore sem travar o runtime assíncrono.
-   Separa diretórios e arquivos suportados antes do processamento.
-   Carrega cache de comparação para scan diferencial.
-   Usa `JoinSet`, `Semaphore` e canal MPSC para classificar arquivos em paralelo.
-   Serializa persistência em batches via `LedgerCommand::BatchCreate`.

**Vantagem:** esse desenho evita o pior caso de “um arquivo por transação” e permite controlar concorrência, algo essencial para bibliotecas grandes.

### 2.6 Watcher + indexador orientados a eventos

O indexador também possui listener de eventos para mudanças pontuais do sistema de arquivos, evitando full-rescan para cada alteração.

Ele trata eventos como:

-   arquivo descoberto;
-   path deletado;
-   path renomeado;
-   diretório descoberto;
-   diretório removido.

**Vantagem:** isso aproxima o Mundam de um comportamento “live library”, fundamental em DAMs modernos.

### 2.7 Ciclo de vida centralizado

A presença de `LifecycleRegistry` é uma decisão madura. Ele usa `CancellationToken` raiz com tokens filhos e mantém `JoinHandle`s para shutdown coordenado.

O shutdown global cancela o token raiz e aguarda cada task, com timeout e abort fallback.

**Vantagem:** isso reduz leaks de watchers, workers e servidores HTTP durante encerramento da aplicação, reinicialização de subsistemas ou testes.

### 2.8 Streaming e protocolos dedicados

A camada de delivery registra protocolos customizados `thumb`, `asset`, `video` e `audio`.

Além disso, há inicialização de servidor Axum para streaming HLS, com token de sessão e cache de transcode.

**Vantagem:** essa separação é correta para um app desktop que precisa servir previews, thumbnails e streaming de mídia sem transformar tudo em IPC pesado.

### 2.9 Observabilidade inicial

A aplicação inicializa telemetria estruturada logo no boot.

O código usa `tracing` em indexador, ledger, lifecycle e workers. Isso ainda não equivale a observabilidade completa, mas é uma base adequada.

## 3. Desvantagens e riscos da arquitetura atual

### 3.1 O `lib.rs` virou um composition root grande demais

O arquivo `src-tauri/src/lib.rs` está acumulando:

- registro de protocolos;
- setup de paths;
- settings;
- event bus;
- bridge legada para frontend;
- lifecycle;
- format registry;
- HLS manager;
- banco;
- query handlers;
- ledger;
- cache;
- streaming server;
- workers;
- FFmpeg health check;
- indexer;
- watcher;
- invoke handlers.
    

**Risco:** o composition root está correto como conceito, mas muito denso. Isso dificulta testes de bootstrap, troca de subsistemas e diagnóstico de falhas de inicialização.

**Refinamento recomendado:** extrair builders/modularizadores: **(Concluído: extração realizada para a pasta `src-tauri/src/bootstrap/` com `database.rs`, `workers.rs`, etc.)**

- `bootstrap::init_settings`
- `bootstrap::init_events`
- `bootstrap::init_database`
- `bootstrap::init_workers`
- `bootstrap::init_delivery`
- `bootstrap::init_library_services`

### 3.2 Bridge de eventos ainda carrega compatibilidade legada

O event bus publica `mundam://domain-event`, mas também traduz eventos para nomes legados como `library:batch-change`, `indexer:progress`, `thumbnail:ready`, `metadata:ready` e `extraction:completed`.

**Risco:** isso mantém o frontend acoplado a dois modelos de eventos: o novo domínio e eventos legados. A curto prazo é pragmático; a médio prazo cria duplicidade, comportamento implícito e risco de refresh excessivo.

**Refinamento recomendado:** definir um contrato público de eventos versionado para o frontend, por exemplo:

TypeScript

```
type FrontendEvent =  | { type: 'asset.created'; payload: ... }  | { type: 'scan.progress'; payload: ... }  | { type: 'thumbnail.generated'; payload: ... }
```

Depois, depreciar gradualmente os eventos legados.

### 3.3 Event Bus baseado em broadcast é simples, mas não durável

O `TokioEventBus` usa `broadcast` com capacidade fixa de 2048 eventos.

O próprio código reconhece que consumidores lentos podem receber erro `Lagged`.

**Risco:** para UI e notificações voláteis, isso é aceitável. Para workflows importantes, como fila de thumbnail, extração, reindexação e sincronização com disco, eventos perdidos podem causar estado incompleto.

**Refinamento recomendado:** separar os tipos de eventos em três classes:

1. **Eventos de UI voláteis** — broadcast atual é suficiente.
2. **Eventos de workflow** — precisam de fila persistente ou tabela `jobs`.
3. **Eventos de auditoria** — devem ir para log/event store ou tabela de operações.

### 3.4 O Ledger ainda mistura responsabilidade de domínio e implementação SQL

O `SqliteAssetLedger` tem pontos fortes, mas concentra muita lógica concreta:

- queries SQL;
- detecção de move baseada em assinatura;
- logging de operações;
- emissão de eventos;
- normalização de paths;
- regras de atualização de assets.

**Risco:** com crescimento do número de comandos, o arquivo tende a virar um “god adapter”. Isso dificulta testes unitários por comando e torna regressões mais prováveis.

**Refinamento recomendado:** manter o `SqliteAssetLedger` como orquestrador transacional, mas extrair handlers internos por comando: **(Concluído: os handlers foram extraídos para `src-tauri/src/infra/database/handlers/` separando a lógica de `asset_handler.rs`, `folder_handler.rs`, etc.)**

- `asset_create_handler.rs`
- `asset_update_handler.rs`
- `folder_handler.rs`
- `tags_handler.rs`
- `metadata_handler.rs`
- `smart_folder_handler.rs`
- `thumbnail_handler.rs`
- `smart_tag_handler.rs`

### 3.5 Atomicidade entre banco e sistema de arquivos ainda é uma promessa maior que a implementação visível

O port declara que o ledger garante atomicidade entre banco e operações de filesystem.

Porém, pelo trecho analisado, muitas operações são transacionais no banco, mas a atomicidade real com o filesystem é mais complexa. SQLite não consegue fazer transação atômica com disco externo sem estratégia adicional.

**Risco:** comandos como move, delete físico, rename, geração de thumbnails e atualização de metadata podem deixar resíduos se uma etapa de disco falhar após commit ou vice-versa.

**Refinamento recomendado:** adotar padrão de **saga local / outbox**: **(Concluído: `saga_recovery.rs` implementado em `infra/database/`)**

- registrar intenção no banco;
- executar operação de filesystem;
- marcar status;
- permitir retry/compensação;
- emitir evento somente depois do estado final.

### 3.6 Indexador ainda pode sofrer pressão de memória em bibliotecas muito grandes

O scan atual coleta todos os `DirEntry` em um `Vec` antes de processar.

Depois cria coleções para arquivos, diretórios, paths verificados e caches.

**Risco:** para bibliotecas com centenas de milhares ou milhões de arquivos, o consumo de memória pode crescer muito. Além disso, a UI só recebe progresso útil depois que parte substancial da enumeração já ocorreu.

**Refinamento recomendado:** migrar para pipeline streaming:

- produtor percorre diretórios e envia entries para canal;
- consumidores classificam;
- agregador faz batches;
- contador/progresso usa estimativa ou duas fases opcionais.

### 3.7 Reconhecimento de rename/move baseado em tamanho e created\_at pode gerar falso positivo

O indexador tenta recuperar rename/move via `recent_removals`, comparando tamanho e `created_at`, com tolerância de 2 segundos.

O ledger também tenta recuperação por `file_size + created_at` quando encontra caminho antigo inexistente.

**Risco:** arquivos diferentes podem compartilhar tamanho e timestamps próximos, especialmente exports em lote, arquivos copiados ou assets gerados por ferramenta. Isso pode preservar tags/metadata de forma incorreta em outro arquivo.

**Refinamento recomendado:** introduzir fingerprint em camadas:

1. barato: tamanho + mtime + created\_at;
2. intermediário: hash parcial do início/fim;
3. forte: hash completo assíncrono, cacheado e calculado em background.

### 3.8 Alguns workers não parecem completamente registrados no lifecycle

O thumbnail worker é registrado no `LifecycleRegistry`.

O color worker é iniciado logo depois, mas pelo trecho visto não há registro explícito no lifecycle.

**Risco:** tasks órfãs no shutdown, especialmente se o worker usa event subscriptions ou loops assíncronos internos.

**Refinamento recomendado:** todos os workers e listeners devem receber `CancellationToken` e ser registrados no lifecycle, incluindo:

- color worker;
- event bridge frontend;
- indexer event listener;
- boot scan;
- HLS cleanup worker.

### 3.9 Contratos de IPC ainda estão centralizados em `generate_handler!`

O `invoke_handler` tem uma lista extensa de comandos Tauri.

**Risco:** isso vira gargalo de organização conforme o app cresce. Também fica difícil versionar comandos, gerar documentação de API interna e validar compatibilidade frontend/backend.

**Refinamento recomendado:** agrupar comandos por domínio e gerar documentação/contratos TypeScript:

- `assets`
- `folders`
- `tags`
- `smartFolders`
- `streaming`
- `settings`
- `maintenance`

## 4. Pontos críticos que precisam ser refinados

### Prioridade P0 — estabilidade e corretude

1. **Corrigir testes de frontend quebrados** 
    - Os testes de `stream-utils` estão divergentes do comportamento atual das URLs. 
    - Isso indica contrato quebrado ou testes obsoletos. 
    - Como streaming é parte central da experiência, esse ponto deve ser resolvido antes de evoluções grandes.

2. **Definir contrato único de URL/protocolo para asset, audio, video e HLS** 
    - Hoje os testes esperam `audio://` e `audio-stream://`, mas o código retorna `asset://` em alguns cenários. 
    - A arquitetura precisa declarar claramente:
        - quando usar protocolo customizado;
        - quando usar HTTP HLS;
        - como autenticar;
        - como lidar com cache;
        - como expirar sessões.

3. **Separar evento volátil de evento confiável** 
    - O `broadcast` atual é bom para UI, mas insuficiente para workflows críticos sob carga. 
    - Introduzir uma job queue persistente para thumbnails, colors, metadata, waveform e transcode.

4. **Garantir lifecycle completo para todos os processos** 
    - Nenhum worker, listener ou task de boot deve ficar fora do `LifecycleRegistry`.
        
5. **Revisar atomicidade entre filesystem e banco** 
    - O ledger é transacional para SQLite, mas operações reais de disco precisam de saga/outbox.

### Prioridade P1 — performance e escalabilidade

1. **Transformar o indexador em pipeline streaming** 
    - Evitar coletar toda a árvore em memória. 
    - Melhorar progresso real. 
    - Reduzir latência até o primeiro asset aparecer.

2. **Melhorar fingerprint de assets** 
    - Evitar falso positivo em rename/move. 
    - Criar tabela de fingerprints incrementais.

3. **Tornar o scheduler de background explícito** 
    - Hoje thumbnail worker tem fila híbrida LIFO/FIFO. 
    - Próximo passo: scheduler central com prioridades, retry, backoff, cancelamento, deduplicação e persistência.

4. **Criar camada de cache formal** 
    - Thumbnails, previews, waveforms, HLS e metadados derivados devem compartilhar política:
        - tamanho máximo;
        - TTL;
        - invalidação;
        - reconstrução;
        - verificação de integridade.

5. **Observabilidade com métricas** 
    - Além de logs, adicionar métricas:
        - tempo médio de scan;
        - throughput de arquivos/s;
        - fila de thumbnails;
        - falhas por provider;
        - tempo de query;
        - locks SQLite;
        - uso de cache.

### Prioridade P2 — arquitetura de produto e “estado da arte”

1. **Plugin system real para formatos** 
    - O `FormatRegistry` já abre caminho, mas providers ainda parecem compilados no binário. 
    - Estado da arte seria permitir providers opcionais/externos, feature flags ou carregamento modular.

2. **Busca avançada com índice dedicado** 
    - SQLite pode atender bem no começo, mas DAMs avançados usam índices especializados. 
    - Avaliar:
        - SQLite FTS5 para texto;
        - Tantivy para busca local;
        - embeddings locais para busca semântica;
        - índice por cores/formas/metadados.

3. **Pipeline de análise semântica** 
    - Para chegar ao estado da arte:
        - tags automáticas;
        - classificação por tipo visual;
        - OCR;
        - detecção de duplicatas;
        - similaridade visual;
        - busca por paleta/cor;
        - busca por imagem de referência.

4. **Sistema de jobs persistente e painel de operações** 
    - Usuário profissional precisa ver:
        - o que está processando;
        - o que falhou;
        - o que foi cancelado;
        - reprocessar lote;
        - pausar/resumir indexação.

5. **Modelo de biblioteca versionado** 
    - Migrations existem via SQLx.
    - Próximo nível: versionar também cache, fingerprints, providers e schema de metadata.

## 5. Comparação com sistemas semelhantes de alto nível

Para se aproximar de DAMs e asset managers modernos, o Mundam precisa consolidar estes blocos:

| Área            | Estado atual                   | Próximo nível                                                  |
| --------------- | ------------------------------ | -------------------------------------------------------------- |
| Indexação       | Paralela, diferencial, watcher | Pipeline streaming, persistência de jobs, fingerprints fortes  |
| Eventos         | Broadcast em memória           | Event bus dividido entre UI volátil e jobs duráveis            |
| Formatos        | Registry extensível por traits | Plugins/providers modulares e test suite por formato           |
| Thumbnails      | Worker híbrido LIFO/FIFO       | Scheduler central com retry, dedupe, prioridade e cancelamento |
| Streaming       | Protocolos + Axum HLS          | Contrato unificado, cache policy, métricas, fallback robusto   |
| Metadata        | Extração técnica por provider  | Metadata schema versionado + semântica + OCR/AI                |
| Busca           | Queries SQLite/advanced search | FTS, índices facetados, similaridade visual/semântica          |
| Observabilidade | Tracing inicial                | Métricas, tracing spans, diagnósticos in-app                   |
| Consistência    | Ledger transacional no DB      | Saga/outbox para DB + filesystem + cache                       |
| UX de operação  | Eventos para UI                | Painel de jobs, erros recuperáveis, retry manual               |

## 6. Roadmap recomendado

### Fase 1 — estabilização da refatoração

- [x] Corrigir/atualizar testes de `stream-utils`. (Concluído)
- [x] Documentar contrato definitivo de URLs e streaming. (Concluído: `streaming-contracts.md`)
- [x] Registrar todos os workers/listeners no lifecycle. (Concluído)
- [x] Extrair bootstrap de `lib.rs`.
- [ ] Criar testes de integração para:
    - [ ] scan inicial;
    - [ ] rename;
    - [ ] move;
    - [ ] delete;
    - [ ] thumbnail generation;
    - [ ] eventos emitidos.

### Fase 2 — robustez operacional

- [ ] Criar tabela de jobs persistente.
- [ ] Migrar thumbnail/color/metadata/transcode para jobs duráveis.
- [ ] Introduzir retry/backoff/cancelamento.
- [x] Implementar outbox para eventos críticos.
- [ ] Implementar fingerprint em camadas.

### Fase 3 — performance em bibliotecas grandes

- [ ] Trocar scan “coleta tudo antes” por streaming pipeline.
- [ ] Adicionar benchmarks com bibliotecas sintéticas:
    - [ ] 10k assets;
    - [ ] 100k assets;
    - [ ] 1M paths;
    - [ ] árvore profunda;
    - [ ] muitos arquivos pequenos.
- [ ] Medir tempo de boot scan, memory peak, locks SQLite e throughput.

### Fase 4 — estado da arte em DAM

- [ ] FTS5/Tantivy para busca textual.
- [ ] Índice facetado para filtros.
- [ ] Busca por similaridade visual.
- [ ] Detecção de duplicados.
- [ ] OCR.
- [ ] Tags automáticas.
- [ ] Provider SDK para novos formatos.
- [ ] Painel de jobs e saúde da biblioteca.

## 7. Conclusão executiva

A branch `backend-refactor` representa uma evolução arquitetural real e positiva. A direção geral está correta: `Core` com contracts, `Infra` com adapters, `Feature` com use cases, `Processing` com workers e `Delivery` com Tauri/protocolos/streaming. O `Asset Ledger`, o `Format Registry`, o pipeline paralelo do indexador e o `LifecycleRegistry` são decisões maduras e alinhadas com um DAM desktop moderno.

O principal risco não está na direção, mas na consolidação: algumas partes ainda parecem em transição entre arquitetura nova e legado. Os pontos mais urgentes são contrato de streaming/URLs, confiabilidade dos eventos, persistência de jobs, lifecycle completo e atomicidade real entre banco, filesystem e cache.

Se esses pontos forem refinados, a arquitetura tem base suficiente para escalar para um Mundam competitivo com sistemas profissionais de organização de assets, especialmente se evoluir para busca semântica, análise visual, jobs duráveis e providers de formatos mais modulares.

## 7. Checks executados

- ✅ `git remote add origin https://github.com/marcusagm/Mundam.git || git remote set-url origin https://github.com/marcusagm/Mundam.git && git fetch origin backend-refactor && git checkout -B backend-refactor origin/backend-refactor && git status --short`
- ✅ `npm run typecheck`
- ❌ `npm test -- --runInBand` — falhou porque o Vitest não reconhece a opção Jest `--runInBand`.
- ❌ `npm test` — falhou em 3 testes de `src/lib/stream-utils.spec.ts`, com divergência entre URLs esperadas e URLs geradas.
- ⚠️ `cargo test` em `/workspace/Mundam/src-tauri` — falhou por limitação de ambiente: dependência sistêmica `glib-2.0` ausente via `pkg-config`.
- ✅ `git status --short && git rev-parse --abbrev-ref HEAD && git rev-parse HEAD` — confirmou branch `backend-refactor`, commit `85c4b9e4cb2affab6ab4586b947b21d948073ee6`, sem alterações locais.
