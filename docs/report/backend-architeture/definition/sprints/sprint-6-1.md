# Sprint 6.1: Mapeamento e Refatoração do Frontend

**Status:** Pendente
**Data e hora de inicio:** -
**Data e hora da conclusão:** -

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
- [ ] Atualizar `src/lib/tags.ts` para invocar os novos Queries Base.
- [ ] Atualizar `src/core/tauri/services.ts` (Substituir chamadas de FileSystem/Folders para a porta "folders" em V2).
- [ ] Atualizar `src/core/store/metadata/searchActions.ts` para SearchQueries.
- [ ] Mapear as Mutations em `src/core/store/library/` para o Controller de Mutações (`mutations.rs`).

### 2. Alinhamento de DTOs e Tipos (TS x Rust)
- [ ] Garantir que o nome das propriedades das Structs Rust (`#[serde(rename_all = "camelCase")]`) coincidam com o que a UI espera no retorno JSON.
- [ ] Resolver possíveis descompassos, como o retorno de Listas `Vec<DTO>` no lugar de wrappers exóticos.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- `docs/report/backend-architeture/definition/sprints/sprint-6-1.md` (Update Tracker)

---

## 💡 Notas para o Desenvolvedor / Agente
> A transição do payload V1 para V2 no frontend não pode gerar downtime ou regressões no Client State (`Solid Store`). Faça mapeamentos com Destructuring e Adapters localmente no Typescript se a Model do Rust mudar drásticamente.
