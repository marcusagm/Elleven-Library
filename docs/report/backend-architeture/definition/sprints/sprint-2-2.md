# Sprint 2.2: O Adaptador do Ledger (Infra SQLx)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Sujar as mãos implementando o lado de Inserts do CQRS. Conectar a Interface criada na sprint anterior ao `SQLx`, efetuando transações ACID no SQLite real para as tabelas vitais de Media/Assets.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Transações Reais:** O `SqliteLedgerAdapter` deve gravar na tabela `assets` real sem falhas de lock.
2. **Rolback Seguro:** Uma falha proposital no meio do método (ex: duplicidade de Hash) deve provar que a transação `sqlx::Transaction` não gravou nada parcial na base de dados.
3. **Persistência Completa:** Inserir um `CreateAssetCommand` usando o Ledger e conferir via SQLite viewer (ou via teste direto) a existência da tupla cravada no disco com carimbos `created_at`.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Criação do Adaptador
- [ ] Criar o arquivo `src-tauri/src/infra/db/ledger_adapter.rs`.
- [ ] Instanciar a struct passiva `pub struct SqliteLedgerAdapter { pool: sqlx::SqlitePool, bus: Arc<dyn EventBus> }`.
- [ ] Implementar a trait `TransactionalAssetLedger` mapeando os `Commands` para `INSERT INTO` e `UPDATE` usando a macro `sqlx::query!`.

### 2. Controle Transacional Rigoroso
- [ ] Utilizar estritamente `self.pool.begin().await?` toda vez que um Command modificar mais de 1 estado ou tabela no futuro, ativando `commit().await?` apenas na penúltima linha.

### 3. Emissão de Eventos Genuínos
- [ ] Após o `commit().await?` ser garantido como Sucesso do SQLx, disparar via `self.bus.publish(...)` o Reflexo do domínio. Um arquivo gravado emite `DomainEvent::AssetDiscovered`.

### 4. Registro no Container
- [ ] Abstrair e instanciar `Arc::new(SqliteLedgerAdapter)` e injetar em `app.manage(...)` no local do antigo mock da Fase 1, selando a fiação de ponta a ponta.

---

## 💡 Notas para o Desenvolvedor / Agente
> Você passará raiva com "borrow checker" e "macros do Sqlx" não achando colunas aqui. Lembre-se primordialmente de rodar o `cargo sqlx prepare` no contexto do workspace. Mantenha os métodos compactos; o `LedgerAdapter` só obedece às Structs, quem montou as regras e validou se o diretório existe foram as camadas superiores de Handler antes de chamar este adaptador.
