# Sprint 6.2: Purificação do Tidy Backend (Eliminação V1)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 6:** Cleanup e Consolidação V2
**Objetivo:** Após homologar totalmente os Comandos do Tauri e Frontend V2, executar a supressão e exclusão física das dependências "Lixo" do sistema legado.

---

## 🎯 Critérios de Aceite
1. Todo o código V1 obsoleto não existe mais (`src-tauri/src/library/`, `db/`, `media/`, `indexer/`, `transcoding/`, `thumbnails/`).
2. O arquivo `lib.rs` foi limpo de todas as instâncias de injeções V1 (`watcher_registry`, `config_state`, managers antigos, builders obsoletos).
3. A Árvore de Builds do Rust está magra. Sem warnings de compilação `"unused import"` no final do Cargo Check / Cargo Build.
4. A aplicação Boota limpa, exclusivamente através do pipeline assíncrono TokioEventBus V2.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Remoções Cirúrgicas
- [ ] Deletar `src-tauri/src/library/` (As querys de BD da v1 e as `[tauri::command]`).
- [ ] Deletar `src-tauri/src/db/` (Conexões, DB manager `sqlx` V1 bruto).
- [ ] Deletar `src-tauri/src/indexer/` (Lógica de Watcher/File discovery antiga).
- [ ] Deletar `src-tauri/src/transcoding/` (FFmpeg e Stream handlers não HLS).
- [ ] Deletar `src-tauri/src/media/` (O extrator monolítico anterior ao `core/formats`).
- [ ] Deletar `src-tauri/src/thumbnails/` e o híbrido `priority_state` de lá (O V2 reescreveu a fila LIFO).

### 2. Saneamento do Entrypoint (`lib.rs`)
- [ ] Identificar e remover os Imports V1 do topo da página.
- [ ] Remover do `setup()` na instância do App Handle todas as dependências Arc nativas do V1 (`handle.manage(db_arc.clone())`).
- [ ] Limpar as assinaturas dos Handlers dentro do `invoke_handler(tauri::generate_handler![...])`.
- [ ] Limpar macros legados e corrigir sintaxe.

### 3. Saneamento de Módulos (Rust) e Warnings
- [ ] Rodar `cargo check` ou `cargo clippy` detectando onde o apagão deixou módulos importando arquivos inexistentes (Corrigir `mod.rs` globais).

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- `docs/report/backend-architeture/definition/sprints/sprint-6-2.md` (Tracker)
- Múltiplas Deleções

---

## 💡 Notas para o Desenvolvedor / Agente
> Tenha cuidado de checar em *lib.rs* de não remover acidentalmente o Handshake do V2! A purga deve ser metodicamente executada nas pastas declaradamente obsoletas. Confirme via Cargo Build após cada deleção!
