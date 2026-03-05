# Relatório Técnico — Modularização do Backend (Indexação, Metadados, Thumbnails e Evolução para Plugins)

## 1) Estado da base e sincronização

- Repositório analisado: `/workspace/Mundam`.
- Branch atual: `work`.
- Tentativa de atualização para a última versão: `git pull --ff-only`.
- Resultado: **não foi possível atualizar automaticamente**, pois a branch local não possui upstream configurado.

> Recomendação operacional imediata:
>
> 1. Configurar upstream (`git branch --set-upstream-to=<remote>/<branch> work`), ou
> 2. Informar explicitamente remoto/branch em cada pull.

---

## 2) Diagnóstico da arquitetura atual

A base atual já evoluiu bastante e já contém peças importantes para o que você descreveu:

- Um **registro central de formatos** com tipo de mídia e estratégias (`formats/definitions.rs`, `formats/types.rs`).
- Pipeline de **indexação inicial + watcher** (`indexer/scan.rs`, `indexer/watcher.rs`).
- Camada de **extração de preview/thumbnail** com diversos extratores especializados (`thumbnails/mod.rs`, `thumbnails/extractors/mod.rs`).
- Persistência consolidada em `db/*` com operações de assets, folders, tags, busca e configurações.

### 2.1 Pontos fortes já existentes

1. **Registro de formatos centralizado**: a ideia de mapear extensão/MIME -> comportamento já existe e é um ótimo ponto de ancoragem para evolução plugável.
2. **Separação básica por domínio técnico**: indexer, thumbnails, protocols, media, transcoding, db.
3. **Pipeline do watcher em fases** (parse/classificação/persistência/emit), que já facilita evolução.
4. **Fallbacks robustos** de thumbnail/preview para cenários difíceis (raw, ffmpeg, ícone, extração binária).

### 2.2 Gargalos e acoplamentos que dificultam manutenção

1. **Lógica de extração concentrada em match por extensão**:
   - Há muito comportamento condicional em `thumbnails/extractors/mod.rs`, que tende a crescer e virar ponto único de conflito.
2. **Mistura de responsabilidades no indexador**:
   - `scan.rs` e `watcher.rs` combinam descoberta de FS, heurística de rename, extração de metadado, persistência e emissão de eventos.
3. **Modelo de metadados ainda “genérico demais” para formatos avançados**:
   - `get_asset_metadata` preenche metadados básicos (dimensão/size/timestamp), mas formatos como áudio, vídeo, PSD/AI, 3D, fontes e archives exigem campos específicos.
4. **Ausência de contratos explícitos de capacidades por tipo de arquivo**:
   - Hoje existe strategy enum, mas não uma API formal de “capabilities” (metadata, thumbnail, preview, index hints, validation, etc.).
5. **Operações de gestão de arquivos/pastas dispersas**:
   - Existem operações de persistência e de reação ao watcher, porém falta uma API de domínio coesa para rename/move/delete/reindex com idempotência e auditoria.

---

## 3) Proposta de arquitetura alvo (modular + extensível)

## Visão geral

Criar um **núcleo orientado a capabilities** com registries, onde cada formato (ou família de formatos) implementa contratos padronizados.

### 3.1 Módulos propostos

```text
src-tauri/src/
  backend/
    core/
      asset.rs                # tipos centrais (AssetId, AssetPath, hashes, enums)
      errors.rs               # erro unificado por domínio
      context.rs              # contexto de execução (db, cfg, tools)
    registry/
      format_registry.rs      # resolve handler por assinatura/extensão/mime
      capability_registry.rs  # catálogo de capacidades por handler
    extract/
      metadata/
        mod.rs
      thumbnail/
        mod.rs
      preview/
        mod.rs
      fingerprint/
        mod.rs                # hash, frame-key, etc
    indexing/
      scanner.rs
      watcher.rs
      change_classifier.rs
      orchestrator.rs
    filesystem/
      commands.rs             # rename/move/delete/copy (safe)
      transactions.rs         # simulação/rollback lógico/auditoria
    plugins/
      abi.rs                  # contratos e versão
      loader.rs               # carregamento e validação
      sandbox.rs              # políticas/permissões/timeout
```

> Observação prática: não é necessário “big bang rewrite”. Dá para migrar incrementalmente começando por `thumbnails/extractors` e `indexer/metadata`.

---

## 4) Design de APIs por tipo de arquivo (sua ideia, formalizada)

A melhor forma de evitar crescimento desordenado é separar:

1. **Tipo de mídia/família** (Image, Video, Audio, Project, Archive, Model3D, Font, Document, Unknown).
2. **Formato específico** (ex.: psd, ai, zip, mp4, glb).
3. **Capacidades implementadas** (metadados, thumbnail, preview, waveform, text-extract, etc).

### 4.1 Contratos-base (traits) sugeridos

```rust
pub enum AssetFamily {
    Image,
    Video,
    Audio,
    Project,
    Archive,
    Model3D,
    Font,
    Document,
    Unknown,
}

pub struct DetectResult {
    pub format_id: &'static str,      // "image.jpeg", "project.psd", ...
    pub family: AssetFamily,
    pub confidence: f32,
}

pub trait FormatDetector: Send + Sync {
    fn detect(&self, input: &ProbeInput) -> Option<DetectResult>;
}

pub trait MetadataExtractor: Send + Sync {
    fn extract(&self, ctx: &ExtractContext, input: &AssetRef) -> Result<AssetMetadataEnvelope, ExtractError>;
}

pub trait ThumbnailProvider: Send + Sync {
    fn generate(&self, ctx: &ThumbContext, input: &AssetRef, req: &ThumbRequest) -> Result<ThumbResult, ExtractError>;
}

pub trait PreviewProvider: Send + Sync {
    fn build(&self, ctx: &PreviewContext, input: &AssetRef, req: &PreviewRequest) -> Result<PreviewResult, ExtractError>;
}

pub trait IndexHintsProvider: Send + Sync {
    fn index_hints(&self, input: &AssetRef) -> IndexHints;
}
```

### 4.2 Envelope de metadados (core + extensões)

```rust
pub struct AssetMetadataEnvelope {
    pub core: CoreMetadata,                // comum: path, size, time, kind
    pub technical: serde_json::Value,      // codec, bitdepth, fps, channels...
    pub semantic: serde_json::Value,       // ex: nº de páginas, artboards, camadas
    pub diagnostics: Vec<ExtractionNote>,  // warnings/erros parciais
}
```

Esse modelo permite salvar dados comuns para busca/listagem e dados específicos por formato sem quebrar schema a cada novo tipo.

### 4.3 Registry de handlers

- `FormatRegistry` mantém lista ordenada por prioridade.
- Cada handler declara:
  - formatos que suporta,
  - capacidades que implementa,
  - custo estimado (rápido/pesado),
  - timeout sugerido.
- O orchestrator escolhe pipeline com fallback determinístico.

---

## 5) API de indexação e operações de arquivo/pasta

Você citou um ponto crucial: indexar não basta; é preciso operação de ciclo de vida do ativo.

### 5.1 Separar “detecção de mudança” de “aplicação de mudança”

Hoje isso está parcialmente misturado no watcher.

#### Proposta

- `ChangeDetector` (filesystem events -> `DetectedChange`)
- `ChangeClassifier` (`DetectedChange` + heurísticas + db snapshot -> `DomainChange`)
- `ChangeApplier` (`DomainChange` -> transações DB + side effects)

### 5.2 API de comandos de filesystem (domínio)

Criar serviço explícito (invocado por UI, automações e plugins internos):

- `rename_asset(id, new_name)`
- `move_asset(id, target_folder_id)`
- `delete_asset(id, mode: soft|hard)`
- `rename_folder(id, new_name)`
- `move_folder(id, target_parent_id)`
- `reindex_path(path, depth, mode)`
- `refresh_asset_capabilities(id)`

Cada comando deve:

1. Validar pré-condições,
2. Executar FS + DB com estratégia consistente,
3. Emitir evento de domínio único (para UI),
4. Registrar auditoria/log técnico.

### 5.3 Idempotência e reconciliação

Para watcher e operações manuais não se atropelarem:

- Adicionar `operation_id` por comando.
- Armazenar `source` da mudança (`watcher`, `user_command`, `plugin`).
- Reconciliar conflitos por versão (`asset_version`) e timestamp lógico.

---

## 6) Modelo de plugin (evolução futura)

Você pode começar com “plugins internos” (compilados no binário) e evoluir para externos.

### 6.1 Fases recomendadas

1. **Fase A — Internal plugin architecture**
   - Traits + registry + handlers em crates internas.
2. **Fase B — Dynamic loading opcional**
   - Plugins `.so/.dll/.dylib` com ABI versionada.
3. **Fase C — Sandbox e permissões**
   - Limites de CPU/memória/tempo + permissões de I/O.

### 6.2 Contrato de plugin

- `plugin_id`, `plugin_version`, `api_version`.
- Declaração de capacidades e formatos suportados.
- Healthcheck e benchmark básico na carga.
- Política de fallback: se plugin falhar, motor core não quebra.

---

## 7) Plano de migração incremental (baixo risco)

### Etapa 1 — Fundacional (curta)

- Introduzir `AssetFamily` + `Capabilities` explícitas.
- Criar interfaces de `MetadataExtractor`, `ThumbnailProvider`, `PreviewProvider`.
- Implementar registry simples.

### Etapa 2 — Thumbnail/Preview primeiro

- Extrair `thumbnails/extractors/mod.rs` para handlers por formato/família.
- Manter comportamento atual com adaptadores para evitar regressão.

### Etapa 3 — Metadados especializados

- Evoluir `indexer/metadata.rs` para pipeline por família:
  - image/video/audio/document/project/3d/font/archive.
- Persistir `core_metadata` + `technical_metadata` (JSON).

### Etapa 4 — Indexação orquestrada

- Refatorar watcher para `detector -> classifier -> applier`.
- Centralizar emissão de eventos pós-commit.

### Etapa 5 — File operations API

- Criar camada de comandos com idempotência e auditoria.
- Integrar UI/IPC para usar comandos explícitos ao invés de ações ad-hoc.

### Etapa 6 — Plugin externo (opcional)

- Publicar SDK mínimo.
- Carregamento assinado e feature flag por ambiente.

---

## 8) Recomendações de schema e persistência

### 8.1 Tabelas/colunas sugeridas

- `assets`
  - manter colunas essenciais para filtros rápidos,
  - adicionar `family`, `format_id`, `content_hash` (opcional), `version`.
- `asset_metadata_blob`
  - `asset_id`, `technical_json`, `semantic_json`, `updated_at`.
- `asset_operations_log`
  - trilha de auditoria (`operation_id`, `type`, `status`, `source`, `error`).

### 8.2 Estratégia de cache

- Cache de metadados por `mtime + size + inode` (quando disponível).
- Cache de thumbnail por `asset_id + rendition_key` (size, mode, theme).
- Invalidar por eventos de rename/move/edit detectados no watcher.

---

## 9) Organização de código sugerida (na base atual)

Sem quebrar tudo, recomendo este recorte inicial:

1. `src-tauri/src/formats/*`
   - manter como “catálogo de formatos”,
   - adicionar mapeamento para `AssetFamily` e `capabilities`.
2. `src-tauri/src/thumbnails/*`
   - separar roteamento (orquestração) de implementação de extratores,
   - criar subpastas por família (`image`, `project`, `archive`, `raw`, `3d`, `font`).
3. `src-tauri/src/indexer/*`
   - mover `get_asset_metadata` para módulo `extract/metadata`.
4. `src-tauri/src/library/commands/*`
   - manter apenas interface IPC,
   - delegar regra de negócio para serviços (`backend/services/*`).

---

## 10) Backlog técnico priorizado

### Prioridade Alta

1. Definir contratos de capabilities e registrar handlers.
2. Remover `match` monolítico por extensão de `extractors/mod.rs`.
3. Introduzir envelope de metadados extensível.
4. Criar `FileSystemCommandService` (rename/move/delete/reindex).

### Prioridade Média

1. Unificar política de retry/timeout/cancelamento por operação pesada.
2. Melhorar observabilidade (métricas por formato, tempo de extração, taxa de fallback).
3. Criar testes de contrato por formato (golden files).

### Prioridade Baixa

1. Carregamento de plugins externos.
2. Sandbox isolado para extratores não confiáveis.

---

## 11) Critérios de sucesso (KPIs técnicos)

- **Tempo de onboarding de novo formato**: reduzir para “1 handler + 1 registro”.
- **Taxa de regressão em thumbnails/previews**: reduzir via testes de contrato.
- **Tempo médio de indexação incremental**: melhorar com cache de metadados.
- **Confiabilidade do watcher**: menor taxa de eventos órfãos/conflitos.
- **Acoplamento**: queda de mudanças transversais ao adicionar formato novo.

---

## 12) Conclusão

Sua intuição está correta: o próximo salto do backend não é “adicionar mais if/match”, e sim **instituir contratos de capabilities + registries + serviços de domínio**.

A base atual já tem pilares excelentes (catálogo de formatos, watcher em fases, extratores especializados). O que falta é formalizar isso como arquitetura extensível, reduzindo acoplamento e preparando terreno para plugins sem reescrever o sistema inteiro.

Se você quiser, no próximo passo eu já posso transformar este relatório em um **plano de implementação sprint-a-sprint**, com diffs sugeridos por arquivo (ordem de refatoração e estimativa de risco por módulo).
