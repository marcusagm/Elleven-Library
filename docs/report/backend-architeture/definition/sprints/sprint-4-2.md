# Sprint 4.2: FileSystem Watcher & Scan Debouncer

**Status:** Concluído ✅
**Data e hora de inicio:** 2026-03-09 13:45 
**Data da conclusão:** 2026-03-09 14:30

**Fase 4:** O Músculo Operacional (Workflows) 
**Objetivo:** Implantar o guardião do Submundo do OS. O sistema fará varreduras (Fast Scan Diff) em diretórios conhecidos pelo sistema, disparando intenções de alteração para o Ledger. Além disso, assinará escutas em Eventos Base do Mac/Windows para não precisar ficar varrendo periodicamente.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Varredura Diferencial Rápida:** Um `run_scan(folder_path)` em 10 mil imagens onde não houve edição não deve disparar comandos no Ledger, verificando Hashes/MTime com precisão veloz.
2. **Debounce Múltiplo:** Ao adicionar três arquivos simultâneos pesados num *Folder Monitorado*, a biblioteca de notify do SO envia picos de dezenas de eventos `Change`, mas nosso "Debouncer" condensa em Comandos Limpos únicos que fluirão até o Banco e UI.
3. **De Deleção e Orfanato:** Mover ou apagar uma foto real na "Finder/Explorer" do SO acorda o Debouncer e acata a exclusão no Banco sem crash transacional ou loop infinito.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Diferencial de Estado (Scan Initial)
- [x] Codificar o método em `feature/indexer/scanner.rs` (implementado como `LibraryIndexer` em `feature/library/indexer.rs`) usando a lib `walkdir` ou `ignore` iterando árvores. 
- [x] Acoplar com o lado Leitor-Otimizado (Query Handler) para carregar Cached-States(Size e MTime) pré-varredura. Comparar as datas e os tamanhos. Modificações engatilham o reigistro dos eventos `DomainEvent::AssetDiscovered` ou  `AssetModified` injetados no Fluxo.

### 2. File Watcher Ativo (Notify)
- [x] No pacote `infra/fs/watcher.rs` (implementado como `WatcherService` em `processing/watcher/sensor.rs`), armar o crate cross-platform `notify`. Instanciá-lo nos diretórios guardados pelo BD principal do app no Início da Carga.
- [x] Construir o *Debouncer Channel*. Recebe uma tonelada de eventos OS ruidosos (`tokio::sync::mpsc`) e segura uma represa de milisegundos (`tokio::time::sleep`).

### 3. Fiação para os Commands Reais
- [x] Quando o *Debouncer* soltar a lista processada (Arquivos Adicionados, Arquivos Alterados ou Removidos), ele deve empacotar nas Structs de `AssetCommand` elaboradas lá na Fase 2 e expedir para o Guardião (Asset Ledger).

### 4. Tracking do Lifecycle
- [x] Acoplar cada pasta nova englobada ao *Registry* de Watchers, para podermos Desligar (`.abort()`) a escuta em threads ativas caso o usuário desative a pasta no Frontend, impedindo Memory Leaks.

---

## 💡 Notas para o Desenvolvedor / Agente
> File Watcher de SO é notório por bugs em arquivos que os Browsers estão ainda salvando. Às vezes o evento de "*Criado*" é atirado quando o PSD tem `0 bytes`, resultando no Format-Kit acusando Corrupção. O seu **Debouncer** deve ser robusto; ao captar a intenção do evento num canal, ele idealmente aguarda que os tráfegos sobre àquele arquivo se congelem para invocar o Scan nele! O "FormatRegistry" cuidará da tipagem veloz (is_supported?).

---

## 🚀 Informações da Implementação

### Dificuldades Encontradas:
1.  **Tipagem Estrita do SQLx:** A macro `query!` do SQLx inferia tipos `Option<DateTime<Utc>>` para campos que precisavam ser interpretados como obrigatórios no Rust. Isso exigiu a utilização da sintaxe `as "field!"` para garantir a compatibilidade com o `HashMap` do scanner diferencial.
2.  **Mocking do Notify em Testes:** A struct `notify::event::attrs::Tracker` possui campos privados e não é facilmente instanciável para testes de pairing de rename sem mockar o barramento de eventos ou utilizar reflexão, o que foi mitigado com testes focados na agregação temporal.
3.  **Locks de Ambiente:** Durante a compilação, o barramento de rede/disco causou alguns locks de diretório de artefatos do cargo, exigindo acompanhamento manual dos status dos comandos.

### Melhorias Realizadas:
1.  **Event Listening Reativo:** O `LibraryIndexer` foi expandido para não apenas realizar o scan manual, mas também assinar o `EventBus` e reagir automaticamente a eventos de descoberta/rename detectados pelo `WatcherService`.
2.  **Orquestração Async Otimizada:** O loop de debouncing em `sensor.rs` foi refatorado para utilizar `tokio::select!` em vez de múltiplos canais bloqueantes, tornando o processo de "Cooldown" (mínimo de 600ms de silêncio) mais preciso.
3.  **Mapper de Erros:** Estendi o `AppError` para incluir suporte nativo a `notify::Error`, permitindo o uso do operador `?` em toda a camada de processamento de FS.

### Desvios de Escopo:
- A estrutura de pastas seguiu o padrão de camadas v2 (`processing/watcher` e `feature/library`) em vez de nomes sugeridos (`feature/indexer/scanner.rs`), para manter a consistência com o `overview.md` da arquitetura hexagonal.

---

## 📂 Arquivos Modificados
- `src-tauri/src/core/events/payloads.rs`: Adição de `FsPathRenamed`.
- `src-tauri/src/core/error/domain.rs`: Adição de `Watcher(notify::Error)`.
- `src-tauri/src/core/repository/asset.rs`: Novo método `get_all_files_comparison_data` no port.
- `src-tauri/src/infra/database/queries.rs`: Implementação SQL para o scanner diferencial.
- `src-tauri/src/processing/mod.rs`: Exposição do módulo de watcher.
- `src-tauri/src/processing/watcher/mod.rs`: Definição de sub-módulos.
- `src-tauri/src/processing/watcher/sensor.rs`: Implementação do `WatcherService` (Notify).
- `src-tauri/src/processing/watcher/debouncer.rs`: Motor de agregação de eventos OS.
- `src-tauri/src/feature/mod.rs`: Exposição da feature library.
- `src-tauri/src/feature/library/mod.rs`: Ponto de entrada da feature.
- `src-tauri/src/feature/library/indexer.rs`: Serviço central de indexação diferencial.
- `src-tauri/src/lib.rs`: Bootstrap e fiação dos serviços em paralelo (v1 + v2).
