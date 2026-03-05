# Relatório Técnico — Modularização do Backend (Estratégia por Biblioteca de Formatos + Backend Paralelo)

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

## 2) Diagnóstico da base atual

A arquitetura atual já possui blocos importantes para evolução:

- Catálogo central de formatos e estratégias (`formats/definitions.rs`, `formats/types.rs`).
- Pipeline de indexação inicial + monitoramento (`indexer/scan.rs`, `indexer/watcher.rs`).
- Camada de thumbnails/previews com diversos extratores (`thumbnails/mod.rs`, `thumbnails/extractors/mod.rs`).
- Persistência robusta no `db/*`.

### Pontos fortes

1. Boa base de detecção/classificação de formatos.
2. Watcher com fases lógicas (parse/classificação/persistência/emissão).
3. Estratégias de fallback já maduras (ffmpeg, raw, ícone, extração binária).

### Pontos que travam manutenção

1. `match` crescente por extensão em pontos críticos.
2. Mistura de responsabilidades entre descoberta, extração, persistência e eventos.
3. Falta de contrato explícito por **capacidade** (metadado, thumbnail, preview, index hints).
4. Falta de uma API de domínio única para operações FS (rename/move/delete/reindex) com idempotência.

---

## 3) Ajuste principal solicitado: formatos como biblioteca, não como blocos isolados

Você está certo: o melhor desenho para escalar é tratar formatos como **biblioteca de formatos** com API padronizada.

## 3.1 Proposta: `format-kit` (biblioteca interna)

Criar uma biblioteca interna (crate/module) que concentra o ecossistema de formatos:

```text
src-tauri/src/
  format_kit/
    api/
      mod.rs                 # traits/contratos
      types.rs               # tipos comuns (AssetFamily, DetectResult, Capability)
    registry/
      mod.rs                 # registro de formatos e resolução
    formats/
      image/jpeg.rs
      image/png.rs
      video/mp4.rs
      project/psd.rs
      archive/zip.rs
      model3d/glb.rs
      font/ttf.rs
      document/pdf.rs
      unknown/default.rs
```

Cada formato vira uma implementação pequena e autocontida, registrada no `registry`.

## 3.2 Contratos da API de formato

```rust
pub trait FormatModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn family(&self) -> AssetFamily;
    fn detect(&self, input: &ProbeInput) -> DetectConfidence;
    fn capabilities(&self) -> &'static [Capability];
}

pub trait MetadataCapability: Send + Sync {
    fn extract_metadata(&self, ctx: &ExtractContext, input: &AssetRef)
        -> Result<AssetMetadataEnvelope, ExtractError>;
}

pub trait ThumbnailCapability: Send + Sync {
    fn generate_thumbnail(&self, ctx: &ThumbContext, input: &AssetRef, req: &ThumbRequest)
        -> Result<ThumbResult, ExtractError>;
}

pub trait PreviewCapability: Send + Sync {
    fn generate_preview(&self, ctx: &PreviewContext, input: &AssetRef, req: &PreviewRequest)
        -> Result<PreviewResult, ExtractError>;
}
```

Com isso, o sistema para de crescer por `match` gigante e passa a crescer por **novos módulos de formato**.

---

## 4) Ajuste principal solicitado: backend novo em paralelo (migração gradual)

Também concordo com seu ponto: em vez de reescrever o backend atual, criar um **backend v2 paralelo** e substituir aos poucos (strangler pattern).

## 4.1 Modelo de convivência V1 + V2

```text
src-tauri/src/
  backend_v1/         # implementação atual (mantida funcional)
  backend_v2/
    application/      # casos de uso
    domain/           # regras e contratos
    infra/            # db/fs/tooling
    format_kit_adapter/
```

### Regras de migração

1. Nada de desligar V1 abruptamente.
2. Todo recurso novo entra primeiro no V2, com feature flag.
3. Fluxos do V1 são migrados por fatia (thumbnails -> metadados -> watcher -> comandos FS).
4. Quando um fluxo atingir paridade + estabilidade, V1 correspondente é removido.

## 4.2 Roteador de execução por feature flag

Criar um roteador simples:

- `BACKEND_USE_V2_THUMBNAILS`
- `BACKEND_USE_V2_METADATA`
- `BACKEND_USE_V2_INDEXER`
- `BACKEND_USE_V2_FS_COMMANDS`

Esse roteador decide em runtime qual engine processa cada operação, reduzindo risco de rollout.

---

## 5) API de capacidades por família de arquivo

Famílias alvo:

- Image
- Video
- Audio
- Project (`.psd`, `.ai`, etc.)
- Archive (`.zip`, etc.)
- Model3D
- Font
- Document (`pdf`, `docx`, `xlsx`, etc.)
- Unknown

Cada módulo de formato pode implementar uma ou mais capacidades:

- Metadata
- Thumbnail
- Preview
- SearchableText (documentos)
- Waveform (áudio)
- StreamProfile (vídeo)
- IntegrityCheck

---

## 6) API de indexação e operações de arquivos/pastas

Separar três etapas de mudança de FS:

1. `ChangeDetector` (eventos crus)
2. `ChangeClassifier` (interpretação de domínio)
3. `ChangeApplier` (persistência + efeitos)

E expor comandos de domínio unificados:

- `rename_asset`
- `move_asset`
- `delete_asset`
- `rename_folder`
- `move_folder`
- `delete_folder`
- `reindex_path`
- `refresh_asset_capabilities`

Cada comando deve carregar:

- `operation_id`
- `source` (`watcher`, `user`, `system`, `plugin`)
- `idempotency_key`

---

## 7) Modelo de dados recomendado

### 7.1 Estrutura principal

- `assets`: manter campos de filtro rápido + `family`, `format_id`, `version`.
- `asset_metadata_blob`: JSON técnico/semântico por asset.
- `asset_operations_log`: auditoria de comandos e reconciliação.

### 7.2 Envelope de metadados

```rust
pub struct AssetMetadataEnvelope {
    pub core: CoreMetadata,
    pub technical: serde_json::Value,
    pub semantic: serde_json::Value,
    pub diagnostics: Vec<ExtractionNote>,
}
```

---

## 8) Plano de implementação incremental (sem ruptura)

## Fase 0 — Fundação de convivência

- Criar `backend_v2` vazio + roteador de feature flag.
- Manter 100% das operações no V1 inicialmente.

## Fase 1 — Biblioteca de formatos (`format-kit`)

- Criar contratos API e registry.
- Implementar 3 formatos piloto (ex.: `jpg`, `png`, `mp4`).
- Integrar V2 só para thumbnail desses pilotos.

## Fase 2 — Metadados V2

- Introduzir `AssetMetadataEnvelope`.
- Migrar extração de metadados por família no V2.
- Rodar em paralelo com V1 e comparar outputs (telemetria).

## Fase 3 — Indexador V2 (parcial)

- Migrar classificação e aplicação de mudanças.
- Manter detector/watcher V1 se necessário no início.

## Fase 4 — Comandos FS V2

- Implementar serviço de comandos transacionais/idempotentes.
- Migrar ações de UI para o serviço único.

## Fase 5 — Expansão por formato

- Adicionar novos módulos de formato por backlog de prioridade.
- Remover blocos equivalentes do V1 após paridade + testes.

## Fase 6 — Desativação progressiva do V1

- Quando cobertura funcional e estabilidade forem confirmadas, remover módulos legados por domínio.

---

## 9) Backlog priorizado (alinhado ao seu feedback)

### Alta prioridade

1. Criar `format-kit` com API estável por formato.
2. Introduzir `backend_v2` paralelo com roteador por feature flag.
3. Migrar thumbnail e metadata para V2 por fatias.
4. Criar `FileSystemCommandService` com idempotência.

### Média prioridade

1. Contratos de teste por formato (golden files).
2. Telemetria de comparação V1 vs V2.
3. Política unificada de timeout/retry/cancelamento.

### Baixa prioridade

1. Loader de plugins externos (ABI versionada).
2. Sandbox de plugins não confiáveis.

---

## 10) KPIs de sucesso

- Tempo para adicionar novo formato: “1 módulo + 1 registro”.
- Redução de regressão em thumbnail/preview após migração.
- Tempo médio de indexação incremental.
- Taxa de conflitos watcher x comandos de usuário.
- Percentual de tráfego processado pelo V2 por domínio.

---

## 11) Conclusão

Direção recomendada (e alinhada ao seu comentário):

1. **Formatos como biblioteca** com API padronizada e implementação isolada por formato.
2. **Backend V2 paralelo** para migração gradual, sem reescrita disruptiva.
3. Substituição progressiva do V1 por domínio até a exclusão completa do backend atual.

Se você quiser, no próximo passo eu posso te entregar um **plano de execução por sprint** já com ordem exata de arquivos para iniciar o `backend_v2` e o `format-kit` sem risco de regressão.
