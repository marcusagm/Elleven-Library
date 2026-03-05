# Sprint 2.3: Query Handlers Base (Leitura Flexível)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Construir o lado "Q" do CQRS. Os mecanismos de Leitura Flexível ignoram propositalmente a rigidez do `Ledger` mutativo para varrer o SQLite com performance brutal para entregar listagens ao Frontend.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Consultas Livres Otimizadas:** Um método `get_all_assets` deve retornar milhares de itens processados numa DTO resumida rapidamente sem lockar Threads de escrita mutativas.
2. **Separação de Traits:** Toda query feita num Handler não deve usar instâncias de "AssetLedger", provando o by-pass do padrão de CQRS via a trait `AssetQueries`.
3. **Paginação Direta:** Demonstrar `LIMIT/OFFSET` em funcionamento sobre a interface de chamadas.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição da Porta de Reading
- [ ] Expandir ou consolidar o `src-tauri/src/core/repository/queries.rs`.
- [ ] Adicionar os escopos de Read: `async fn get_asset(&self, id: i64) -> AppResult<Asset>`, `async fn list_assets_paginated`, etc.

### 2. Implementação SQLx Sem Transação Aberta
- [ ] No `src-tauri/src/infra/db/queries.rs` criado na Fase 1, plugar SQLx macros em métodos diretos (As transações pesadas do Ledger ficam isoladas longe destas queries Read-Only que usam as Pools passivas).
- [ ] Cuidar da conversão em massa para Modelos DTO leves se possível (evitar puxar Blobs se a UI quer apenas listar 200 nomes de arquivos).

### 3. Criação dos Handlers de Ligação Front/Back
- [ ] Em `src-tauri/src/feature/assets/queries.rs` criar as `query_handlers`. Elas engolem parâmetros brutos (id, path, offset) e repassam para a infra de `AssetQueries`.
- [ ] Exportar essa query no `delivery/tauri/asset_commands.rs` através do `#[tauri::command]`.

### 4. Ciclo Falso-E2E de Confirmação
- [ ] Executar o frontend Solid local, usar a janela do DevTools para invocar o comando do Tauri `invoke('get_assets')` e confirmar a ponte veloz entre Tauri-JS -> Tauri-Rust -> Feature -> Infra(SQLx).

---

## 💡 Notas para o Desenvolvedor / Agente
> Em CQRS, as Views (Interfaces visuais) conversam e bebem puramente dos Readings. Fazer o Frontend chamar a Interface do AssetLedger que grava no banco para recuperar um ativo é um crime capital nesta arquitetura. Foque no desempenho (Read Model focado em Queries Indexadas).
