# Sprint 2.4: Taxonomia, Metadata e Pastas (Grafos e Hierarquia)

**Status:** Concluído
**Data e hora de inicio:** 2026-03-06T06:45:00-03:00
**Data da conclusão:** 2026-03-06T16:45:00-03:00

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Expandir drasticamente a Base para acomodar a espinha dorsal de classificação do Mundam: Mapear a Árvore Lógica Recursiva (Pastas) e a categorização N:N Livre (Tags).

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **[x] Self-Referential Completa:** A Tabela/Struct `Folder` deve suportar e recuperar árvores lógicas validando chaves estrangeiras (`parent_id`) sem corromper órfãos.
2. **[x] Tags Isoladas:** Criar Tags únicas. Associa-las a N Assets na Tabela-Pivô. Listar Assets sob uma dada tag através do "Lado Q" da base de dados.
3. **[x] Mutações Dependentes Tidas:** Uma mutação `AssignTagCommand` deve invocar transação limpa no Adaptador Ledger que verifique integridades (se a tag e o Asset existem antes de popular a tabela join).

---

## 📋 Tarefas (Checklist do Agente)

### 1. Extensão de Domínios
- [x] Em `core/commands/` criar comandos robustos: `CreateFolderCommand`, `SetAssetFolderCommand`, `TagAssetCommand`, `UntagAssetCommand`. (Refinado: Implementado como `CreateFolder`, `SetAssetFolder` e `UpdateAssetTags` para maior coesão).
- [x] No `AssetLedger`, adicionar endpoints correspondendo a estes Commands para garantir Locks corretos.

### 2. Tabelas e Adaptador
- [x] Revisitar/Desenformar no Adapter CQRS SQL (`infra/database/ledger_adapter.rs` e models complementares). (Nota: Mapeado diretamente no `ledger.rs` visando simplicidade técnica nesta fase).
- [x] Inserir SQLx querys para relacionamentos (Pivot Tables). Empregar comandos limpos garantindo ausência de duplicação: `INSERT OR IGNORE` para uniões de Tags se apropriado na lógica sqlite, ou conferências prévias na `Transaction` Rust.

### 3. Expansion dos Queries e Handlers
- [x] Expandir `AssetQueries` para resolver complexidades hierárquicas, exemplo: Resgatar todos descendentes de Pastas Lógicas simulando uma *Materialized Path* ou Recursive CTE, mapeado por métodos claros `async fn get_children_folders`.

### 4. Bateria de Relacionamento
- [x] `tokio::test`: Criar Pasta -> Criar Arquivo na Pasta -> Criar Tag "Arte" -> Vincular. Recuperar Arquivo através do Id da Pasta garantindo Relacionamento Intocado.

---

## 🚀 Relatório de Implementação (Pós-Sprint)

### Dificuldades e Desafios
- **Inconsistência de Modelo (dominant_colors):** A introdução do campo `dominant_colors` no modelo `AssetDb` exigiu a atualização de todos os macros `query_as!` no `ledger.rs` e `queries.rs` com casts explícitos para `NULL` onde joins estavam ausentes.
- **Gestão de Nulidade no SQLx:** Necessidade de overrides explícitos (ex: `folder_id as "folder_id?"`) para garantir a compilação offline e evitar erros de inferência de tipos em campos opcionais.

### Melhorias Realizadas
- **Simplificação da API Tauri:** Removido o sufixo `_v2` dos comandos Tauri (`get_assets`, `create_folder`, etc.) para manter uma interface limpa e profissional.
- **Auditoria Centralizada:** Todas as mutações de taxonomia agora geram logs automáticos em `v2_asset_operations_log`.

### Desvios de Escopo
- **Sincronização Global de Metadados:** Inclusão do suporte ao campo `dominant_colors` em todo o ciclo de vida do asset para manter a integridade técnica global.

---

## 💡 Notas para o Desenvolvedor / Agente
> A Gestão em Grafos (Pastas Relacionais Self-Referencing) em SQLite necessita de CTE recursiva se for extrair caminhos completos `/v2/final/aprovados` com uma query só. Se isso ultrapassar a margem de complexidade macro do Rust na adaptação, mapeie as tabelas usando pathing prefixado no DB. Lembrete crucial: CQRS obriga centralização mutável. As Tags se inserem sempre por via de Intent Commands.
## 📂 Arquivos Modificados

### Banco de Dados & Infraestrutura
- [20260306000000_sprint_2_4_taxonomy_and_folders.sql](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/migrations/20260306000000_sprint_2_4_taxonomy_and_folders.sql)
- [ledger.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/infra/database/ledger.rs)
- [queries.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/infra/database/queries.rs)
- [models.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/infra/database/models.rs)

### Domínio & Lógica de Negócio
- [asset.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/models/asset.rs)
- [command.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/ledger/command.rs)
- [asset.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/repository/asset.rs)
- [mock.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/ledger/mock.rs)

### Camada de Entrega (Tauri API)
- [asset_ledger.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/delivery/tauri/asset_ledger.rs)
- [asset_queries.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/delivery/tauri/asset_queries.rs)
- [lib.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/lib.rs)
