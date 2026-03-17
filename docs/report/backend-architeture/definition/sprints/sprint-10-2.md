# Sprint 10.2: Indexador Paralelo — Fan-out Producer-Consumer

**Status da sprint:** Pendente
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

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

**Status:** Pendente

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

**Status:** Pendente

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

**Status:** Pendente

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

**Status:** Pendente

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

- [ ] Scan de 1000 arquivos: tempo < 5s (vs >30s com scan serial)
- [ ] Scan de 50k arquivos: tempo < 30s
- [ ] Progresso de scan mostrado no StatusBar com `processed/total` correto
- [ ] Pastas hierárquicas detectadas corretamente
- [ ] Rename de arquivo refletido na biblioteca em < 2s
- [ ] Move de arquivo entre pastas refletido corretamente

## Notas para o Desenvolvedor

> A concorrência de Tokio para I/O de disco é eficiente porque o kernel pode fazer I/O paralelo via io_uring (Linux) ou kqueue (macOS). No entanto, SQLite só suporta um writer por vez — o consumer serializado é correto. Não usar `tokio::spawn` para os `LedgerCommand::Create*` — manter no consumer.

> Limite razoável de concorrência: usar um semáforo para limitar a 100-200 tasks paralelas simultâneas em discos externos lentos (SD Card, HDD USB):
> ```rust
> let semaphore = Arc::new(tokio::sync::Semaphore::new(200));
> let permit = semaphore.clone().acquire_owned().await?;
> tokio::spawn(async move { let _permit = permit; /* trabalho */ });
> ```
