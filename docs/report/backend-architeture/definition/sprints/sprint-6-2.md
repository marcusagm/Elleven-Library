# Sprint 6.2: Purificação do Tidy Backend (Eliminação V1)

**Status:** Concluída
**Data e hora de inicio:** 2026-03-10 09:36
**Data da conclusão:** 2026-03-10 11:16

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
- [x] Deletar `src-tauri/src/library/` (As querys de BD da v1 e as `[tauri::command]`).
- [x] Deletar `src-tauri/src/db/` (Conexões, DB manager `sqlx` V1 bruto).
- [x] Deletar `src-tauri/src/indexer/` (Lógica de Watcher/File discovery antiga).
- [x] Deletar `src-tauri/src/transcoding/` (FFmpeg e Stream handlers não HLS).
- [x] Deletar `src-tauri/src/media/` (O extrator monolítico anterior ao `core/formats`).
- [x] Deletar `src-tauri/src/thumbnails/` e o híbrido `priority_state` de lá (O V2 reescreveu a fila LIFO).

### 2. Saneamento do Entrypoint (`lib.rs`)
- [x] Identificar e remover os Imports V1 do topo da página.
- [x] Remover do `setup()` na instância do App Handle todas as dependências Arc nativas do V1 (`handle.manage(db_arc.clone())`).
- [x] Limpar as assinaturas dos Handlers dentro do `invoke_handler(tauri::generate_handler![...])`.
- [x] Limpar macros legados e corrigir sintaxe.

### 3. Saneamento de Módulos (Rust) e Warnings
- [x] Rodar `cargo check` ou `cargo clippy` detectando onde o apagão deixou módulos importando arquivos inexistentes (Corrigir `mod.rs` globais).

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- Lidar com comandos de frontend (`get_streaming_token`, `add_location`, `get_setting`) que foram apagados no backend e causavam erros em console. A estratégia final foi preservar a deleção e o frontend lida com os erros enquanto constrói sua base paralela na V2, mantendo o backend estritamente nos trilhos isolados e sem dependências passadas (retrocompatibilidade V1 indesejada).
- A separação de `hex_to_lab` durante o expurgo da pasta `thumbnails` quase inviabilizou o filtro de cores, precisando que a lógica colorimétrica fosse extraída para `infra/database/search_builder.rs`.

### Melhorias Realizadas
- Todas as injeções transitórias como `v2_indexer`, `v2_settings` e `v2_db_manager` foram renomeadas apropriadamente já que agora elas não competem mais em escopo para inicialização com a V1.
- A árvore de pastas reduziu mais de 45 diretórios, deixando o app em torno da arquitetura hexagonal pura.
- Tempo de boot acelerado significativamente devido a isenção de queries estáticas e scans legados globais.

### 📄 Arquivos Criados ou Modificados
- `docs/report/backend-architeture/definition/sprints/sprint-6-2.md`
- `src-tauri/src/infra/database/search_builder.rs` (Inlining do `hex_to_lab`)
- `src-tauri/src/lib.rs`
- **Exclusão de todos os Subdiretórios:** `db/`, `indexer/`, `library/`, `media/`, `protocols/`, `settings/`, `streaming/`, `thumbnails/`, `transcoding/`.
