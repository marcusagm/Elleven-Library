# Sprint 10.6: Migração Completa — Extractor GIMP XCF (.xcf)

**Status da sprint:** Verificação necessária
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Garantir paridade completa do extractor GIMP XCF (`.xcf`) na V2, incluindo extração de thumbnail, dimensões e suporte a todas as versões do formato XCF.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/xcf.rs`
- **Tamanho:** 11,192 bytes
- Parse do formato XCF com suporte a versões `gimp xcf v001` até `gimp xcf v013`
- Extração de thumbnails do image comment `Thumb::Image::*`
- Leitura de dimensões do header XCF (offset 14-21)
- Suporte a layer compositing quando thumbnail embutido não existe

### V2 — `mundam-main/src-tauri/src/processing/media/extractors/xcf.rs`
- **Tamanho:** 7,890 bytes (−30% vs V1)
- Implementação parcial

## Análise de Gap

A diferença de 30% de tamanho sugere que certas versões do XCF ou caminhos de fallback não estão cobertos.

| Funcionalidade | V1 | V2 |
|---|---|---|
| Parse header XCF + versão | ✅ | ❓ Verificar |
| Extract thumbnail embutido | ✅ | ❓ Verificar |
| Extract dimensões do header | ✅ | ❓ Verificar |
| Suporte XCF v001–v013 | ✅ | ❓ Verificar |
| Fallback via layer compositing | ✅ | ❓ Verificar |

## Tarefas

### 1. Auditar xcf.rs V2 vs V1

**Status:** Pendente

**Arquivos a comparar:**
- `mundam-main/src-tauri/src/thumbnails/extractors/xcf.rs` (V1, 11KB)
- `src-tauri/src/processing/media/extractors/xcf.rs` (V2, 7.9KB)

**Áreas a verificar:**
- Suporte às versões mais novas do XCF (v010+)
- Lógica de fallback quando thumbnail embutido está ausente
- Extração de `width` e `height` do header para MetadataCapability

### 2. Implementar extract_xcf_dimensions()

**Status:** Pendente

O header XCF tem dimensões em offsets fixos. Implementar função de extração:

```rust
// src-tauri/src/processing/media/extractors/xcf.rs

pub fn extract_xcf_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut header = [0u8; 22];
    file.read_exact(&mut header)?;

    // Verificar magic bytes: "gimp xcf "
    if !header.starts_with(b"gimp xcf ") {
        return Err("Not a valid XCF file".into());
    }

    // Width em bytes 14-17, height em bytes 18-21 (big-endian)
    let width = u32::from_be_bytes([header[14], header[15], header[16], header[17]]);
    let height = u32::from_be_bytes([header[18], header[19], header[20], header[21]]);

    Ok((width, height))
}
```

### 3. Integrar extract_xcf_dimensions no BinaryDesignFormatProvider

**Status:** Dependente da sprint 10.3 (MetadataCapability)

Adicionar `"xcf"` ao match de dimensões no `MetadataCapability` do `BinaryDesignFormatProvider`.

### 4. Testar com Múltiplas Versões XCF

**Status:** Pendente

| Versão XCF | GIMP Version | Testado |
|---|---|---|
| `gimp xcf v001` | GIMP 2.0 | [ ] |
| `gimp xcf v006` | GIMP 2.6 | [ ] |
| `gimp xcf v010` | GIMP 2.10 | [ ] |
| `gimp xcf v013` | GIMP 3.0 | [ ] |

## Arquivos a Modificar

- `src-tauri/src/processing/media/extractors/xcf.rs` — portar se necessário + adicionar `extract_xcf_dimensions()`
- `src-tauri/src/processing/media/binary_design_formats.rs` — integrar dimensions

## Critérios de Aceitação

- [ ] Arquivo `.xcf` gera thumbnail correto para todas as versões suportadas
- [ ] Inspector mostra largura e altura corretas do canvas GIMP
- [ ] Sem erros para arquivos XCF criados no GIMP 3.0 (v013)

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/xcf.rs`
