# Plano de Implementação — Detecção de Arquivos Duplicados

Este documento refina a proposta técnica e o overview para a arquitetura de detecção de duplicados, ajustando rigorosamente o design às diretrizes do Mundam: Arquitetura Hexagonal (Backend), Solid.js Strict Patterns (Frontend) e TSDoc/Rustdoc (Documentação).

## 1. Avaliação de Coerência

Os documentos `gpt-proprosal.md` e `gpt-overview.md` apresentam uma visão conceitual **extremamente aderente** ao Mundam:
- Processamento assíncrono e incremental acionado via watcher/indexer.
- UI leve focada em revisão e triagem.
- Uso correto do **Asset Ledger** para mutações atômicas (preservando o log de auditoria).

**⚠️ Incoerências corrigidas neste plano:**
1. **Arquitetura de Pastas (Rust)**: A proposta sugeria um módulo plano (`mod.rs`, `service.rs`, `commands.rs`). Isso viola as regras da Arquitetura Hexagonal. Todo código deve ser distribuído entre `core`, `feature`, `infra` e `delivery`.
2. **CQRS e Eventos**: A proposta sugeria um `DuplicateService`. No padrão do Mundam, usaremos *Command Handlers*, *Query Handlers* e o *EventBus* central.
3. **Padrões de Nomenclatura e Documentação**: Variáveis sem abreviação (`duplicate_groups` em vez de `dup_groups`) e exigência de `# Errors` no Rustdoc e `@example` no TSDoc não foram citados na proposta original.

---

## 2. Arquitetura Backend (Hexagonal & CQRS)

O módulo de Duplicados não será uma pasta isolada com tudo dentro, mas sim integrado às camadas existentes do sistema no `src-tauri/src/`.

### 2.1. `core/` (Domínio & Portas)
Não há dependência de banco de dados ou Tauri aqui.
- **Entidades (`core/domain/duplicates/`)**: 
  - `DuplicateGroup`, `DuplicateCandidate`, `DuplicateFingerprint`, `DuplicateRuleSet`.
- **Eventos (`core/events/`)**: 
  - `DuplicateGroupCreated`, `DuplicateScanProgressed`.
- **Portas / Traits (`core/ports/`)**:
  - `DuplicateRepository` (para leitura/gravação de fingerprints e grupos).
  - `FingerprintGenerator` (para algoritmos de perceptual hash).

### 2.2. `feature/` (Handers CQRS)
Aqui reside a orquestração do domínio.
- **Command Handlers (`feature/duplicates/commands/`)**:
  - `StartDuplicateScanHandler`: Enfileira a varredura no job system.
  - `ResolveDuplicateGroupHandler`: Encaminha ações de exclusão/merge diretamente para o `TransactionalAssetLedger`.
- **Query Handlers (`feature/duplicates/queries/`)**:
  - `GetDuplicateGroupsHandler`: Acessa a infraestrutura para leitura rápida (Read-Model) dos grupos aguardando revisão.

### 2.3. `infra/` (Adaptadores Ativos)
Implementação das portas (banco de dados e algoritmos).
- **Repositório (`infra/sqlite/duplicates_repository.rs`)**: 
  - Executa as queries SQL (com SQLx) baseadas nas tabelas de fingerprints e candidates.
- **Fingerprint Engine (`infra/hashing/image_fingerprint_provider.rs`)**:
  - Usa a crate `image-rs` nativa via `tokio::task::spawn_blocking` (obrigatório não bloquear o executor).

### 2.4. `delivery/` (Tauri Endpoints)
Endpoints puros, apenas repassam dados para o `feature/`.
- **Comandos (`delivery/tauri/duplicates.rs`)**:
  - Funções decoradas com `#[tauri::command]` como `start_duplicate_scan`, que fazem parse do payload JSON, invocam o Command Handler respectivo e devolvem `AppResult<T>`.

---

## 3. Banco de Dados SQLite

O esquema relacional da proposta original está perfeito e será mantido, injetado via migrações SQLx:
- `duplicate_fingerprints` (Hashes exatos e perceptuais).
- `duplicate_rule_sets` (As configurações definidas pelo usuário).
- `duplicate_groups` (Os grupos detectados baseados na regra).
- `duplicate_candidates` (Os assets que compõem o grupo e seu respectivo score).
- `duplicate_resolutions` (O histórico de ação, para não reprocessar o que foi ignorado).

---

## 4. Arquitetura Frontend (Solid.js)

A prévia em `DuplicateFinderView.tsx` servirá como âncora, mas precisamos seguir as diretrizes rigorosamente para a expansão do recurso.

### Estrutura de Diretórios
Seguiremos a arquitetura baseada em features. Tudo relacionado aos duplicados viverá em `src/components/features/duplicates/`.

```
src/components/features/duplicates/
├── index.ts                     # Public exports + TSDoc Module Comment
├── types.ts                     # Interfaces (sem abreviações)
├── DuplicateComparisonPanel/    # Componentes aninhados
│   ├── DuplicateComparisonPanel.tsx
│   ├── duplicate-comparison-panel.css
│   └── utils.ts                 # Utilitários ESTREITAMENTE locais ao painel
├── DuplicateGroupList/
│   ├── DuplicateGroupList.tsx
│   └── duplicate-group-list.css
└── hooks/
    └── useDuplicateResolution.ts # Lógica complexa extraída
```

### Regras Ouro do Frontend
1. **Nunca usar destructuring de propriedades**: `props` deve ser acessado como `props.header`, para não quebrar a reatividade do Solid.
2. **Nenhuma abreviação**: Variavéis devem se chamar `duplicateGroupList`, não `dupGrpList`.
3. **Lógica Visual**: Uso mandatório de `<Show>`, `<For>` e `<Switch>`. Nada de mapeamento manual com `.map()` ou operadores ternários encadeados para renderização de UI.
4. **Zero Magic Numbers e Cores Hardcoded**: Todo CSS deve puxar do arquivo de tokens (ex: `var(--color-bg-surface-1)`).
5. **Nada de `// === Separadores Visuais ===`**: Se um arquivo (como o `DuplicateFinderView.tsx`) precisar ser dividido visualmente, está na hora de quebrar em mais arquivos.

---

## 5. Tratamento de Erros e Logs

- Backend Rust: NADA DE `unwrap()`. Falhas de leitura de imagens corrompidas ou falha de SQL devem retornar um `AppError::FingerprintGenerationFailed`, repassando adequadamente via Tauri para a UI com um código JSON consumível.
- Usar `tracing::info!` e `tracing::error!` nos Handlers para telemetria. Não usar `println!`.

---

## 6. Documentação (Obrigatório)

Todos os novos arquivos deverão seguir estritamente o `documentation.md`.

### TypeScript (`types.ts` e `index.ts`)
Uso de TSDoc, SEMPRE em inglês, explicando o *por que* e incluindo `@example`.

```tsx
/**
 * Resolves a duplicate group by sending a command to the Asset Ledger.
 *
 * @param {string} duplicateGroupId - The unique identifier of the duplicate group.
 * @param {ResolutionAction} action - The action chosen by the user (keep, delete, merge).
 * @returns {Promise<void>} Resolves when the command has been accepted.
 *
 * @example
 * ```tsx
 * await resolveDuplicateGroup('group-uuid', 'delete_others');
 * ```
 */
```

### Rust (Em todos os `.rs` modificados)
Uso de `rustdoc` com a seção `# Errors`.

```rust
/// Calcula o fingerprint perceptual de um Asset usando spawn_blocking para não travar o Tokio.
///
/// # Arguments
/// * `asset_path` - Caminho absoluto da mídia extraída do banco de dados.
///
/// # Errors
/// Retorna `AppError::InvalidMediaFormat` caso o binário da imagem não possa ser decodificado.
```

---

## 7. Próximos Passos (Workflow Sugerido)

1. **Migrações Banco de Dados:** Criar os arquivos `.sql` de migração para as 5 tabelas.
2. **Core e Infraestrutura (Rust):** Implementar as structs de domínio e o repositório SQLite `infra/sqlite/duplicates_repository.rs`.
3. **Eventos e Handlers (Rust):** Implementar os listeners que reagem aos eventos do Indexador e geram os fingerprints via `spawn_blocking`.
4. **Tauri Delivery:** Expor as rotas de queries (listar grupos) e commands (resolver grupos via Asset Ledger).
5. **Frontend (Solid):** Refinar `DuplicateFinderView.tsx` conectando-o aos endpoints reais através de hooks (ex: `useDuplicateGroups.ts`).
