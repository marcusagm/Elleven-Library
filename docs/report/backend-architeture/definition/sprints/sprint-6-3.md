# Sprint 6.3: Limpeza de Banco de Dados e Migração Definitiva

**Status:** Concluído ✅
**Data e hora de inicio:** 2026-03-10 12:00
**Data da conclusão:** 2026-03-10 14:30

**Fase 6:** Cleanup e Consolidação V2
**Objetivo:** Após a exclusão do código V1 do backend do Rust, o Banco de Dados (SQLite) necessita ter todas as tabelas legadas eliminadas `DROP TABLE`. Adicionalmente, as tabelas que assumiram provisoriamente o namespace oficial com `v2_` (`v2_assets`, `v2_folders`, etc.) devem ser renomeadas para seus nomes finais e definitivos sem o prefixo.

---

## 🎯 Critérios de Aceite
1. [x] Execução de uma Migration Limpa do `sqlx` que derrube logicamente e fisicamente as tabelas do Mundam V1 originais.
2. [x] A migration renomeia logicamente os schemas transicionais (ex: `ALTER TABLE v2_assets RENAME TO assets;`).
3. [x] Tamanho do banco de testes deve reduzir. Integridade relacional em PRAGMA foreign_keys = ON deve passar sem Warnings.
4. [x] Checklist de auditoria do Agente (`scripts/checklist.py`) deve rodar com êxito sem erros de Schema ou Tipagens SQL.

---

## 📋 Tarefas (Checklist do Agente)

### 1. SQLx Database Migration Final
- [x] Listar quais são as tabelas exclusivas do V1 usando `sqlite3` ou analisando as planilhas de migrations legadas: `assets`, `folders`, `db_state`, e outras que não foram transferidas para `v2_xxx`.
- [x] Criar nova Migration `.sql`: `src-tauri/migrations/20260310120000_cleanup_v1_and_rename_v2.sql`.
- [x] Escrever as queries de `DROP TABLE IF EXISTS ...` de forma sequencial parando chaves entrelinhas se necessário.

### 2. Validação Contínua (Sanity Check)
- [x] Passar o script mestre `checklist.py` focado no banco para ver se a auditoria em Runtime detecta furos do Schema ou campos órfãos V2 (como a Tipagem da cor no ID resolvida na Fase 5).

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- **Parsing Chronológico do SQLx**: Durante os testes automatizados, o SQLx tentava validar queries contra a tabela `assets` antes que a migration de renomeação fosse concluída em bancos `:memory:`. Isso exigiu uma estratégia de **Bridge Schemas** (tabela `IF NOT EXISTS` criada no início da migration) para garantir que a estrutura existisse para inspeção do compilador.
- **Tipagem de Identificadores**: A transição de IDs baseados em `i64` (V1) para `String` (V2 - UUID/ULID) causou falhas silenciosas no SQLx. Foi necessário refatorar o `ledger.rs` para garantir conversões explícitas usando `.to_string()`.

### Melhorias Realizadas
- **Migration Resiliente**: O script de migração foi projetado para ser idempotente e seguro contra qualquer estado inicial do banco de dados, utilizando o padrão de tabelas temporárias para garantir que comandos `CREATE TABLE` sejam limpos e sem referências a prefixos legados.
- **Sincronização Offline**: Atualização completa do cache `.sqlx/` para garantir que o suporte a tipos em tempo de compilação esteja 100% alinhado com o novo esquema.

### 📄 Arquivos Criados ou Modificados
- `src-tauri/migrations/20260310120000_cleanup_v1_and_rename_v2.sql` (Criação da migration de limpeza)
- `src-tauri/src/infra/database/ledger.rs` (Ajuste de tipagem de IDs e queries)
- `src-tauri/src/infra/database/queries.rs` (Remoção de prefixos `v2_` e normalização de projeções)
- `src-tauri/src/infra/database/models.rs` (Atualização dos modelos `Db` para o esquema final)
- `src-tauri/src/infra/database/manager.rs` (Ajuste na inicialização e migrations)
- `src-tauri/src/infra/database/search_builder.rs` (Correção de asserts de testes)
- `docs/report/backend-architeture/definition/sprints/sprint-6-3.md` (Atualização deste documento)

---

## 💡 Notas para o Desenvolvedor / Agente
> O SQLite bloqueia DELETES drásticos em tabelas que possuem views ativas ou hooks. A migração deve rodar perfeitamente limpa e os SQLx structs de compilação em `.sqlx/` (`cargo sqlx prepare`) devem ser os únicos a moldar o executável.
