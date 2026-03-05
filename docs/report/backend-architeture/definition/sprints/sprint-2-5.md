# Sprint 2.5: Search Builder Avançadas (Cores, Arrays e Dictionary)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 2:** Domínio & Operações de Mutação (CQRS)
**Objetivo:** Trazer de volta a lendária super-busca do Mundam. O Lado "Q" (Leitura) é convertido num poderoso `QueryBuilder` capaz de correlacionar dezenas de filtros interativos, tags e os polêmicos filtros Visuais baseados em distância CIELAB de Cores e Proximidade de Hash Euclidiano.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Composição Híbrida Infinita:** Uma query solicitando "Extensões = PSD || JPG", "Tamanho > 1GB", "Tag=Aprovado" e "Cor Próxima a #FF0000 limitando Distância a X" deve ser forjada em uma única query SQL limpa devolvida em frações de segundo.
2. **Definição da Search Criteria:** Uma DTO formal deve codificar o objeto JSON obscuro que vem do frontend mapeando as Regras de Seleção.
3. **Rust Query Builder Dinâmico:** Um motor de `sqlx::QueryBuilder` seguro em Strings concêntricas unindo as Tabelas Assets com as Tabelas `AssetColor`, prevenindo injeções de SQL.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Definição Base de Filtros (Core -> Feature)
- [ ] Em `core/models/`, estabelecer Enums complexos: `FilterOperator` (Gt, Lt, Exact, InArray). 
- [ ] Criar `SearchCriteria`, com vetores opcionais genéricos que suportam o escopo da busca pesada legada do repositório da V1.

### 2. SQLx Query Builder (Infra)
- [ ] Em `infra/db/search_builder.rs`, instanciar um formatador manual empurrando trechos de SQL (`WHERE`, `AND`) conforme a validade lógica dos campos.
- [ ] A extração Euclidiana da Cor dominante baseada na matemática do banco: O SQL ou o Rust deve iterar as tags Hex/LAB convertidas da tabela associativa de Cores do SQLite perante distâncias pre-indexadas. (Reúso da mecânica robusta anterior).

### 3. Exposição ao Frontend
- [ ] Criar command `#[tauri::command] pub async fn search_assets(criteria: SearchCriteria)`.
- [ ] Interligar invocação em Typescript via `invoke("search_assets", { criteria: { ... } })` e despejar no Console log a magia das tuplas sendo varridas em O(Log N) pelos Indices corretos.

---

## 💡 Notas para o Desenvolvedor / Agente
> Construir query SQL stringada (dinâmica) fere o uso restrito do macro `query_as!` que compila offline, exigindo que você intere o uso passivo da lib subjacente e aplique a variação `QueryBuilder` do SQLx. Mantenha os Bindings tipados (`.bind(valor)`) sempre ativados em todo loop de montagem do WHERE para imunizar qualquer inserção perigosa pela UI (SQL Injection Alert). Atenção as querys de Cor: Traga de volta o join complexo atual do Mundam para cá intacto!
