# Sprint 6.3: Limpeza de Banco de Dados e Migração Definitiva

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 6:** Cleanup e Consolidação V2
**Objetivo:** Após a exclusão do código V1 do backend do Rust, o Banco de Dados (SQLite) necessita ter todas as tabelas legadas eliminadas `DROP TABLE`. Adicionalmente, as tabelas que assumiram provisoriamente o namespace oficial com `v2_` (`v2_assets`, `v2_folders`, etc.) devem ser renomeadas para seus nomes finais e definitivos sem o prefixo.

---

## 🎯 Critérios de Aceite
1. Execução de uma Migration Limpa do `sqlx` que derrube logicamente e fisicamente as tabelas do Mundam V1 originais.
2. A migration renomeia logicamente os schemas transicionais (ex: `ALTER TABLE v2_assets RENAME TO assets;`).
3. Tamanho do banco de testes deve reduzir. Integridade relacional em PRAGMA foreign_keys = ON deve passar sem Warnings.
4. Checklist de auditoria do Agente (`scripts/checklist.py`) deve rodar com êxito sem erros de Schema ou Tipagens SQL.

---

## 📋 Tarefas (Checklist do Agente)

### 1. SQLx Database Migration Final
- [ ] Listar quais são as tabelas exclusivas do V1 usando `sqlite3` ou analisando as planilhas de migrations legadas: `assets`, `folders`, `db_state`, e outras que não foram transferidas para `v2_xxx`.
- [ ] Criar nova Migration `.sql`: `src-tauri/migrations/[timestamp]_cleanup_v1_legacy_tables.sql`.
- [ ] Escrever as queries de `DROP TABLE IF EXISTS ...` de forma sequencial parando chaves entrelinhas se necessário.

### 2. Validação Contínua (Sanity Check)
- [ ] Passar o script mestre `checklist.py` focado no banco para ver se a auditoria em Runtime detecta furos do Schema ou campos órfãos V2 (como a Tipagem da cor no ID resolvida na Fase 5).

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- `docs/report/backend-architeture/definition/sprints/sprint-6-3.md` (Tracker)
- `src-tauri/migrations/*_cleanup_v1_legacy_tables.sql` (Nova Migration Opcional)

---

## 💡 Notas para o Desenvolvedor / Agente
> O SQLite bloqueia DELETES drásticos em tabelas que possuem views ativas ou hooks. A migração deve rodar perfeitamente limpa e os SQLx structs de compilação em `.sqlx/` (`cargo sqlx prepare`) devem ser os únicos a moldar o executável.
