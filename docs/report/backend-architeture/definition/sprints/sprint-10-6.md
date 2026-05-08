# Sprint 10.6: Migração Completa — Extractor GIMP XCF (.xcf)

**Status da sprint:** Concluído ✅
**Data e hora de inicio da sprint:** 2026-05-08T13:32:00-03:00
**Data e hora da conclusão da sprint:** 2026-05-08T17:15:00-03:00

## Objetivo

Garantir paridade completa do extractor GIMP XCF (`.xcf`) na V2, incluindo extração de thumbnail, dimensões e suporte a todas as versões do formato XCF.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/xcf.rs`
- **Tamanho:** 11,192 bytes
- Parse do formato XCF com suporte a versões `gimp xcf v001` até `gimp xcf v013`
- Extração de thumbnails do image comment `Thumb::Image::*`
- Leitura de dimensões do header XCF (offset 14-21)
- Suporte a layer compositing quando thumbnail embutido não existe

### V2 (Inicial) — `src-tauri/src/processing/media/extractors/xcf.rs`
- **Tamanho:** 7,890 bytes (−30% vs V1)
- Implementação parcial

### V2 (Pós-Sprint) — `src-tauri/src/processing/media/extractors/xcf.rs`
- **Tamanho:** 17,170 bytes (+53% vs V1)
- Implementação completa com paridade V1 + Otimização PROP_THUMBNAIL + Refatoração Clean Code (Nomes Descritivos).

## Análise de Gap

A diferença de 30% de tamanho inicial sugeria que certas versões do XCF ou caminhos de fallback não estavam cobertos.

| Funcionalidade                 | V1  | V2 (Inicial) | V2 (Final)    |
| ------------------------------ | --- | ------------ | ------------- |
| Parse header XCF + versão      | ✅   | ❓ Verificar  | ✅             |
| Extract thumbnail embutido     | ✅   | ❓ Verificar  | ✅ (Otimizado) |
| Extract dimensões do header    | ✅   | ❓ Verificar  | ✅             |
| Suporte XCF v001–v013          | ✅   | ❓ Verificar  | ✅             |
| Fallback via layer compositing | ✅   | ❓ Verificar  | ✅             |

## Tarefas

### 1. Auditar xcf.rs V2 vs V1

**Status:** Concluído ✅

**Arquivos a comparar:**
- `mundam-main/src-tauri/src/thumbnails/extractors/xcf.rs` (V1, 11KB)
- `src-tauri/src/processing/media/extractors/xcf.rs` (V2, 7.9KB)
- `src-tauri/src/processing/media/extractors/xcf.rs` (V2 Final, 17KB)

**Áreas verificadas:**
- [x] Suporte às versões mais novas do XCF (v010+)
- [x] Lógica de fallback quando thumbnail embutido está ausente
- [x] Extração de `width` e `height` do header para MetadataCapability

### 2. Implementar extract_xcf_dimensions()

**Status:** Concluído ✅

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

**O que foi feito:**

O header XCF tem dimensões em offsets fixos. Implementada função de extração robusta integrada ao MetadataCapability.

### 3. Integrar extract_xcf_dimensions no BinaryDesignFormatProvider

**Status:** Concluído ✅

Adicionado `"xcf"` ao match de dimensões no `MetadataCapability` do `BinaryDesignFormatProvider`.

### 4. Testar com Múltiplas Versões XCF

**Status:** Concluído ✅

| Versão XCF      | GIMP Version | Testado |
| --------------- | ------------ | ------- |
| `gimp xcf v001` | GIMP 2.0     | [x]     |
| `gimp xcf v006` | GIMP 2.6     | [x]     |
| `gimp xcf v010` | GIMP 2.10    | [x]     |
| `gimp xcf v013` | GIMP 3.0     | [x]     |

## Arquivos Modificados

- `src-tauri/src/processing/media/extractors/xcf.rs` — Refatoração total e novas funções.
- `src-tauri/src/processing/media/extractors/mod.rs` — Re-exportação de dimensões.
- `src-tauri/src/processing/media/binary_design_formats.rs` — Integração de metadados.

## Critérios de Aceitação

- [x] Arquivo `.xcf` gera thumbnail correto para todas as versões suportadas
- [x] Inspector mostra largura e altura corretas do canvas GIMP
- [x] Sem erros para arquivos XCF criados no GIMP 3.0 (v013)

## Referência V1

- `mundam-main/src-tauri/src/thumbnails/extractors/xcf.rs`

## Detalhes da Execução (V2)

A execução desta sprint focou em elevar o padrão de qualidade do código e adicionar otimizações de performance que não existiam na V1.

### 1. Refatoração Completa de Nomenclatura
- Seguindo a regra `[user_global]`, todas as variáveis abreviadas herdadas do porte inicial da V2 foram renomeadas para termos descritivos:
  - `cw` -> `canvas_width`
  - `ch` -> `canvas_height`
  - `bpo` -> `bytes_per_offset`
  - `txs/tys` -> `tiles_x/tiles_y`
  - `tp` -> `total_pixels`
  - `ci/ti` -> `canvas_index/tile_index`

### 2. Otimização: Extração de PROP_THUMBNAIL (Tipo 25)
- Implementada a busca ativa pela propriedade global de thumbnail do GIMP.
- Caso o arquivo possua um thumbnail embutido (comum em versões recentes), o sistema extrai o bloco RAW RGB/RGBA e converte para PNG instantaneamente, garantindo performance O(1) em arquivos complexos.

### 3. Implementação de Dimensões e Metadados
- Criada a função `extract_xcf_dimensions` que lê o header (offsets 14-21) com suporte a Big-Endian.
- Integrada ao `MetadataCapability` do `BinaryDesignFormatProvider`.

### 4. Melhoria no Tratamento de Erros
- O enum `XcfError` foi expandido para capturar falhas específicas de assinatura, versão não suportada e erros de parse de strings GIMP, garantindo robustez superior à V1.
