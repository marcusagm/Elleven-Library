# Proposta Técnica — Detecção e Revisão de Duplicados no Mundam

## 1. Objetivo

Adicionar ao Mundam um subsistema de detecção de duplicados capaz de identificar:

* arquivos idênticos
* imagens visualmente equivalentes após recompressão, resize ou rotação
* imagens derivadas por crop, recorte, espelhamento ou pequena edição
* grupos configuráveis de “duplicado” conforme a regra escolhida pelo usuário

A solução deve operar de forma incremental, integrada ao indexador atual, sem bloquear a UI, e com trilha de decisão persistida no banco.

---

## 2. Princípio de arquitetura

A deduplicação deve ser tratada como um pipeline paralelo ao indexador principal, mas acoplado a ele por eventos de domínio.

### Fluxo base

1. O watcher detecta criação, alteração, rename ou remoção.
2. O indexador atualiza o `asset`.
3. O pipeline de fingerprint é enfileirado.
4. O matcher consulta candidatos e forma grupos.
5. O frontend recebe eventos e abre a interface de revisão.
6. O usuário decide a ação.
7. O Asset Ledger aplica a mutação final e grava o log.

Esse desenho preserva a separação já existente entre indexação, worker em background, banco e UI.

---

## 3. Módulos Rust propostos

### 3.1 Novo módulo de domínio

`src-tauri/src/duplicates/`

Responsável por tudo relacionado à detecção e resolução de duplicados.

#### Estrutura sugerida

```text
src-tauri/src/duplicates/
├── mod.rs
├── commands.rs
├── events.rs
├── model.rs
├── service.rs
├── matcher.rs
├── fingerprints.rs
├── rules.rs
├── score.rs
├── repository.rs
├── jobs.rs
└── tests/
```

### 3.2 Responsabilidades por arquivo

#### `model.rs`

Define as entidades centrais:

* `DuplicateGroup`
* `DuplicateCandidate`
* `DuplicateFingerprint`
* `DuplicateRuleSet`
* `DuplicateResolution`
* `DuplicateAction`

#### `fingerprints.rs`

Gera e atualiza os fingerprints do asset:

* `content_hash` para igualdade exata
* `perceptual_hash` para similaridade visual
* `block_hash` ou fingerprint em múltiplas escalas para crops e edições
* `thumb_hash` para checagem rápida baseada em thumbnail

#### `matcher.rs`

Agrupa candidatos com base em:

* hash exato
* buckets de similaridade perceptual
* regras configuráveis de corte, resize e rotação
* score final de confiança

#### `rules.rs`

Guarda a configuração de “o que conta como duplicado”:

* considerar crop como duplicado
* ignorar diferenças de dimensão
* aceitar compressão/reexportação
* aceitar espelhamento
* exigir mesma família de mídia
* restringir por pasta ou biblioteca

#### `score.rs`

Calcula a pontuação final do grupo e dos candidatos.

#### `repository.rs`

Camada de persistência SQLx para:

* fingerprints
* grupos
* candidatos
* resoluções
* regras

#### `service.rs`

Orquestra o fluxo:

* recebe evento do indexer
* enfileira job
* gera fingerprint
* chama matcher
* persiste grupo
* emite evento para UI

#### `commands.rs`

Expose Tauri commands para a interface.

#### `events.rs`

Padroniza os eventos Tauri disparados para o frontend.

#### `jobs.rs`

Define jobs assíncronos para:

* fingerprint
* scan incremental
* rescan completo
* recomputação por regra

---

## 4. Integração com a arquitetura atual

O Mundam já inicializa banco, watcher, thumbnails, streaming e registra comandos Tauri em `src-tauri/src/lib.rs`. A deduplicação deve entrar no mesmo ciclo de bootstrap, como mais um serviço gerenciado pelo lifecycle registry.

### Ponto de encaixe

No startup:

* criar `DuplicateService`
* registrar no `LifecycleRegistry`
* inicializar estado compartilhado no `app.handle().manage(...)`
* adicionar comandos em `invoke_handler(...)`
* ligar eventos ao frontend

### Integração com o Asset Ledger

O documento do Asset Ledger define que mutações de domínio devem passar por um core transacional, com validação, atomicidade e log de auditoria. Então, ações como “excluir”, “manter”, “mesclar metadados” e “ignorar grupo” devem ser comandos do Ledger, não mutações diretas da UI.

---

## 5. Tabelas SQLite propostas

### 5.1 `duplicate_fingerprints`

Guarda fingerprints normalizados por asset.

```sql
CREATE TABLE duplicate_fingerprints (
    asset_id TEXT PRIMARY KEY,
    content_hash TEXT,
    perceptual_hash TEXT,
    block_hash TEXT,
    thumb_hash TEXT,
    width INTEGER,
    height INTEGER,
    file_size INTEGER,
    mime_type TEXT,
    format_family TEXT,
    color_profile TEXT,
    orientation INTEGER,
    fingerprint_version INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE
);
```

### 5.2 `duplicate_rule_sets`

Armazena perfis de detecção.

```sql
CREATE TABLE duplicate_rule_sets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    consider_exact_match INTEGER NOT NULL DEFAULT 1,
    consider_visual_match INTEGER NOT NULL DEFAULT 1,
    consider_crop_match INTEGER NOT NULL DEFAULT 0,
    ignore_resolution_difference INTEGER NOT NULL DEFAULT 1,
    ignore_recompression INTEGER NOT NULL DEFAULT 1,
    allow_rotation INTEGER NOT NULL DEFAULT 1,
    allow_mirroring INTEGER NOT NULL DEFAULT 0,
    min_score REAL NOT NULL DEFAULT 0.85,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### 5.3 `duplicate_groups`

Agrupa assets detectados como parte de um mesmo conjunto.

```sql
CREATE TABLE duplicate_groups (
    id TEXT PRIMARY KEY,
    rule_set_id TEXT NOT NULL,
    group_type TEXT NOT NULL, -- exact | near | derived
    canonical_asset_id TEXT,
    confidence REAL NOT NULL,
    status TEXT NOT NULL, -- open | reviewed | ignored | resolved
    candidate_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(rule_set_id) REFERENCES duplicate_rule_sets(id)
);
```

### 5.4 `duplicate_candidates`

Lista os membros do grupo e o score individual.

```sql
CREATE TABLE duplicate_candidates (
    group_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    score REAL NOT NULL,
    reasons TEXT NOT NULL, -- JSON array
    is_selected INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(group_id, asset_id),
    FOREIGN KEY(group_id) REFERENCES duplicate_groups(id) ON DELETE CASCADE,
    FOREIGN KEY(asset_id) REFERENCES assets(id) ON DELETE CASCADE
);
```

### 5.5 `duplicate_resolutions`

Registra a decisão final do usuário.

```sql
CREATE TABLE duplicate_resolutions (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    action TEXT NOT NULL, -- keep_one | delete_selected | merge_metadata | ignore_group | custom
    selected_asset_id TEXT,
    payload TEXT, -- JSON com detalhes da decisão
    resolved_by TEXT,
    resolved_at TEXT NOT NULL,
    FOREIGN KEY(group_id) REFERENCES duplicate_groups(id) ON DELETE CASCADE
);
```

### 5.6 Índices recomendados

```sql
CREATE INDEX idx_duplicate_fingerprints_content_hash ON duplicate_fingerprints(content_hash);
CREATE INDEX idx_duplicate_fingerprints_phash ON duplicate_fingerprints(perceptual_hash);
CREATE INDEX idx_duplicate_groups_status ON duplicate_groups(status);
CREATE INDEX idx_duplicate_candidates_asset_id ON duplicate_candidates(asset_id);
```

---

## 6. Eventos Tauri propostos

### Eventos de processamento

* `duplicate:fingerprint-started`
* `duplicate:fingerprint-ready`
* `duplicate:scan-started`
* `duplicate:scan-progress`
* `duplicate:scan-finished`
* `duplicate:group-created`
* `duplicate:group-updated`
* `duplicate:group-resolved`
* `duplicate:group-ignored`

### Payloads sugeridos

#### `duplicate:group-created`

```json
{
  "group_id": "uuid",
  "group_type": "derived",
  "confidence": 0.94,
  "canonical_asset_id": "asset_123",
  "candidate_count": 4,
  "rule_set_id": "rule_default"
}
```

#### `duplicate:scan-progress`

```json
{
  "processed": 1200,
  "matched": 38,
  "groups_created": 7,
  "estimated_remaining": 4
}
```

### Eventos de comando e resolução

* `duplicate:resolve-group-requested`
* `duplicate:resolve-group-completed`
* `duplicate:refresh-requested`
* `duplicate:reindex-requested`

---

## 7. Tauri commands propostos

### Leitura

* `duplicates::get_duplicate_groups(filters)`
* `duplicates::get_duplicate_group(group_id)`
* `duplicates::get_duplicate_candidates(group_id)`
* `duplicates::get_duplicate_rule_sets()`
* `duplicates::get_duplicate_scan_status()`
* `duplicates::get_duplicate_stats()`

### Escrita / mutação

* `duplicates::start_duplicate_scan(rule_set_id?)`
* `duplicates::rescan_asset(asset_id)`
* `duplicates::rescan_selection(asset_ids)`
* `duplicates::create_duplicate_rule_set(payload)`
* `duplicates::update_duplicate_rule_set(payload)`
* `duplicates::delete_duplicate_rule_set(rule_set_id)`

### Resolução

Essas operações devem passar pelo Asset Ledger:

* `duplicates::keep_asset(group_id, asset_id)`
* `duplicates::delete_assets(group_id, asset_ids)`
* `duplicates::merge_metadata(group_id, target_asset_id, source_asset_ids)`
* `duplicates::ignore_group(group_id)`
* `duplicates::mark_group_reviewed(group_id, payload)`

---

## 8. Regras de detecção

### 8.1 Duplicado exato

Critério:

* `content_hash` igual

Uso:

* cópias idênticas
* arquivos renomeados
* clones gerados por backup/importação

### 8.2 Duplicado visual

Critério:

* `perceptual_hash` muito próximo
* dimensões compatíveis
* score acima do limite

Uso:

* reexportação
* compressão diferente
* pequenas edições de brilho/cor

### 8.3 Duplicado derivado

Critério:

* sobreposição parcial significativa
* correlação de blocos
* relações de aspecto compatíveis
* score acima de um limiar menor, mas exigindo revisão manual

Uso:

* crop
* canvas expandido
* versão com área removida
* imagem com sobreposição de elementos

### 8.4 Regras por contexto

O usuário deve poder criar perfis como:

* “Somente exatos”
* “Exatos + visuais”
* “Exatos + visuais + crops”
* “Modo rigoroso para RAW/PSD”
* “Modo agressivo para limpeza de biblioteca”

---

## 9. Pipeline de execução

### Evento de filesystem

`Indexer -> DuplicateService`

Quando o indexer detectar um asset novo ou alterado, ele deve emitir algo equivalente a:

```rust
DuplicateJob::AssetIndexed {
    asset_id,
    path,
    media_kind,
    fast_mode: true
}
```

### Etapas internas

1. ler metadados do asset
2. gerar hash exato
3. gerar fingerprint visual
4. consultar candidatos por bucket
5. calcular score
6. criar ou atualizar grupo
7. emitir evento para UI

### Jobs assíncronos

* `GenerateFingerprints`
* `FindDuplicateCandidates`
* `BuildDuplicateGroup`
* `ResolveDuplicateGroup`
* `RecomputeRuleSet`
* `ReindexForDuplicateDetection`

---

## 10. Estrutura de comandos no `src-tauri/src/lib.rs`

No `invoke_handler(...)`, adicionar algo como:

```rust
duplicates::commands::get_duplicate_groups,
duplicates::commands::get_duplicate_group,
duplicates::commands::get_duplicate_candidates,
duplicates::commands::get_duplicate_rule_sets,
duplicates::commands::start_duplicate_scan,
duplicates::commands::rescan_asset,
duplicates::commands::keep_asset,
duplicates::commands::delete_assets,
duplicates::commands::merge_metadata,
duplicates::commands::ignore_group,
duplicates::commands::create_duplicate_rule_set,
duplicates::commands::update_duplicate_rule_set,
duplicates::commands::delete_duplicate_rule_set,
```

Também registrar o estado compartilhado:

* `DuplicateService`
* `DuplicateScanState`
* `DuplicateRuleRegistry`

---

## 11. Interface de usuário proposta

A UI ideal não deve ser só uma lista de resultados. Deve ser uma tela de triagem especializada.

### 11.1 Página principal

`DuplicatesPage`

Seções:

* resumo geral
* filtros
* lista de grupos
* painel de comparação
* ações em lote

### 11.2 Lista de grupos

Cada card de grupo deve exibir:

* miniatura principal
* quantidade de candidatos
* score médio
* tipo do grupo
* regras aplicadas
* status da revisão

### 11.3 Painel de comparação

Ao abrir um grupo:

* grid com todos os assets do grupo
* destaque no asset canônico
* metadados lado a lado
* dimensão, tamanho, data, hash, caminho
* zoom sincronizado para análise visual
* overlay opcional para comparação pixel a pixel

### 11.4 Ações rápidas

Botões:

* manter este
* manter o maior
* manter o mais antigo
* excluir os demais
* mesclar metadados
* ignorar grupo
* marcar como revisado

### 11.5 Comparação avançada

Para grupos com crop/modificação leve:

* visor lado a lado
* overlay com transparência
* ajuste de escala sincronizado
* recorte detectado destacado
* score de semelhança explicado em linguagem simples

---

## 12. Componentes frontend sugeridos

```text
src/
├── features/
│   └── duplicates/
│       ├── DuplicatesPage.tsx
│       ├── DuplicateGroupList.tsx
│       ├── DuplicateGroupCard.tsx
│       ├── DuplicateComparisonPanel.tsx
│       ├── DuplicateRulesDialog.tsx
│       ├── DuplicateScanStatus.tsx
│       ├── DuplicateResolutionActions.tsx
│       └── hooks/
│           ├── useDuplicateGroups.ts
│           ├── useDuplicateSelection.ts
│           └── useDuplicateRules.ts
```

---

## 13. Estados de UX

### Estados principais

* `idle`
* `scanning`
* `building_groups`
* `needs_review`
* `resolved`
* `ignored`

### Estados por grupo

* `open`
* `reviewed`
* `ignored`
* `resolved`

A UI deve deixar claro quando o sistema “achou candidatos” e quando realmente “há duplicado confirmado”.

---

## 14. Auditoria e rastreabilidade

Toda decisão do usuário deve ser registrada com:

* grupo afetado
* regra usada
* asset mantido
* assets descartados
* data/hora
* origem da ação
* payload completo da decisão

Isso é importante porque deduplicação altera o acervo e precisa ser reversível ou pelo menos auditável.

---

## 15. Estratégia de implantação

### Fase 1

Duplicado exato por `content_hash`.

### Fase 2

Similaridade visual por `perceptual_hash`.

### Fase 3

Detecção de crop e variação derivada.

### Fase 4

UI completa de triagem com resolução em lote.

### Fase 5

Regras avançadas configuráveis por perfil.

---

## 16. Recomendação final

A implementação mais robusta para o Mundam é:

* **motor incremental no backend**
* **fingerprints persistidos em SQLite**
* **matcher configurável por regra**
* **eventos Tauri para atualização reativa da UI**
* **Asset Ledger como autoridade das mutações**
* **tela especializada de revisão com comparação visual**

Isso preserva a filosofia atual do projeto, aproveita a base de indexação e thumbnails, e transforma deduplicação em uma função de produtividade séria, não em um “botão de limpeza” simples.

## 17. Nome dos módulos e comandos, prontos para usar

### Módulo Rust

`src-tauri/src/duplicates/`

### Tabelas

* `duplicate_fingerprints`
* `duplicate_rule_sets`
* `duplicate_groups`
* `duplicate_candidates`
* `duplicate_resolutions`

### Eventos Tauri

* `duplicate:group-created`
* `duplicate:group-updated`
* `duplicate:group-resolved`
* `duplicate:scan-progress`
* `duplicate:scan-finished`

### Commands

* `get_duplicate_groups`
* `get_duplicate_group`
* `start_duplicate_scan`
* `keep_asset`
* `delete_assets`
* `merge_metadata`
* `ignore_group`
* `create_duplicate_rule_set`
* `update_duplicate_rule_set`
