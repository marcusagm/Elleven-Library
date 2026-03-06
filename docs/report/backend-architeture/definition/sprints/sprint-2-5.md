# Sprint 2.5: Search Builder Avançadas (Cores, Arrays e Dictionary)

**Status:** Concluído ✅
**Data e hora de inicio:** 2026-03-06 14:00
**Data da conclusão:** 2026-03-06 16:30

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Trazer de volta a lendária super-busca do Mundam. O Lado "Q" (Leitura) é convertido num poderoso `QueryBuilder` capaz de correlacionar dezenas de filtros interativos, tags e os polêmicos filtros Visuais baseados em distância CIELAB de Cores e Proximidade de Hash Euclidiano.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. [x] **Composição Híbrida Infinita:** Uma query solicitando "Extensões = PSD || JPG", "Tamanho > 1GB", "Tag=Aprovado" e "Cor Próxima a #FF0000 limitando Distância a X" deve ser forjada em uma única query SQL limpa devolvida em frações de segundo.
2. [x] **Definição da Search Criteria:** Uma DTO formal deve codificar o objeto JSON obscuro que vem do frontend mapeando as Regras de Seleção.
3. [x] **Rust Query Builder Dinâmico:** Um motor de `sqlx::QueryBuilder` seguro em Strings concêntricas unindo as Tabelas Assets com as Tabelas `AssetColor`, prevenindo injeções de SQL.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição Base de Filtros (Core -> Feature)
- [x] Em `core/models/`, estabelecer Enums complexos: `FilterOperator` (Gt, Lt, Exact, InArray). 
- [x] Criar `SearchCriteria`, com vetores opcionais genéricos que suportam o escopo da busca pesada legada do repositório da V1.

### 2. SQLx Query Builder (Infra)
- [x] Em `infra/database/search_builder.rs`, instanciar um formatador manual empurrando trechos de SQL (`WHERE`, `AND`) conforme a validade lógica dos campos.
- [x] A extração Euclidiana da Cor dominante baseada na matemática do banco: O SQL ou o Rust deve iterar as tags Hex/LAB convertidas da tabela associativa de Cores do SQLite perante distâncias pre-indexadas. (Reúso da mecânica robusta anterior).

### 3. Exposição ao Frontend
- [x] Criar command `#[tauri::command] pub async fn search_assets(criteria: SearchCriteria)`.
- [x] Interligar invocação em Typescript via `invoke("search_assets", { criteria: { ... } })` e despejar no Console log a magia das tuplas sendo varridas em O(Log N) pelos Indices corretos.

---

## 🚀 Informações da Implementação

### Dificuldades Encontradas
- **Matemática de Cor em SQL**: O SQLite não possui funções nativas para cálculos de raiz quadrada, necessários para a distância Euclidiana em CIELAB. A solução foi otimizar a query comparando o quadrado da distância acumulada com o quadrado do threshold (`d² < t²`), preservando a performance sem dependências externas.
- **Complexidade de Joins**: Integrar buscas por tags (Many-to-Many) e cores (One-to-Many) em uma única query dinâmica exigiu o uso cuidadoso de `LEFT JOIN` e `DISTINCT` para garantir que o número de resultados permanecesse correto sem duplicações artificiais.

### Melhorias Realizadas
- **Recursividade Lógica**: Implementamos suporte a grupos aninhados de busca, permitindo combinações infinitas de `(A AND B) OR (C AND D)` nativamente no motor de busca.
- **Segurança Proativa**: Todas as entradas dinâmicas são bindadas via `sqlx::QueryBuilder`, tornando o sistema imune a ataques de SQL Injection por design.
- **Testes Unitários**: O motor de construção de query (`search_builder.rs`) conta com cobertura de testes que validam a integridade do SQL gerado para filtros simples, compostos e visuais.

---

## 📁 Arquivos Modificados
- `src-tauri/src/core/models/search.rs` (Criação definitiva dos modelos de busca)
- `src-tauri/src/core/models/mod.rs` (Exportação dos modelos)
- `src-tauri/src/core/repository/asset.rs` (Atualização do port AssetQueryHandler)
- `src-tauri/src/infra/database/search_builder.rs` (Criação do motor dinâmico SQLx)
- `src-tauri/src/infra/database/queries.rs` (Implementação SQLite da busca avançada)
- `src-tauri/src/infra/database/mod.rs` (Exposição da infra de busca)
- `src-tauri/src/feature/search/query_handler.rs` (Orquestrador de aplicação)
- `src-tauri/src/feature/search/mod.rs` (Configuração do módulo de feature)
- `src-tauri/src/feature/mod.rs` (Registro da feature de busca)
- `src-tauri/src/delivery/tauri/asset_queries.rs` (Exposição do command `search_assets`)
- `src-tauri/src/lib.rs` (Wiring de dependências e registro do handler Tauri)

---

## 💡 Notas para o Desenvolvedor / Agente
> Construir query SQL stringada (dinâmica) fere o uso restrito do macro `query_as!` que compila offline, exigindo que você intere o uso passivo da lib subjacente e aplique a variação `QueryBuilder` do SQLx. Mantenha os Bindings tipados (`.bind(valor)`) sempre ativados em todo loop de montagem do WHERE para imunizar qualquer inserção perigosa pela UI (SQL Injection Alert). Atenção as querys de Cor: Traga de volta o join complexo atual do Mundam para cá intacto!
