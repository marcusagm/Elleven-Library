# Sprint 6.1: Mapeamento e Refatoração do Frontend

**Status:** Concluído
**Data e hora de inicio:** 2026-03-10
**Data e hora da conclusão:** 2026-03-10

**Fase 6:** Cleanup e Consolidação V2
**Objetivo:** Substituir todas as chamadas de comandos Tauri no Frontend (legados da V1) pelas equivalentes da V2, garantindo que a UI se comunique exclusivamente com as novas rotas da Arquitetura Hexagonal.

---

## 🎯 Critérios de Aceite
1. Nenhum arquivo `.ts` do frontend (como `db.ts`, `tags.ts`, `searchActions.ts`) invoca as Actions antigas. Retirar chamadas do tipo `get_all_subfolders`, `get_assets_filtered` e `create_tag`.
2. As interfaces TypeScript em `types/` devem estar sincronizadas com os DTOs do Rust (Commands & Queries).
3. Todas as rotas IPC / Componentes UI não devem fazer uso do prefixo `v2` nas chamadas ou nomenclaturas contidas na V2, este prefixo foi provisório e será consolidado.
4. Todas as chamadas Tauri Tauri (`invokeCommand`) funcionam sem disparos de erros de _"Command Not Found"_.
5. O build do frontend local (`npm run build` / tauri) reporta zero erros TS.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Refatoração de Invokes
- [x] Atualizar `src/lib/tags.ts` para invocar os novos Queries Base.
- [x] Atualizar `src/core/tauri/services.ts` (Substituir chamadas de FileSystem/Folders para a porta "folders" em V2).
- [x] Atualizar `src/core/store/metadata/searchActions.ts` para SearchQueries.
- [x] Mapear as Mutations em `src/core/store/library/` para o Controller de Mutações (`mutations.rs`).

### 2. Alinhamento de DTOs e Tipos (TS x Rust)
- [x] Garantir que o nome das propriedades das Structs Rust (`#[serde(rename_all = "camelCase")]`) coincidam com o que a UI espera no retorno JSON.
- [x] Resolver possíveis descompassos, como o retorno de Listas `Vec<DTO>` no lugar de wrappers exóticos.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- Adaptar as antigas assinaturas baseadas em milhares de parâmetros independentes (ex `getAssetsFiltered`) para a nova abstração `AssetFilter` via Destructuring em `libraryActions.ts`.
- Tipar com precisão o enum de Criteria e LogicalOperator para Typescript de acordo com o Serde Tag do Rust, solucionando problemas de descompasso via mapeamento forçado para minúsculo (`and`, `or`).
- Identificar e solucionar a ausência da macro `#[serde(rename_all = "camelCase")]` nas estruturas `PageParams` e `AssetFilter` no Rust, o que causava o erro `missing field page_size`, pois o TS enviava em camelCase e o Rust esperava snake_case.
- Adicionar os novos comandos V2 dentro das listas de `permissions` e `capabilities` do Tauri de forma minuciosa, pois as IPC calls da nova versão geravam erros de `Command Not Found` no Client.
- Remover o enum artificial de status visual `Untagged` em prol de um filtro booleano focado na ausência de tags nos relacionamentos.

### Melhorias Realizadas
- Todos os endpoints da API de biblioteca utilizam CQRS unificado através da porta nativa V2 (`get_assets`, `search_assets`, `list_folders`).
- Compilação `npx tsc` ocorrendo de maneira saudável validando os contratos de payload com a API V2.
- Consolidado o objeto unico de manipulação de tag array (`tagsToAdd`, `tagsToRemove`) permitindo chamadas em lote (Batch).
- Filtro de `untagged` migrado para pesquisa na query SQL do sqlite (via subquery `NOT IN (SELECT asset_id FROM v2_asset_tags)`), deixando o Backend mais inteligente e retirando dependência da Enum de status da máquina de estado.

### 📄 Arquivos Criados ou Modificados
- `docs/report/backend-architeture/definition/sprints/sprint-6-1.md` (Update Tracker)
- `src/types/index.ts` (Implementação de interfaces V2 como `AssetFilter`, `PageParams`, `SearchCriteria`, e tipos atômicos como `LogicalOperator`)
- `src/lib/tags.ts` (Mapeamento de rotas V2 para endpoints como `list_tags` e `update_asset_tags`)
- `src/lib/db.ts` (Rotas legadas isoladas e reaproveitadas via API V2 `list_folders`)
- `src/core/store/library/libraryActions.ts` (Criação de parâmetros em lote, destituição das payloads V1 gigantes e mapper `mapToV2SearchGroup`)
- `src/core/store/metadata/searchActions.ts`
- `src/core/store/metadata/tagActions.ts`
- `src/core/tauri/services.ts`
- `src-tauri/permissions/main.toml` (Permissões de IPC adicionadas para novos commands V2)
- `src-tauri/capabilities/default.json` (Capacidades de IPC expostas para o Frontend em runtime)
- `src-tauri/src/core/models/asset.rs` (Correção do camelCase via Serde e inclusão do parâmetro flag `untagged`)
- `src-tauri/src/infra/database/queries.rs` (Lógica SQL embarcada para suporte à busca refinada de arquivos não categorizados, incluindo `untagged`)

---

## 💡 Notas para o Desenvolvedor / Agente
> A transição do payload V1 para V2 no frontend não pode gerar downtime ou regressões no Client State (`Solid Store`). Faça mapeamentos com Destructuring e Adapters localmente no Typescript se a Model do Rust mudar drásticamente.
