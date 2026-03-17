# Sprint 10.7: Migração Completa — Extractor MDP (MediBang Paint / FireAlpaca)

**Status da sprint:** Verificação necessária
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Verificar paridade do extractor MDP (`.mdp`) e garantir que dimensões e preview estão sendo extraídos corretamente na V2.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/mdp.rs`
- **Tamanho:** 30,193 bytes — o maior extractor da V1
- Parse binário completo do formato MDI Pack
- Magic bytes: `mdipack`
- Parse XML embutido para localizar o bloco de thumbnail
- Blocos PAC identificados por nome, suportando compressão Zlib
- Extração completa de thumbnails RGBA

### V2 — `src-tauri/src/processing/media/extractors/mdp.rs`
- **Tamanho:** 3,074 bytes (−90% vs V1)
- ✅ Magic bytes `mdipack` verificado
- ✅ Parse XML via `quick_xml` para localizar bloco Thumb
- ✅ Parse de PAC blocks com suporte a Zlib
- ✅ BGRA → RGBA swap + PNG encoding
- ✅ Implementação completa mas muito mais concisa via boas bibliotecas

## Análise Refinada

**Apesar da diferença de tamanho, o extractor V2 parece funcional.** O código V1 provavelmente tinha mais verbosidade, tratamento de erros com mensagens detalhadas e talvez suporte a mais variações do formato. A diferença de 90% de tamanho pode ser parcialmente explicada por:
- V1 usando código mais verboso com structs separadas
- V2 usando `byteorder`, `flate2`, `quick_xml` de forma mais concisa

**O que falta verificar:**
1. Extração de dimensões (width, height) do arquivo MDP
2. Se todos os casos de PAC são tratados (comprimido vs não comprimido)
3. Arquivos MDP de versões mais antigas do MediBang

## Tarefas

### 1. Verificar Extração de Dimensões MDP

**Status:** Pendente

O XML embutido no arquivo MDP contém o atributo `Thumb` com `width` e `height`. O V2 já lê `tw` e `th` ao parsear o XML, mas esses valores são usados apenas para o PNG encoding interno.

**Implementar `extract_mdp_dimensions()`:**

```rust
// src-tauri/src/processing/media/extractors/mdp.rs

pub fn extract_mdp_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::{BufReader, Read, Seek, SeekFrom};

    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 7];
    reader.read_exact(&mut magic)?;
    if &magic != b"mdipack" { return Err("Invalid MDP magic".into()); }
    reader.seek(SeekFrom::Current(5))?;
    let xml_len = reader.read_u32::<LittleEndian>()?;
    let _ = reader.read_u32::<LittleEndian>()?;
    let mut xml_buf = vec![0u8; xml_len as usize];
    reader.read_exact(&mut xml_buf)?;
    let xml = String::from_utf8_lossy(&xml_buf);

    // Parse do XML já presente no V2
    let mut canvas_width: u32 = 0;
    let mut canvas_height: u32 = 0;
    let mut xml_reader = quick_xml::reader::Reader::from_str(&xml);
    let mut buf = Vec::new();
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(e)) | Ok(quick_xml::events::Event::Empty(e))
                if e.name().as_ref() == b"Canvas" =>
            {
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"width" => { canvas_width = attr.unescape_value()?.parse().unwrap_or(0); }
                        b"height" => { canvas_height = attr.unescape_value()?.parse().unwrap_or(0); }
                        _ => {}
                    }
                }
                break;
            }
            Ok(quick_xml::events::Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    if canvas_width > 0 && canvas_height > 0 {
        Ok((canvas_width, canvas_height))
    } else {
        Err("Canvas dimensions not found in MDP XML".into())
    }
}
```

### 2. Testar com Arquivos MDP Reais

**Status:** Pendente

O formato MDP é usado por MediBang Paint e FireAlpaca. Testar com arquivos de ambos os programas em versões diferentes.

### 3. Integrar no BinaryDesignFormatProvider

**Status:** Verificar

O `.mdp` precisa ser adicionado à lista `supported_extensions` do `BinaryDesignFormatProvider`:

```rust
fn supported_extensions(&self) -> Vec<&'static str> {
    vec!["sai", "sai2", "xcf", "rif", "riff", "clip", "mdp"] // adicionar "mdp"
}
```

E no `ThumbnailCapability.generate()`:
```rust
"mdp" => extract_mdp_preview(path),
```

## Arquivos a Modificar

- `src-tauri/src/processing/media/extractors/mdp.rs` — adicionar `extract_mdp_dimensions()`
- `src-tauri/src/processing/media/binary_design_formats.rs` — registrar `.mdp` e integrar dimensions

## Critérios de Aceitação

- [ ] Arquivo `.mdp` (MediBang Paint) gera thumbnail correto
- [ ] Arquivo `.mdp` (FireAlpaca) gera thumbnail correto
- [ ] Inspector mostra dimensões (canvas width × height)
- [ ] Sem panic para arquivos MDP corrompidos ou com PAC ausente

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/mdp.rs`
