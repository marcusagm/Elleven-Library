# Sprint 8.4: Validação E2E, Compilação Limpa e Relatório Final de Migração

**Status:** Pendente
**Data e hora de inicio:** -  
**Data da conclusão:** -

**Fase 8:** Paridade IPC — Mídia, Manutenção e Utilidades
**Objetivo:** Validação completa end-to-end do backend V2 contra todos os requisitos do V1. Garantir compilação limpa (`cargo build` + `cargo clippy`), verificar integração frontend-backend para todos os 53+ IPC commands, e gerar o relatório final de migração.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. `cargo build --release` compila com 0 errors e 0 warnings.
2. `cargo clippy` passa sem warnings (exceto os allow listados).
3. `cargo sqlx prepare` gera queries verificadas sem erros.
4. Todos os IPC commands registrados no `invoke_handler` do `lib.rs` correspondem a funções existentes.
5. O frontend inicia e renderiza a galeria com assets, thumbnails e metadados.
6. Tags CRUD funciona end-to-end na UI.
7. Smart Folders CRUD funciona end-to-end na UI.
8. Reprodução de vídeo/áudio funciona via streaming server.
9. Indexação e re-indexação de pastas funciona pelo botão da UI.
10. Graceful shutdown encerra todos os workers sem corrupção de dados.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Inventário Final de IPC Commands
- [ ] Listar todos os IPC commands registrados no `lib.rs` V2 e comparar contra a tabela definida no `architecture-comparison-report.md`.
- [ ] Confirmar que todos os 53 equivalentes do V1 estão presentes (migrados ou com equivalente V2).
- [ ] Documentar qualquer comando V1 intencionalmente excluído e justificar (ex: `get_location_root_counts` era stub vazio no V1).

### 2. Compilação Limpa
- [ ] Executar `cargo build --release` e resolver quaisquer warnings.
- [ ] Executar `cargo clippy -- -W clippy::all` e resolver issues.
- [ ] Executar `cargo sqlx prepare` para validar queries compiladas.

### 3. Testes de Integração Frontend-Backend
- [ ] Iniciar o app com `cargo tauri dev`.
- [ ] Testar cada grupo funcional:

**Galeria e Assets:**
- [ ] Verificar listagem de assets com paginação
- [ ] Verificar filtros por família (Image, Video, 3D, etc.)
- [ ] Verificar busca textual

**Tags:**
- [ ] Criar uma tag nova
- [ ] Editar nome/cor de uma tag
- [ ] Aplicar tag a um asset
- [ ] Remover tag de um asset
- [ ] Deletar uma tag
- [ ] Batch: aplicar tag a múltiplos assets

**Folders:**
- [ ] Adicionar uma nova location (pasta)
- [ ] Verificar scan automático
- [ ] Remover uma location
- [ ] Navegar por subfolders
- [ ] Verificar contadores por pasta

**Smart Folders:**
- [ ] Criar smart folder com query
- [ ] Editar smart folder
- [ ] Deletar smart folder

**Rating e Notes:**
- [ ] Atribuir rating a um asset
- [ ] Escrever notes em um asset

**Inspector/Metadata:**
- [ ] Visualizar dados EXIF no painel de propriedades
- [ ] Visualizar paleta de cores

**Streaming:**
- [ ] Reproduzir vídeo MP4 nativo
- [ ] Reproduzir vídeo que requer transcoding (se FFmpeg disponível)
- [ ] Reproduzir áudio

**Thumbnails:**
- [ ] Verificar geração automática de thumbnails para novos assets
- [ ] Solicitar regeneração de thumbnail
- [ ] Verificar priorização de thumbnails visíveis

### 4. Teste de Graceful Shutdown
- [ ] Iniciar app, esperar indexação de pelo menos 100 arquivos, fechar app.
- [ ] Verificar nos logs que todos os workers (thumbnail, watcher, HLS, streaming) encerraram limpo.
- [ ] Verificar que o banco não ficou corrompido (reabrir app).

### 5. Verificar Paridade de Custom Protocols
- [ ] Confirmar que `asset://` serve thumbnails corretamente.
- [ ] Confirmar que imagens renderizam na grid sem erros de CORS ou Content-Type.

### 6. Gerar Relatório Final
- [ ] Criar `docs/report/backend-architeture/walkthrough-final.md` com:
  - Resumo da migração completa (Fases 1-8).
  - Tabela de todos os IPC commands V2 finais.
  - Métricas de código (n° de arquivos, linhas, warnings).
  - Diferenças intencionais do V2 vs V1 (melhorias, decisões arquiteturais).
  - Lista de todo arquivo `.rs` do projeto V2 final.
  - Screenshots ou evidências de funcionamento (se possível via app rodando).

### 7. Atualizar Documentação
- [ ] Marcar todas as sprints 7.1-8.4 como "Concluídas" nos tracker files.
- [ ] Atualizar `roadmap.md` refletindo a conclusão da Fase 7 e 8.
- [ ] Atualizar `architecture-comparison-report.md` com status "100% migrado".

---

## 📁 Arquivos de Referência V1

| Funcionalidade         | Arquivo V1 (Mundam-main)              | Notas                |
| ---------------------- | ------------------------------------- | -------------------- |
| IPC commands completos | `src-tauri/src/lib.rs` L159-212       | Lista de 53 commands |
| Todos os DB modules    | `src-tauri/src/db/*.rs`               | 9 arquivos           |
| Todos os commands      | `src-tauri/src/library/commands/*.rs` | 7 arquivos           |
| Custom protocols       | `src-tauri/src/protocols/*.rs`        | 10 arquivos          |

## 📁 Arquivos a Verificar/Modificar no V2

| Arquivo V2 (Mundam)                                                 | Ação                                    |
| ------------------------------------------------------------------- | --------------------------------------- |
| `src-tauri/src/lib.rs`                                              | Confirmar TODOS os commands registrados |
| Todos os `delivery/tauri/commands/*.rs`                             | Verificar compilação                    |
| `docs/report/backend-architeture/walkthrough-final.md` (novo)       | Relatório final                         |
| `docs/report/backend-architeture/architecture-comparison-report.md` | Atualizar status                        |
| `docs/report/backend-architeture/definition/roadmap.md`             | Marcar conclusão                        |
| Todos os sprint docs em `sprints/*.md`                              | Marcar como concluído                   |

---

## 💡 Notas para o Desenvolvedor / Agente
> Esta sprint é primariamente de **verificação** e **documentação**. Não se escreve funcionalidade nova aqui — apenas corrige-se bugs encontrados durante a validação e gera-se o relatório final.

> Se bugs forem encontrados durante a validação, corrigi-los NESTA sprint. Não crie sprints adicionais para bugfix. O objetivo é sair desta sprint com o backend V2 **100% funcional e equivalente ao V1** em termos de feature set.

> O relatório final (`walkthrough-final.md`) é o documento que encerra oficialmente a saga de migração. Ele deve ser auto-suficiente para que qualquer novo desenvolvedor entenda o que foi feito e por quê.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- 
