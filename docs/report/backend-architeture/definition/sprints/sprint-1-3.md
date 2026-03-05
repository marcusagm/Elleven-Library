# Sprint 1.3: Data Model Base Base (DB Infra)

**Status:** Concluída
**Data e hora de inicio:** 2026-03-05 16:30
**Data da conclusão:** 2026-03-05 19:15

**Fase 1:** Fundação & Observabilidade (Core Mínimo)
**Objetivo:** Acoplar o ambiente limpo de Banco de Dados (`SQLx SQLite`) no formato CQRS estrito, provisionando os tipos (Structs do DB) e preparando o ecossistema para mutações do Ledger e queries autônomas.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Migrations Verificadas:** Ao inicializar a compilação, o `SQLx` rodou em um banco vazio (`mundam_test.db`), validando DDL/Tabelas com sucesso e gerando o diretório `.sqlx` canônico offline.
2. **Separação Abstrata DTO:** Criação da Struct `Asset` no formato legível de Domínio, distante do Raw SqlRow puro, validando conversão com segurança de Nullables.
3. **Escrita Simples Mapeada:** Realizar a gravação isolada num teste `tokio::test` ou um comando temporário do Tauri na tabela inicial de Audit/Assets.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Preparação Estrutural DTO e SQLx
- [x] Fomentar as Structs puras `db/models.rs` que atuarão entre a `Infra` e o `Core`:
  - `AssetMetadata`
  - `Asset` original
- [x] Garantir o `.gitignore` isentando os `*.db` limpos de migração.
- [x] Configuração formal do Conector em `src-tauri/src/infra/database/manager.rs` que retorne a Pool do banco, já aplicando o comando pragma `PRAGMA default_cache_size`, `PRAGMA synchronous = NORMAL`, e `PRAGMA journal_mode = WAL`.

### 2. Contrato de Leitura Básica (Queries)
- [x] Criar o `Trait` de leitor rápido `AssetQueryHandler` em `src-tauri/src/core/repository/`.
- [x] Instanciar o Adaptador `SqliteAssetQueries` (em `infra/database/queries.rs`) consumindo a Pool injetada, devolvendo um `Vec<Asset>` simples.
- [x] Respeitar macro segura: Empregue apenas abstrações seguras de compilação em disco (macros `sqlx::query_as!`). Execute o comando `cargo sqlx prepare` manualmente em background.

### 3. Integração (Wiring)
- [x] Chamar `manager::init_database()` na carga do main.
- [x] Encapsular a Trait nas State Wrappers: `app.manage(Arc::new(sqlite_query_handler) as Arc<dyn AssetQueryHandler>)`.
- [x] Elaborar um teste unitário transacional que injete "Logo_Temp.png" e resgate da Base isolada confirmando a estrutura.

---

## 💡 Notas para o Desenvolvedor / Agente
> Nunca mescle lógicas vitais dentro do `manager.rs`. Queries ficam em read-models passivos, longe das portas ativas (Transactions do Ledger). Essa Sprint trata puramente de **Fundações passivas no Banco**. A validação e máquina de estado das mutações complexas ficarão pra próxima etapa (Fase 2). Atenção triplicada à macro em tempo de compilação: Não trave a build quebrando macros contra tabelas recém extintas. Use cache limpo!
