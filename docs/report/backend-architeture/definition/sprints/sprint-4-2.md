# Sprint 4.2: FileSystem Watcher & Scan Debouncer

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

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
- [ ] Codificar o método em `feature/indexer/scanner.rs` usando a lib `walkdir` ou `ignore` iterando árvores. 
- [ ] Acoplar com o lado Leitor-Otimizado (Query Handler) para carregar Cached-States(Size e MTime) pré-varredura. Comparar as datas e os tamanhos. Modificações engatilham o reigistro dos eventos `DomainEvent::AssetDiscovered` ou  `AssetModified` injetados no Fluxo.

### 2. File Watcher Ativo (Notify)
- [ ] No pacote `infra/fs/watcher.rs`, armar o crate cross-platform `notify`. Instanciá-lo nos diretórios guardados pelo BD principal do app no Início da Carga.
- [ ] Construir o *Debouncer Channel*. Recebe uma tonelada de eventos OS ruidosos (`tokio::sync::mpsc`) e segura uma represa de milisegundos (`tokio::time::sleep`).

### 3. Fiação para os Commands Reais
- [ ] Quando o *Debouncer* soltar a lista processada (Arquivos Adicionados, Arquivos Alterados ou Removidos), ele deve empacotar nas Structs de `AssetCommand` elaboradas lá na Fase 2 e expedir para o Guardião (Asset Ledger).

### 4. Tracking do Lifecycle
- [ ] Acoplar cada pasta nova englobada ao *Registry* de Watchers, para podermos Desligar (`.abort()`) a escuta em threads ativas caso o usuário desative a pasta no Frontend, impedindo Memory Leaks.

---

## 💡 Notas para o Desenvolvedor / Agente
> File Watcher de SO é notório por bugs em arquivos que os Browsers estão ainda salvando. Às vezes o evento de "*Criado*" é atirado quando o PSD tem `0 bytes`, resultando no Format-Kit acusando Corrupção. O seu **Debouncer** deve ser robusto; ao captar a intenção do evento num canal, ele idealmente aguarda que os tráfegos sobre àquele arquivo se congelem para invocar o Scan nele! O "FormatRegistry" cuidará da tipagem veloz (is_supported?).
