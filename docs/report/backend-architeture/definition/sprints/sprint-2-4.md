# Sprint 2.4: Taxonomia, Metadata e Pastas (Grafos e Hierarquia)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Expandir drasticamente a Base para acomodar a espinha dorsal de classificação do Mundam: Mapear a Árvore Lógica Recursiva (Pastas) e a categorização N:N Livre (Tags).

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Self-Referential Completa:** A Tabela/Struct `Folder` deve suportar e recuperar árvores lógicas validando chaves estrangeiras (`parent_id`) sem corromper órfãos.
2. **Tags Isoladas:** Criar Tags únicas. Associa-las a N Assets na Tabela-Pivô. Listar Assets sob uma dada tag através do "Lado Q" da base de dados.
3. **Mutações Dependentes Tidas:** Uma mutação `AssignTagCommand` deve invocar transação limpa no Adaptador Ledger que verifique integridades (se a tag e o Asset existem antes de popular a tabela join).

---

## 📋 Tarefas (Checklist do Agente)

### 1. Extensão de Domínios
- [ ] Em `core/commands/` criar comandos robustos: `CreateFolderCommand`, `SetAssetFolderCommand`, `TagAssetCommand`, `UntagAssetCommand`.
- [ ] No `AssetLedger`, adicionar endpoints correspondendo a estes Commands para garantir Locks corretos.

### 2. Tabelas e Adaptador
- [ ] Revisitar/Desenformar no Adapter CQRS SQL (`infra/db/ledger_adapter.rs` e models complementares).
- [ ] Inserir SQLx querys para relacionamentos (Pivot Tables). Empregar comandos limpos garantindo ausência de duplicação: `INSERT OR IGNORE` para uniões de Tags se apropriado na lógica sqlite, ou conferências prévias na `Transaction` Rust.

### 3. Expansion dos Queries e Handlers
- [ ] Expandir `AssetQueries` para resolver complexidades hierárquicas, exemplo: Resgatar todos descendentes de Pastas Lógicas simulando uma *Materialized Path* ou Recursive CTE, mapeado por métodos claros `async fn get_children_folders`.

### 4. Bateria de Relacionamento
- [ ] `tokio::test`: Criar Pasta -> Criar Arquivo na Pasta -> Criar Tag "Arte" -> Vincular. Recuperar Arquivo através do Id da Pasta garantindo Relacionamento Intocado.

---

## 💡 Notas para o Desenvolvedor / Agente
> A Gestão em Grafos (Pastas Relacionais Self-Referencing) em SQLite necessita de CTE recursiva se for extrair caminhos completos `/v2/final/aprovados` com uma query só. Se isso ultrapassar a margem de complexidade macro do Rust na adaptação, mapeie as tabelas usando pathing prefixado no DB. Lembrete crucial: CQRS obriga centralização mutável. As Tags se inserem sempre por via de Intent Commands.
