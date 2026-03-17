# Sprint 10.8: Migração Completa — Extractors Rebelle e Penpot

**Status da sprint:** Pendente
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Verificar e garantir paridade dos extractors Rebelle (`.reb`) e Penpot (`.penpot`) na V2.

## Estado Atual

### Rebelle (`.reb`)

| | V1 | V2 |
|---|---|---|
| Arquivo | `extractors/rebelle.rs` | `extractors/rebelle.rs` |
| Tamanho | 1,298 bytes | 780 bytes (−40%) |

### Penpot (`.penpot`)

| | V1 | V2 |
|---|---|---|
| Arquivo | `extractors/penpot.rs` | `extractors/penpot.rs` |
| Tamanho | 5,219 bytes | 2,976 bytes (−43%) |

## Análise de Gap

### Rebelle

Rebelle (`.reb`) é um formato ZIP-based. O thumbnail fica em `thumbnail.jpg` ou `thumbnail.png` dentro do ZIP. A diferença de 40% pode indicar suporte a menos variações de layout do ZIP.

**V1 approach:**
```rust
pub fn extract_rebelle_preview(path: &Path) -> Result<(Vec<u8>, String)> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Tentar thumbnail.jpg primeiro, depois thumbnail.png
    for thumb_name in &["thumbnail.jpg", "thumbnail.png", "preview.png", "preview.jpg"] {
        if let Ok(mut entry) = archive.by_name(thumb_name) {
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            let mime = if thumb_name.ends_with(".png") { "image/png" } else { "image/jpeg" };
            return Ok((data, mime.to_string()));
        }
    }
    Err("No thumbnail found in Rebelle file".into())
}
```

### Penpot

Penpot (`.penpot`) é também um formato ZIP com uma estrutura específica que contém um thumbnail SVG ou PNG. O arquivo `.penpot` é essencialmente um export do Penpot cloud.

**V1 approach:**
O V1 tinha lógica para encontrar o arquivo `thumbnail.png` ou `thumbnail.svg` dentro do ZIP, além de extrair metadados do arquivo `manifest.json` embutido (dimensões da página, número de frames).

## Tarefas

### 1. Auditar rebelle.rs e penpot.rs V2

**Status:** Pendente

Leia os dois arquivos da V2 e compare com V1 para identificar os casos não cobertos.

### 2. Portar Casos Faltantes — Rebelle

**Status:** Pendente (após auditoria)

Se o V2 não cobre todos os caminhos de thumbnail, portar da V1:
- Múltiplos nomes de thumbnail: `thumbnail.jpg`, `thumbnail.png`, `preview.png`
- Fallback para primeira imagem encontrada no ZIP

### 3. Portar Casos Faltantes — Penpot

**Status:** Pendente (após auditoria)

O principal gap suspeito no Penpot é a leitura do `manifest.json` para metadados:
```rust
// Extrair dimensões do manifest.json dentro do ZIP
let mut manifest_entry = archive.by_name("manifest.json")?;
let mut manifest_json = String::new();
manifest_entry.read_to_string(&mut manifest_json)?;
let manifest: serde_json::Value = serde_json::from_str(&manifest_json)?;
let width = manifest["width"].as_f64().unwrap_or(0.0) as u32;
let height = manifest["height"].as_f64().unwrap_or(0.0) as u32;
```

### 4. Registrar Rebelle no Provider Correto

**Status:** Verificar

O `.reb` (Rebelle) precisa estar registrado. Verificar se está no `BinaryDesignFormatProvider` ou em `project_zip_formats.rs`.

**Verificar em `project_zip_formats.rs`:**
```rust
fn supported_extensions(&self) -> Vec<&'static str> {
    vec!["clip", "sketch", "fig", "penpot", "kra"] // tem "reb"?
}
```

Se `.reb` estiver ausente, adicionar.

### 5. Testar Com Arquivos Reais

**Status:** Pendente

- [ ] Arquivo `.reb` criado no Rebelle 6
- [ ] Arquivo `.penpot` exportado do Penpot Cloud

## Arquivos a Modificar

- `src-tauri/src/processing/media/extractors/rebelle.rs` — completar se necessário
- `src-tauri/src/processing/media/extractors/penpot.rs` — completar se necessário
- `src-tauri/src/processing/media/project_zip_formats.rs` — registrar `.reb` e `.penpot`

## Critérios de Aceitação

- [ ] Arquivo `.reb` gera thumbnail JPEG/PNG correto
- [ ] Arquivo `.penpot` gera thumbnail correto
- [ ] Penpot: inspector mostra dimensões da página
- [ ] Sem erros para ZIPs corrompidos (graceful degradation)

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/rebelle.rs`
- `mundam-main/src-tauri/src/thumbnails/extractors/penpot.rs`
