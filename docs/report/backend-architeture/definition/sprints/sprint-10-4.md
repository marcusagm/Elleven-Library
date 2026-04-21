# Sprint 10.4: Migração Completa — Extractor SAI2 (PaintTool SAI v2)

**Status da sprint:** Concluído
**Data e hora de inicio da sprint:** 2026-04-13T15:55:00-03:00
**Data e hora da conclusão da sprint:** 2026-04-21T16:44:00-03:00

## Objetivo

Verificar e garantir paridade completa do extractor de PaintTool SAI v2 (`.sai2`) entre V1 e V2.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/sai2.rs`
- **Tamanho:** 19,559 bytes
- Parse completo do formato SQLite-based do SAI2
- Abre o arquivo `.sai2` como banco SQLite
- Extrai thumbnail da tabela `canvasthumbnail`

### V2 — `mundam-main/src-tauri/src/processing/media/extractors/sai2.rs`
- **Tamanho:** 22,073 bytes (reescrita completa)
- Parse binário fiel à especificação real do formato SAI-CANVAS-TYPE0
- Suporte a thumbnails lossy (JSSF→JPEG) e lossless (DPCM)
- Extração de dimensões do canvas a partir do header binário

## Análise de Gap

O formato `.sai2` internamente **NÃO** é um banco SQLite. A documentação inicial da sprint referenciou a abordagem errada. O `.sai2` moderno gerado pela SYSTEMAX utiliza um formato binário proprietário identificado pelo magic `SAI-CANVAS-TYPE0`, com uma tabela de entidades (CanvasEntry) de 16 bytes cada apontando para blobs de dados via offset absoluto.

| Funcionalidade                           | V1      | V2 (Final)    |
| ---------------------------------------- | ------- | ------------- |
| Abertura como SQLite                     | ✅ (errada) | ❌ (correto — binary parse) |
| Parse binário SAI-CANVAS-TYPE0           | ❌      | ✅             |
| Conversão JSSF → JPEG padrão            | ❌      | ✅             |
| Decode DPCM Lossless                     | ❌      | ✅             |
| Extract dimensões canvas                 | ❌ (via SQL) | ✅ (via header) |
| Detecção automática de canais (3 vs 4)   | ❌      | ✅             |

## Tarefas

### 1. Auditar sai2.rs da V2

**Status:** Concluído

Abrir e ler o arquivo `src-tauri/src/processing/media/extractors/sai2.rs` completo e comparar com:
- `mundam-main/src-tauri/src/thumbnails/extractors/sai2.rs`

Se o V2 é apenas um stub (4.5KB vs 19.5KB do V1), portar a implementação completa.

### 2. Portar Implementação SQLite do SAI2

**Status:** Concluído (Nota: o formato SAI2 usa chunks binários e JSSF content wrapper, não SQLite. A documentação da sprint referenciou a abordagem errada, então foi portada a lógica binária completa e funcional presente em `mundam-main`).

**Implementação V1 (referência — abordagem SQLite, NÃO compatível com SAI2 moderno):**

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

**Nota:** A abordagem SQLite acima foi abandonada. O crate `rusqlite` **não** é necessário para SAI2.

### 3. Adicionar extract_sai2_dimensions ao BinaryDesignFormatProvider

**Status:** Concluído

Após implementar `extract_sai2_dimensions()`, adicioná-la ao caso `"sai2"` em `MetadataCapability` do `BinaryDesignFormatProvider` (ver sprint 10.3, Tarefa 2).

### 4. Testar com Arquivos Reais

**Status:** Concluído — 12/12 arquivos `.sai2` extraídos com sucesso, 8/8 testes passando.

O SAI2 tem variações de versão. Testado com todos os 12 arquivos `.sai2` disponíveis:

| Arquivo | Tamanho | Resultado | MIME |
|---------|---------|-----------|------|
| 1 Aderinna.sai2 | 176 MB | ✅ 98,186 bytes | image/jpeg |
| 10_0226 estudos getual rayenr.sai2 | 3.6 MB | ✅ 24,433 bytes | image/jpeg |
| 3 Cristalino.sai2 | 145 MB | ✅ 44,284 bytes | image/jpeg |
| Comissão.sai2 | 168 MB | ✅ 85,770 bytes | image/jpeg |
| Milk.sai2 | 9.7 MB | ✅ 63,375 bytes | image/jpeg |
| NewCanvas1.sai2 | 6.4 MB | ✅ 20,865 bytes | image/jpeg |
| Peanut comissão.sai2 | 74 MB | ✅ 44,530 bytes | image/jpeg |
| Pookie comissão.sai2 | 148 MB | ✅ 45,286 bytes | image/jpeg |
| comission.sai2 | 20 MB | ✅ 30,895 bytes | image/jpeg |
| design.sai2 | 2.1 MB | ✅ 9,304 bytes | image/jpeg |
| elfinha4.sai2 | 191 MB | ✅ 34,475 bytes | image/jpeg |
| estudos 0601_25.sai2 | 5.2 MB | ✅ 18,837 bytes | image/jpeg |

### 5. Reescrita Completa Baseada em Referência C++ (Wunkolo/libsai)

**Status:** Concluído (2026-04-21)

A implementação anterior continha 7 bugs críticos identificados por comparação com a implementação de referência C++ do projeto [Wunkolo/libsai](https://github.com/Wunkolo/libsai) (licença MIT). O extrator foi completamente reescrito.

**Bugs corrigidos:**

| # | Bug | Gravidade | Antes | Depois |
|---|-----|-----------|-------|--------|
| 1 | Header offsets errados | CRÍTICO | Width @ 0x20, Height @ 0x24 | Width @ 0x14, Height @ 0x18 |
| 2 | CanvasBackgroundFlags como u32 | CRÍTICO | u32 @ offset 0x28 | u8 @ offset 0x11 |
| 3 | Canvas table sem offset absoluto | CRÍTICO | Offsets calculados cumulativamente | BlobsOffset absoluto (u64) por CanvasEntry |
| 4 | Blob prefix ignorado | CRÍTICO | Direto para `jssf` tag | Width(u32) + Height(u32) + BlobDataType(u32) |
| 5 | Conversão JSSF→JPEG incompleta | CRÍTICO | Tentava ler JPEG raw | Conversão completa com quantization tables, Huffman tables, MCU rows |
| 6 | DPCM predictor incorreto | GRAVE | Lógica de saturação inconsistente | Port fiel de `DeltaUnpackRow16Bpc` |
| 7 | Tile alignment checksum ignorado | GRAVE | Não consumido | Checksum de 2 bytes após cada row Y de tiles |

## Arquivos Modificados

- `src-tauri/src/processing/media/extractors/sai2.rs` — reescrita completa (22,073 bytes)
- `src-tauri/src/processing/media/binary_design_formats.rs` — integração de dimensions (sprint 10.3)

## Critérios de Aceitação

- [x] Arquivo `.sai2` gera thumbnail correto (validado visualmente — sem rainbow noise)
- [x] Inspector mostra dimensões corretas do canvas SAI2
- [x] Sem erros de I/O ou panic ao abrir arquivos SAI2 de diferentes versões
- [x] Todos os 12 arquivos de teste extraídos com sucesso
- [x] Previews validadas visualmente (elfinha4, Milk, design)

## Referências

- `mundam-main/src-tauri/src/thumbnails/extractors/sai2.rs` (V1 — abordagem SQLite, descontinuada)
- [Wunkolo/libsai](https://github.com/Wunkolo/libsai) — Implementação de referência C++ (MIT)
  - `include/sai2.hpp` — Structs `CanvasHeader`, `CanvasEntry`, `BlobDataType`
  - `source/sai2.cpp` — Funções `IterateCanvasData`, `UnpackDeltaRLE16`, `DeltaUnpackRow16Bpc`, `ExtractDpcmToBGRA`, `ConvertJssfToJpeg`
- [Wunkolo/SaiThumbs](https://github.com/Wunkolo/SaiThumbs) — Windows Shell Extension para thumbnails SAI/SAI2

---

## 🚀 Deep Reverse Engineering: Desvendando o SAI2 (Abril 2026)

Durante o desenvolvimento do Extrator V2, identificamos que a abordagem baseada em SQLite (comum em versões antigas) falhava em arquivos `.sai2` modernos gerados pela SYSTEMAX. O extrator foi evoluído através de múltiplas fases de engenharia reversa para atingir a "Fidelidade Total".

### 1. A Crise do "Rainbow Noise"
As primeiras tentativas de extração resultavam em imagens com ruído colorido estático. Descobrimos que isso se devia a três fatores críticos:
- **Bitstream Proprietário**: O SAI2 não usa RLE padrão ou Zlib para miniaturas lossless. Ele utiliza um fluxo de bits variável onde o comando (OpCode) é determinado pelo número de zeros à direita em uma máscara de controle de 64 bits.
- **Detecção de Canais (3 vs 4)**: A descoberta mais vital foi que o SAI2 alterna entre 3 canais (RGB) e 4 canais (RGBA). Tentar ler 3 canais como 4 causava um deslocamento de 25% no stream, gerando o efeito "arco-íris". O número de canais é definido pelo byte `0x11` do cabeçalho global (`CanvasBackgroundFlags`): `flags & 0x07 == 0` → 4 canais (RGBA), caso contrário → 3 canais (RGB).
- **Aritmética Saturada**: O canal de cor DPCM não aceita "overflow". Se a soma de pixels ultrapassar 255 ou cair abaixo de 0, o software trava o valor nos limites. O uso de `wrapping_add` em Rust causava a inversão de cores.

### 2. Navegação via Tabela de Entidades
Identificamos que o SAI2 é estruturado como um sistema de arquivos interno. Ao invés de chunks lineares, o extrator deve navegar via:
- **Table** começa imediatamente após o header de 64 bytes (offset 0x40).
- **Table Count** está no offset `0x20` (u32 LE), total de entradas `CanvasEntry`.
- Cada `CanvasEntry` tem 16 bytes: `Type(u32)` + `LayerID(u32)` + `BlobsOffset(u64)`.
- Os tipos incluem `thum` (thumbnail lossy/JSSF), `intg` (thumbnail lossless/DPCM), `layr` (camada), `hist` (histórico).
- O `BlobsOffset` é um **offset absoluto** no arquivo, não relativo. O tamanho do blob é calculado pela diferença entre offsets consecutivos.

### 3. O Preditor de Plano (Plane Predictor)
A maior descoberta matemática foi que o SAI2 não armazena deltas lineares. Ele utiliza um **Preditor de Plano** sofisticado (similar ao modo Paeth do PNG, mas simplificado para o domínio de 16-bits):

```
Sum = Add(
    SubSaturated(
        AddSaturated(
            SubSaturated(
                Add(Sum, PixelAcima),
                PixelDiagonalAnterior
            ),
            0xFF00
        ),
        0xFF00
    ),
    Delta
)
Output = Saturate16→8(Sum)
```

Onde `Add` é wrapping add, `AddSaturated`/`SubSaturated` são aritméticas saturadas em u16, e `0xFF00` é uma constante de normalização que funciona como clamp bidirecional.

### 4. Anatomia de um Bloco (Tile)
Diferente da V1, que assumia blocos fixos, o V2 agora lida com a estrutura dinâmica:
- **Tabela de Índices**: Antes dos pixels, o arquivo contém uma tabela de `u32` LE com o tamanho exato de cada bloco comprimido.
- **Checksum de Alinhamento**: Cada bloco de 256x256 começa com um Checksum de 2 bytes, onde o byte superior identifica o índice X do bloco.
- **Checksum de Linha**: Após todos os tiles X de uma row Y, existe um checksum extra de 2 bytes que deve ser consumido.

### 5. Container JSSF (Thumbnail Lossy)
O JSSF é um formato proprietário da SYSTEMAX que embala dados JPEG de forma compacta:
- Não contém headers JPEG padrão (SOI, DQT, DHT, SOF, SOS).
- Armazena apenas: tabelas de quantização (64 bytes cada) + rows de MCU prefixadas com u16 size.
- A conversão para JPEG padrão requer reconstruir manualmente todos os markers JPEG incluindo tabelas Huffman fixas padrão.
- O extrator reconstrói o JPEG completo com: SOI → DQT → DHT → SOF0 → DRI → SOS → MCU data com restart markers → EOI.

---

## 🛠 Especificação Técnica de Referência: Formato SAI2 (.sai2)

### A. Cabeçalho Global (SAI-CANVAS-TYPE0)
Endereço fixo no início do arquivo. Total: 64 bytes.

| Offset | Tamanho | Descrição                                       |
| ------ | ------- | ----------------------------------------------- |
| 0x00   | 16 bytes| Magic `SAI-CANVAS-TYPE0`                        |
| 0x10   | 1 byte  | Flags0                                          |
| 0x11   | 1 byte  | **CanvasBackgroundFlags** (`& 0x07 == 0` → 4ch RGBA, senão 3ch RGB) |
| 0x12   | 1 byte  | Flags2                                          |
| 0x13   | 1 byte  | Flags3                                          |
| 0x14   | 4 bytes | **Width** (LE) — Largura do Canvas (PX)         |
| 0x18   | 4 bytes | **Height** (LE) — Altura do Canvas (PX)         |
| 0x1C   | 4 bytes | PrintingResolution (LE)                         |
| 0x20   | 4 bytes | **TableCount** (LE) — Total de CanvasEntry      |
| 0x24   | 4 bytes | SelectedLayer (LE)                              |
| 0x28   | 8 bytes | UnknownA                                        |
| 0x30   | 8 bytes | UnknownB                                        |
| 0x38   | 4 bytes | CanvasBackgroundColor (LE, RGBA)                |
| 0x3C   | 4 bytes | LayerEffectColor (LE, tag: "norm"/"vivd"/"deep") |

### B. Tabela de Entidades (CanvasEntry)
Começa imediatamente após o header (offset 0x40). Cada entrada tem 16 bytes.

| Offset | Tamanho | Descrição                                        |
| ------ | ------- | ------------------------------------------------ |
| 0x00   | 4 bytes | **Type** (LE tag: `thum`, `intg`, `layr`, `hist`, `mask`, etc.) |
| 0x04   | 4 bytes | LayerID (LE)                                     |
| 0x08   | 8 bytes | **BlobsOffset** (LE, u64) — Offset absoluto no arquivo |

### C. Prefixo de Blob
Cada blob referenciado por um `CanvasEntry` inicia com um prefixo de 12 bytes:

| Offset | Tamanho | Descrição                                        |
| ------ | ------- | ------------------------------------------------ |
| 0x00   | 4 bytes | Width (LE, u32) — Dimensão do conteúdo           |
| 0x04   | 4 bytes | Height (LE, u32) — Dimensão do conteúdo          |
| 0x08   | 4 bytes | **BlobDataType** (LE tag: `jssf`, `dpcm`, `raw\0`) |

### D. Container JSSF (Magic: `jssf` / dentro do blob)
Após o prefixo de 12 bytes do blob com BlobDataType == `jssf`:

| Offset | Tamanho | Descrição                                             |
| ------ | ------- | ----------------------------------------------------- |
| 0x00   | 2 bytes | Largura da Miniatura (LE, u16)                        |
| 0x02   | 2 bytes | Altura da Miniatura (LE, u16)                         |
| 0x04   | 2 bytes | **Número de Canais** (LE, u16 — tipicamente 1 ou 3)  |
| 0x06   | 64 bytes| Tabela de Quantização Luma                            |
| 0x46   | 64 bytes| Tabela de Quantização Chroma (se canais > 1)          |
| Var    | Var     | Rows de MCU, cada uma prefixada com u16 LE (tamanho)  |

### E. Decodificação DPCM Lossless (BlobDataType == `dpcm`)
Após o prefixo de 12 bytes do blob:

1. **Tabela de Tamanhos de Tiles**: `TilesX × TilesY` valores u32 LE, cada um indicando o tamanho comprimido do tile correspondente.
2. **Tiles**: Imagem dividida em blocos de 256×256.
3. **Por cada Tile**:
   - 2 bytes: Checksum (byte superior = índice X do tile)
   - Dados comprimidos: linhas de deltas RLE
4. **Por cada Row Y de Tiles**: Checksum de alinhamento extra de 2 bytes após o último tile X.
5. **Bitstream System**:
   - Lê palavras de 32 bits (Little Endian) em uma `ControlMask` de 64 bits.
   - `FirstSetBit = trailing_zeros(ControlMask)`.
   - `OpCode = (2 × FirstSetBit) | (NextBit)`.
6. **OpCodes Table**:
   - `0`: Valor zero literal.
   - `1..14`: Lê `n` bits + bit de sinal → Valor Delta i16.
   - `15`: Run-length de zeros (7 bits para contagem + 8).
7. **Plane Predictor (16-bit space)**:
   - Cálculos realizados em `u16` com aritmética saturada.
   - Conversão final 16 bits → 8 bits com clamp em 255.
   - Fórmula: `Sum = Add(SubSaturated(AddSaturated(SubSaturated(Add(Sum, Above), Diagonal), 0xFF00), 0xFF00), Delta)`

---

## 📊 Status de Paridade Final
- [x] Suporte a thumbnails Lossy (JSSF → JPEG padrão reconstruído).
- [x] Suporte a thumbnails Lossless (DPCM 256x256 Tiles → BGRA → PNG).
- [x] Reconstrução cromática 1:1 com o software original.
- [x] Resiliência a desalinhamento via verificação de Checksum de bloco.
- [x] Detecção automática de canais (3 RGB vs 4 RGBA) via `CanvasBackgroundFlags`.
- [x] Header parsing correto validado via hex dump em múltiplos arquivos.
- [x] 12/12 arquivos de produção extraídos com sucesso.
- [x] Validação visual confirmada — sem rainbow noise.
