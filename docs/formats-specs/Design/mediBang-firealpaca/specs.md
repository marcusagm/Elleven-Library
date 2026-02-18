# Technical Architecture: MediBang Paint / FireAlpaca (.mdp)

## 1. Format Overview

*   **Extension:** `.mdp`
*   **Origin:** MediBang Paint, FireAlpaca (MediBang Inc.).
*   **Category:** Layered Raster Image Document.
*   **Magic Signature:** `6D 64 69 70 61 63 6B 00` ("mdipack\0").
*   **Typical Size:** 2 KB to 500+ MB.
*   **Container Type:** Chained Hybrid Binary Record (PAC).

## 2. Global Binary Structure

The file is partitioned into three segments based on the header. However, the logical structure (PAC blocks) ignores these partition boundaries and chains through them.

| Offset | Size | Type | Field Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 8 | `ASCII` | **Magic** | `mdipack\0` signature. |
| `0x08` | 4 | `u32 LE` | **Version** | Format version (usually 1 or 0). |
| `0x0C` | 4 | `u32 LE` | **BinSize** | Partition size for the tail binary region. |
| `0x10` | 4 | `u32 LE` | **XMLSize** | Partition size for the XML/PAC head region. |
| `0x14` | `XMLSize` | `Mixed` | **Region 1** | XML text followed by PAC blocks. |
| `20+XMLSize` | `BinSize` | `Binary` | **Region 2** | Remainder of PAC blocks. |

## 3. Main Header

The header is 20 bytes long. Any additional bytes before the XML are uncommon.

*   `0x00 - 0x07`: `Magic` ("mdipack\0")
*   `0x08 - 0x0B`: `Version` (LE)
*   `0x0C - 0x0F`: `BinSize` (LE)
*   `0x10 - 0x13`: `XMLSize` (LE)

## 4. Identified Internal Structures

### 4.1. Project XML
Project structure is defined in standard UTF-8 XML.
*   **Structure:** Root tag `<Mdiapp>`.
*   **Asset References:** Tags like `<Thumb>` and `<Layer>` contain a `bin` attribute (e.g., `bin="thumb"`, `bin="layer0img"`) which serves as a key for PAC block lookup.

### 4.2. PAC Block (Chained Records)
Binary data is stored in chained blocks starting immediately after the final XML tag (`</Mdiapp>`). Multiple PAC blocks are concatenated until the end of the file.

| Offset | Size | Type | Field Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 | `ASCII` | **Magic** | `PAC ` (Space-padded). |
| `0x04` | 4 | `u32 LE` | **TotalSize** | Length of the entire PAC block (Header + Metadata + Data). |
| `0x08` | 4 | `u32 LE` | **Unknown** | Internal flags/version. |
| `0x0C` | 4 | `u32 LE` | **ZlibSize** | Size of the compressed payload. |
| `0x10` | 116 | `Binary` | **Metadata** | Entry details including the ASCII key name. |
| `0x84` | `ZlibSize`| `Zlib` | **Payload** | Compressed block data. |

Note: The key name (e.g., `thumb`, `layer0img`) is stored as a null-terminated ASCII string within the 116-byte metadata area, typically starting around offset `0x34` (52 bytes) into the metadata area.

## 5. Endianness

*   **Little-Endian (LE):** All numeric header fields and PAC metadata are stored in LE.

## 6. Compression

*   **Zlib:** Each PAC block payload is a standard Zlib stream.
*   **Method:** Deflate (Usually level 6 or higher).

## 7. Image Data

### 7.1. Thumbnail
*   **Dimensions:** Usually `256 x 256` pixels.
*   **Format:** Raw `BGRA` (4 bytes per pixel) uncompressed.
*   **Total Raw Size:** $256 \times 256 \times 4 = 262,144$ bytes.

### 7.2. Layers
*   **Format:** Stored as raster planes.
*   **Encoding:** Variable (8bpp alpha-only for some layers, 32bpp BGRA for others).
*   **Tiling:** Large canvases may use multiple layers/tiles, although standard project files often group planes by layer ID.

## 8. Embedded Thumbnail / Preview

*   **Detection:** Look for a PAC block where the metadata area contains the string `thumb`.
*   **Extraction:**
    1.  Parse header to jump into the PAC chain.
    2.  Locate the `PAC ` block with name `thumb`.
    3.  Decompress the `ZlibSize` bytes following the 132-byte header.
    4.  Resulting buffer is `256 x 256 x 4` raw pixel data.

## 9. Metadata

*   Found in the XML portion of the file.
*   Includes DPI, layer modes (multiply, screen, etc.), opacity, and custom brushes.

## 10. Structural Reverse Engineering

*   **Linear Chaining:** The format allows O(N) access to all layers but O(1) jump to the whole "binary package" region. 
*   **Boundary Crossing:** PAC blocks are agnostic to the `XMLSize`/`BinSize` split.

## 11. Implementation Strategy

1.  **Header Parsing:** Validate magic and get sizes.
2.  **XML Splitting:** Read Region 1, separate XML text from binary trailing data.
3.  **PAC Iteration:** Walk blocks starting from the first `PAC ` magic found.
4.  **Content Resolution:** Map `bin="..."` values from XML to the names found in PAC headers.

## 12. Parser Pseudocode

```pseudo
open file
read mdipack_header (20 bytes)
xml_data = read(header.xml_size)

# Find where XML ends and PAC starts
pac_offset = xml_data.find("PAC ")
current_abs_offset = 20 + pac_offset

while current_abs_offset < file_size:
    seek(current_abs_offset)
    block_header = read(132 bytes)
    total_size = block_header.u32(4)
    data_size = block_header.u32(12)
    name = block_header.extract_name(16) # Search name in metadata
    
    if name == "thumb":
        compressed_data = read(data_size)
        return zlib_decompress(compressed_data)
        
    current_abs_offset += total_size
```

## 13. Thumbnail Generation Strategy

*   **Direct Extraction:** Always prefer the `thumb` PAC block if present.
*   **Conversion:** Swap `BGRA` channels to `RGBA` for standard web/UI display.

## 14. Basic Visualization

*   Display extracted `thumb` block with transparency enabled.

## 15. File Mapping Between Samples

| File | Res | Layers | PAC Chunks | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `8bit_test.mdp` | 8x8 | 1 | 2 (thumb, layer) | Minimized baseline. |
| `aula_silhueta.mdp` | - | Many | 14 | Production complexity. |

## 16. Uncertain Points

*   **PAC Record Padding [Confidence 75%]:** PAC total size usually aligns with a block or is simply the header + data size.
*   **Multi-part XML [Confidence 90%]:** XML is UTF-8; non-ASCII characters in layer names are encoded normally.

## 17. Technical Conclusion

The MDP format is a high-performance project file designed for rapid saving and loading. By grouping raster data into independent compressed PAC blocks, it minimizes memory pressure and allows partial loading of assets. For thumbnail extractors, it is highly efficient, requiring only a simple header walk and a single Zlib decompression.
