# Sprint 10.4: Migração Completa — Extractor SAI2 (PaintTool SAI v2)

**Status da sprint:** Verificação necessária
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Verificar e garantir paridade completa do extractor de PaintTool SAI v2 (`.sai2`) entre V1 e V2.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/sai2.rs`
- **Tamanho:** 19,559 bytes
- Parse completo do formato SQLite-based do SAI2
- Abre o arquivo `.sai2` como banco SQLite
- Extrai thumbnail da tabela `canvasthumbnail`

### V2 — `mundam-main/src-tauri/src/processing/media/extractors/sai2.rs`
- **Tamanho:** 4,556 bytes (−77% vs V1)
- Implementação parcial ou stub

## Análise de Gap

O formato `.sai2` internamente é um **banco SQLite** embutido no arquivo. O V1 abria o arquivo como SQLite e extraía o thumbnail da tabela `canvasthumbnail`.

| Funcionalidade | V1 | V2 |
|---|---|---|
| Abertura como SQLite | ✅ | ❓ Verificar |
| Query `SELECT * FROM canvasthumbnail` | ✅ | ❓ Verificar |
| Decode JPEG embutido → PNG | ✅ | ❓ Verificar |
| Extract dimensões canvas | ✅ | ❓ Verificar |

## Tarefas

### 1. Auditar sai2.rs da V2

**Status:** Pendente

Abrir e ler o arquivo `src-tauri/src/processing/media/extractors/sai2.rs` completo e comparar com:
- `mundam-main/src-tauri/src/thumbnails/extractors/sai2.rs`

Se o V2 é apenas um stub (4.5KB vs 19.5KB do V1), portar a implementação completa.

### 2. Portar Implementação SQLite do SAI2

**Status:** Pendente (se necessário após auditoria)

**Implementação V1 (referência):**

```rust
// mundam-main/.../extractors/sai2.rs
use rusqlite::{Connection, OpenFlags};

pub fn extract_sai2_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    // SAI2 é um banco SQLite — abrir como banco
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Thumbnail armazenado como BLOB na tabela canvasthumbnail
    let mut statement = connection.prepare(
        "SELECT thumbnail FROM canvasthumbnail LIMIT 1"
    )?;

    let thumbnail_data: Vec<u8> = statement.query_row([], |row| {
        row.get(0)
    })?;

    // O thumbnail é JPEG inline — retornar diretamente
    Ok((thumbnail_data, "image/jpeg".to_string()))
}

pub fn extract_sai2_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Dimensões na tabela canvas
    let (width, height): (u32, u32) = connection.query_row(
        "SELECT width, height FROM canvas LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok((width, height))
}
```

**Nota:** Verificar se o crate `rusqlite` já está nas dependências do V2. Se não, adicionar ao `Cargo.toml`.

### 3. Adicionar extract_sai2_dimensions ao BinaryDesignFormatProvider

**Status:** Dependente da sprint 10.3

Após implementar `extract_sai2_dimensions()`, adicioná-la ao caso `"sai2"` em `MetadataCapability` do `BinaryDesignFormatProvider` (ver sprint 10.3, Tarefa 2).

### 4. Testar com Arquivos Reais

**Status:** Pendente

O SAI2 tem variações de versão. Testar com pelo menos 3 arquivos `.sai2` de versões diferentes para validar compatibilidade.

## Arquivos a Modificar

- `src-tauri/src/processing/media/extractors/sai2.rs` — portar implementação completa se necessário
- `src-tauri/src/processing/media/binary_design_formats.rs` — integrar dimensions (via sprint 10.3)

## Critérios de Aceitação

- [ ] Arquivo `.sai2` gera thumbnail correto
- [ ] Inspector mostra dimensões corretas do canvas SAI2
- [ ] Sem erros de I/O ou panic ao abrir arquivos SAI2 de diferentes versões

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/sai2.rs`
- `mundam-main/src-tauri/Cargo.toml` (dependências — verificar presença do `rusqlite`)
