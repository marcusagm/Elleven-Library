# Refatoração: Images -> Assets & Integração FileFormat::detect

## 🎯 Objetivo
Transformar a arquitetura atual (orientada apenas a "images") em um verdadeiro modelo DAM (Digital Asset Management), suportando e armazenando de forma padronizada os metadados de qualquer tipo de arquivo (vídeo, áudio, documentos, 3D, projetos). 

Nesta mesma etapa, encerramos a issue do roadmap referente à invocação da heurística avançada `crate::formats::FileFormat::detect` no indexer.

## 🗄️ Cenário Atual vs 🚀 Cenário Desejado

**Problemas atuais:**
1. A tabela principal chama-se `images`. A estrutura relacionada no Rust chama-se `ImageMetadata`.
2. Apenas a extensão é extraída do nome do arquivo (`path.extension()`) e salva na coluna `format`. Se um arquivo TIF for salvo como ".tiff" ou ".TIF", ele entra fragmentado.

**Nova Arquitetura:**
1. Tabela renomeada para `assets`.
2. Tabela `image_tags` renomeada para `asset_tags`.
3. Modelos em Rust e Typescript alterados para `AssetMetadata`.
4. Adição da coluna `media_type` em `assets` para gravar a categoria master da mídia (Ex: `Image`, `Video`, `Project`, etc.).
5. Uso do `FileFormat::detect` no indexador persistindo sempre a extensão canônica (primeira extensão do registro) no banco de dados.

---

## 📅 Fases de Implementação (Checklist)

### Fase 1: Atualização das Migrations e Migração Genérica (Banco)
- [x] Editar `src-tauri/migrations/20260210000000_initial_schema.sql` substituindo os domínios `images` e associados para `assets`.
- [x] Adicionar coluna `media_type TEXT NOT NULL` na instrução `CREATE TABLE assets`.
- [x] Revisitar triggers FTS e índices renomeando de `images_fts` e `idx_images_*` para `assets_fts` e `idx_assets_*`.
- [x] Adaptar arquivo complementar de índices (`20260210000001_add_performance_indices.sql`).

### Fase 2: Refatoração de Nomes no Backend (Rust)
- [x] Renomear o modelo `ImageMetadata` para `AssetMetadata` em `db/models.rs`. Modificar a propriedade `total_images` para `total_assets`.
- [x] Renomear fisicamente `src-tauri/src/db/images.rs` para `src-tauri/src/db/assets.rs` e mudar as referências em `db/mod.rs`.
- [x] Fazer as substituições textuais (`images` -> `assets`) em todo o SQL dentro de `folders.rs`, `search.rs` e `assets.rs`.
- [x] Fazer substituições nos nomes das funções Tauri (ex: `get_images_filtered` para `get_assets_filtered`).

### Fase 3: Conexão do formato `FileFormat::detect`
- [x] Modificar `get_image_metadata` em `indexer/metadata.rs` para que retorne `Option<AssetMetadata>`. 
- [x] Renomear a função para `get_asset_metadata`.
- [x] Dentro dela, chamar `FileFormat::detect(path)` e:
  - Definir que `media_type` seja populado.
  - Definir que `format` seja populado com `format.extensions.first()`.
  - Interromper (return `None`) a leitura se o formato falhar.

### Fase 4: O Refatoração do Frontend (TypeScript)
- [x] Atualizar o modelo Typescript (`ImageMetadata` -> `AssetMetadata`).
- [x] Modificar interfaces referenciando os Comandos Tauri alterados (invoke commands).

---

### Considerações Finais
As substituições de textos (como `image`/`images` para `asset`/`assets`) também foram concluídas em comentários e nomenclatura de variáveis em todos os arquivos auxiliares referenciados, incluindo a correção da diretiva de DND (`IMAGE` para `ASSET`). Todos os bugs listados de runtime e de compilação apontados pelo Typescript/Cargo foram corrigidos. A partir de agora, o projeto roda sobre a tabela `assets` no SQLite perfeitamente.
