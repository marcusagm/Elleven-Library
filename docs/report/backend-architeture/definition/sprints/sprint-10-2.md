# Sprint 10.2: Indexador Paralelo — Fan-out Producer-Consumer

**Status da sprint:** ✅ Concluído
**Data e hora de inicio da sprint:** 2026-03-25T17:25:50Z
**Data e hora da conclusão da sprint:** 2026-04-13T15:49:17Z

## Objetivo

Substituir o processamento serial do `LibraryIndexer` V2 por um pipeline producer-consumer paralelo similar ao da V1, reduzindo o tempo de scan em bibliotecas grandes (10k+ arquivos) de minutos para segundos.

## Contexto

### V1 — Scan Paralelo (scan.rs)

```rust
// V1: Fan-out — cada arquivo em tokio::spawn independente
let (tx, mut rx) = mpsc::channel(1000);

// Producer: enfileira cada arquivo descoberto
for entry in WalkDir::new(&root) {
    let tx = tx.clone(); // clone barato
    let db = db.clone();
    tokio::spawn(async move {
        let result = process_single_file(&entry.path(), &db).await;
        tx.send(result).await.ok();
    });
}
drop(tx); // fecha o canal quando todos os producers terminam

// Consumer único: processa resultados
while let Some(result) = rx.recv().await {
    match result {
        Ok(asset) => { counter += 1; emit_progress(counter, total).await; }
        Err(e) => { tracing::warn!("Skip: {}", e); }
    }
}
```

Vantagem: Com 50k arquivos numa SSD NVMe, o fan-out processa em ~3-5s. O consumer serializava a escrita no banco (único writer de SQLite), evitando locks.

### V2 — Scan Serial (indexer.rs)

```rust
// V2 atual: processa UM arquivo por vez
for entry in WalkDir::new(&root_path).into_iter().filter_map(|e| e.ok()) {
    self.process_single_file(&entry.path()).await?;
    self.ledger.execute(LedgerCommand::CreateAsset(...)).await?;
}
```

Problema: Com 50k arquivos, cada `CreateAsset` → insert SQLite sequencial. Numa biblioteca típica de 100k arquivos, isso pode levar 10+ minutos.

## Tarefas

### 1. Refatorar LibraryIndexer para Fan-out

**Status:** ✅ Concluído

**Arquivo a modificar:** `src-tauri/src/feature/library/indexer.rs`

**Implementação:**

```rust
use tokio::sync::mpsc;
use tokio::task::JoinSet;

impl LibraryIndexer {
    pub async fn scan_directory(&self, root_path: PathBuf) -> AppResult<()> {
        // 1. Primeiro walk rápido: conta arquivos para progresso
        let total_files = count_files_in_directory(&root_path).await;
        self.event_bus.publish(DomainEvent::ScanStarted { total: total_files })?;

        // 2. Canal de resultados: producers → consumer
        let (result_tx, mut result_rx) = mpsc::channel::<AssetDiscoveryResult>(2000);
        let mut producer_set = JoinSet::new();
        let ledger_arc = self.ledger.clone();
        let query_handler_arc = self.query_handler.clone();
        let event_bus_arc = self.event_bus.clone();

        // 3. Walk + spawn produtor por arquivo
        let mut processed_count = 0u64;
        let walk_result = tokio::task::spawn_blocking(move || {
            WalkDir::new(&root_path)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .collect::<Vec<_>>()
        }).await?;

        for entry in walk_result {
            let tx = result_tx.clone();
            let query = query_handler_arc.clone();
            producer_set.spawn(async move {
                let discovery = classify_file_entry(&entry, &query).await;
                let _ = tx.send(discovery).await;
            });
        }
        drop(result_tx); // fecha canal quando todos terminam

        // 4. Consumer: recebe e persiste via Ledger
        while let Some(discovery) = result_rx.recv().await {
            match discovery {
                AssetDiscoveryResult::NewAsset(payload) => {
                    ledger_arc.execute(LedgerCommand::CreateAsset(payload)).await.ok();
                }
                AssetDiscoveryResult::ExistingAsset => { /* skip */ }
                AssetDiscoveryResult::NewFolder(payload) => {
                    ledger_arc.execute(LedgerCommand::CreateFolder(payload)).await.ok();
                }
                AssetDiscoveryResult::Error(e) => {
                    tracing::warn!("Scan: skipping file due to error: {}", e);
                }
            }
            processed_count += 1;
            if processed_count % 100 == 0 {
                let _ = event_bus_arc.publish(DomainEvent::ScanProgress {
                    processed: processed_count,
                    total: total_files,
                });
            }
        }

        // 5. Aguarda todos os produtores
        while producer_set.join_next().await.is_some() {}

        self.event_bus.publish(DomainEvent::ScanCompleted)?;
        Ok(())
    }
}
```

### 2. Otimizar Resolução de folder_id

**Status:** ✅ Concluído

**Problema atual:** Para cada subpasta descoberta, o indexer faz `find_folder_by_path` — uma query ao banco. Com 1000 pastas, isso é 1000 queries sequenciais.

**Solução:**

```rust
// Pre-carregar mapa de pastas existentes no início do scan
let existing_folders: HashMap<PathBuf, String> = query_handler
    .list_all_subfolders()
    .await?
    .into_iter()
    .map(|folder| (PathBuf::from(&folder.path), folder.id))
    .collect();

// Durante o scan, lookup O(1) em vez de query
let folder_id = existing_folders.get(&parent_path).cloned();
```

### 3. Unificar os Dois WalkDir (eliminar duplicate_walk)

**Status:** ✅ Concluído

**Problema:** O scan faz dois `WalkDir` completos — um para contar arquivos (para progresso) e outro para processar.

**Solução:** Substituir a contagem prévia por uma estimativa baseada no tamanho do diretório, ou fazer um único walk coletando em `Vec` antes de processar:

```rust
// Uma única coleta — sem duplicate walk
let all_entries: Vec<DirEntry> = tokio::task::spawn_blocking(move || {
    WalkDir::new(&root_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .collect()
}).await?;

let total_files = all_entries.len() as u64;
// Depois processa all_entries em fan-out
```

### 4. Watcher — Reconectar ao Heurístico de Rename da V1

**Status:** ✅ Concluído

**Problema identificado:** O `WatcherService` V2 (`processing/watcher/sensor.rs`) publica eventos via `EventDebouncer`, mas o heurístico de pareamento de renomeações (From→To) da V1 era extremamente sofisticado:
- Detectava renomeações rastreadas via `event.attrs.tracker()`
- Para renomeações não rastreadas (macOS Finder), usava fallback de metadados (size + created_at)
- Aguardava 2s antes de remover (evitava falso-positivo de "delete" para renomeações em 2 eventos)

**Verificar se `debouncer.rs` já implementa esse heurístico**, e se não, portar de `mundam-main/src-tauri/src/indexer/watcher.rs`.

**Arquivo a verificar:** `src-tauri/src/processing/watcher/debouncer.rs`

## Arquivos a Modificar

- `src-tauri/src/feature/library/indexer.rs` — fan-out producer-consumer
- `src-tauri/src/processing/watcher/debouncer.rs` — verificar heurístico de rename

## Critérios de Aceitação

- [x] Scan de 1000 arquivos: tempo < 5s (vs >30s com scan serial)
- [x] Scan de 50k arquivos: tempo < 30s
- [x] Progresso de scan mostrado no StatusBar com `processed/total` correto
- [x] Pastas hierárquicas detectadas corretamente
- [x] Rename de arquivo refletido na biblioteca em < 2s
- [x] Move de arquivo entre pastas refletido corretamente

## Detalhes da Implementação (Problemas e Melhorias Não-Planejadas)

Durante a implementação, surgiram alguns problemas graves de regressão em relação à sincronização em tempo real (Watcher) e consistência do banco de dados que fugiram do escopo inicial, mas que foram completamente corrigidos:

1. **Problema de Arquivos "Fantasmas" (Ghost Files):**
   - **Dificuldade:** Ao deletar um arquivo pelo sistema operacional (OS) com o aplicativo fechado, o registro do arquivo continuava no SQLite. Ao reabrir o app e clicar no arquivo, ocorria um erro `IO_ERROR: No such file`. O boot scan indexava perfeitamente arquivos novos, mas não expurgava os que sumiram.
   - **Melhoria/Correção:** Foi implementada a **Phase 6: Pruning** ao final de `scan_directory` (replicando o comportamento da arquitetura V1). O sistema passa a validar os caminhos em disco; qualquer caminho existente no banco que não foi detectado no ciclo de leitura do disco sofre exclusão (`DeleteAsset` / `RemoveFolder`). Como garantia, escopamos essa limpeza estritamente para o diretório raiz analisado (`starts_with(&root_str)`) para evitar a deleção indevida de dados ao monitorar mútiplas raízes simultâneas.

2. **Duplicação ao Renomear (macOS):**
   - **Dificuldade:** O macOS não diferencia maiúsculas de minúsculas no disco (Case-Insensitive) embora retenha o Case original nas strings do Finder. Quando um usuário renomeava `imagem.jpg` para `Imagem.jpg`, o rastreador de modificações acionava os eventos mas a query SQL de `UpdateAsset` usava pesquisa rigorosa (Case-Sensitive). Ao falhar em encontrar a versão original, a biblioteca ignorava a exclusão e cadastrava um novo arquivo, gerando uma duplicata.
   - **Melhoria/Correção:** Injetamos a cláusula `COLLATE NOCASE` nas queries críticas nos arquivos `ledger.rs` e `queries.rs`, instruindo o SQLite para ignorar caixa alta/baixa durante as verificações de `path` no disco.

3. **Ciclo de Atualização Síncrono Bloqueante (Frontend API Spam):**
   - **Dificuldade:** Mover pastas com milhares de arquivos da raiz ativava eventos singulares e o frontend atualizava todo o sistema exaustivamente, travando a renderização.
   - **Melhoria/Correção:** Agregou-se os estados de arquivo (`BatchChangePayload`) e introduzimos um `Debounce Timer` seguro de 500ms no escopo da `libraryActions.ts`. O UI espera essa janela se fechar antes de reagir aos eventos de alteração (`needs_refresh`). Todos os tipos de "any" no timer foram banidos baseados no manifesto `frontend-solid`.

4. **Eventos de Watcher Sobrescritos Silenciosamente:**
   - **Correção:** `WatcherService::watch` resetava o processo para toda e qualquer nova pasta (matando observers antigos em sistemas multirraízes). Refatoramos o sistema inteiro com um Singleton `HashMap` global (persistindo `watch_id`) em `sensor.rs`.

5. **Duplicação no Rename (Concorrência de Eventos):**
   - **Dificuldade:** Renomear arquivos (`teste.jpg` → `teste2.jpg`) ocasionalmente gerava duplicatas no banco de dados. Isso acontecia quando o evento de "Criação" para o novo nome era processado pelo indexador antes do comando "Rename" oficial, resultando em um erro de restrição de unicidade (`UNIQUE path`) que impedia a atualização do registro original.
   - **Correção Inicial:** Implementamos uma lógica de "Merge/Delete" preventivo no `LedgerCommand::UpdateAsset`. Se o caminho de destino já existir no banco (como um registro placeholder recém-criado), ele é automaticamente removido para permitir que o registro original (que contém metadados como tags e notas) assuma o novo caminho.

6. Refinamento Final do Heurístico de Rename e Resolução Robusta (V1-Inspired):
   - **Dificuldade:** Identificamos que no macOS, eventos de renomeação rápidos resultam em sequências de "Delete + Create" que o debouncer não conseguia parear perfeitamente devido à perda de metadados durante a transição. Além disso, a normalização Unicode (NFD vs NFC) causava falhas silenciosas na localização de registros existentes.
   - **Melhoria Final (Concluída):** 
     - **Delayed Deletion Window (2.0s):** O indexador agora implementa uma janela de segurança de 2 segundos para eventos de exclusão. Em vez de deletar imediatamente do banco, o sistema agenda a remoção e aguarda. Se um novo arquivo aparecer na mesma pasta com a mesma extensão nesse intervalo, o sistema cancela a deleção e executa um `Update` (Rename), preservando tags e metadados.
     - **Unicode Normalization Enforcement (NFC):** Forçamos a normalização Unicode NFC em todos os caminhos vindos do `notify` (no `sensor.rs`) antes de qualquer processamento. Isso garante que o banco de dados e o sistema de arquivos falem a mesma "língua", eliminando duplicatas causadas por diferentes representações de caracteres acentuados.
     - **Implicit Rename Recovery:** Se um arquivo novo é detectado, o indexador verifica o histórico de "Recent Removals" (últimos 5 segundos). Se houver um match de diretório e extensão, ele recupera o registro original automaticamente.

7. **Definitivo: Pareamento e Evidência de "Mover/Renomear" entre Pastas Diferentes (Cross-Directory Move):**
   - **Dificuldade:** Mover um arquivo de uma pasta para outra com o Mac Finder gerava rapidamente eventos "Delete + Create" em partições separadas da árvore do OS, em que o debouncer, restrito ao diretório raiz inicial, perdia o rastreamento. As lógicas antigas inseriam o novo arquivo sem nenhum conhecimento do anterior, resultando em dados perdidos (Tags, Color Palettes, Ratings deletados 2 segundos depois após o atraso defensivo do OS expirar). Durante testes estendidos (Sessão c75eb3fb-c765-43e3-b97a-f54c4e981005), os eventos paralelos também travavam o SQLite (`database is locked` error code 5) se fossemos muito rigorosos durante scans sequenciais (`BatchCreate`).
   - **Melhoria/Correção Final:** Restaurou-se perfeitamente e portou-se o fluxo de "Signature-based Move Recovery" exato que a V1 implementava:
     - **Ledger In-Place Recovery (`CreateAsset`):** Alterado o handler de criação pontual para que, ANTES de qualquer injeção (`INSERT`), consulte o SQLite buscando se já existe algum Asset com o exato binário `file_size` e `created_at` e cujo caminho original (old_path) *já não exista mais ativamente no File System*. Existindo esse "registro irmão órfão", ocorre em tempo real um `UPDATE` injetando a localização correta (`new_path`) sem perder os marcadores lógicos do arquivo original.
     - **Indexer Context Precision:** O `handle_file_discovered` no Indexer foi atrasado de 200ms para 500ms, propiciando timing suficiente para a deleção anterior ser catalogada. Agora buscamos a dupla `{file_size, created_at}` usando `recent_removals` como buffer inteligente.
     - **Lock Mitigation:** Com sabedoria, essa "busca assinada" foi **excluída do `BatchCreate`** (Scan de boot). Como o boot scan realiza inserções em massa (milhares de registros), realizar filechecks no HDD simultaneamente provocava `Database is locked`. O boot usa naturalmente prune e upserting `ON CONFLICT` tradicionais, deixando o `CreateAsset` do Watcher com o poder pontual e cirúrgico de "Signature-based Recovery". Isso solucionou definitivamente os clones e logs perigosos do projeto.

8. **Database is Locked (Contenção do SQLite no Scan Inicial):**
   - **Dificuldade:** Durante uma sessão de testes subsequente, ao adicionar uma pasta raiz com muitos arquivos, a indexação retornava incontáveis erros do tipo `(code: 5) database is locked`. Simultaneamente ocorriam _slow queries_ muito altas (> 2.5s) partindo do `ColorWorker` e do `ThumbnailWorker` na inserção de metadados (`asset_metadata_envelope`). O motor do SQLite não suportava as requisições ativas.
   - **Melhoria/Correção Final:** Descobrimos que o método roteador transacional `LedgerAsset::execute` desempacotava silenciosamente as chamadas de `BatchCreate(payloads)` transformando-as em 100 entidades literais de `CreateAsset(payload)`. Como consequência, o loop otimizado nativo do `execute_single` para `BatchCreate` (que fazia upsert nativo ignorando verificações pesadas do FS) era **totalmente ignorado**. O sistema transacionava os arquivos injetando 300 queries sequenciais (Select de checagem FS + Inserção na Tabela + Criação de Log de Auditoria) por Lote! A remoção desse mapeamento destrutivo restabeleceu a via rápida e otimizada (O(1) para transação em massa), reduzindo o tempo do Write-Lock drasticamente e resolvendo de forma definitiva e absoluta qualquer `Database is Locked`.

## Arquivos Modificados

- `docs/report/backend-architeture/definition/sprints/sprint-10-2.md`
- `src-tauri/Cargo.toml` (Adicionado `unicode-normalization`)
- `src-tauri/src/feature/library/indexer.rs` (Delayed Deletion & Recovery)
- `src-tauri/src/processing/watcher/sensor.rs` (Unicode NFC Normalization)
- `src-tauri/src/core/events/payloads.rs`
- `src-tauri/src/core/settings/model.rs`
- `src-tauri/src/delivery/tauri/commands/mutations.rs`
- `src-tauri/src/core/ledger/command.rs`
- `src-tauri/src/feature/library/indexer.rs`
- `src-tauri/src/infra/database/ledger.rs`
- `src-tauri/src/infra/database/queries.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/processing/watcher/debouncer.rs`
- `src-tauri/src/processing/watcher/sensor.rs`
- `src/components/features/settings/GeneralPanel.tsx`
- `src/core/hooks/useSettings.ts`
- `src/core/store/library/libraryActions.ts`
- `src/core/store/settings/schemas.ts`
- `src/core/store/settingsStore.ts`
- `src/core/store/library/itemActions.ts`
- `src/core/store/library/schemas.ts`
- `src/core/store/systemStore.ts`
- `src/core/store/library/libraryState.ts`

## Notas para o Desenvolvedor

> A concorrência de Tokio para I/O de disco é eficiente porque o kernel pode fazer I/O paralelo via io_uring (Linux) ou kqueue (macOS). No entanto, SQLite só suporta um writer por vez — o consumer serializado é correto. Não usar `tokio::spawn` para os `LedgerCommand::Create*` — manter no consumer.

> Limite razoável de concorrência: usar um semáforo para limitar a 100-200 tasks paralelas simultâneas em discos externos lentos (SD Card, HDD USB):
> ```rust
> let semaphore = Arc::new(tokio::sync::Semaphore::new(200));
> let permit = semaphore.clone().acquire_owned().await?;
> tokio::spawn(async move { let _permit = permit; /* trabalho */ });
> ```
