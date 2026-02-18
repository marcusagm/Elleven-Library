# Technical Specification: Corel Painter (.rif)

## 1. Format Overview
*   **Extension Name:** `.rif` (Raster Image File)
*   **Possible Origin:** Procreate (Fractal Design), MetaCreation, Corel Corporation (Corel Painter).
*   **Category:** Multilayer Raster Graphics Document / Digital Art.
*   **Magic Signature (Hexadecimal):** `00 02` (Version 2) or `52 49 46 46` (Legacy RIFF variant).
*   **Typical Size Observed:** 80 KB (minimalist) to 50+ MB.
*   **Variations Between Analyzed Files:** All modern analyzed files use the `00 02` signature and follow a fixed 8-byte header structure followed by binary blocks.

## 2. Global Binary Structure

| Offset | Size | Type | Field Name | Description | Observations |
| ------ | ------- | ---- | ------------- | --------- | ----------- |
| `0x00` | 8 bytes | `Struct` | **Global Header** | Identification and canvas dimensions. | Big-Endian. |
| `0x08` | Variable | `Binary` | **Raster Data** | Compressed canvas data (pixel layers). | Often the largest block. |
| `EOF-Var`| Variable | `List` | **Metadata Segment**| Segment containing thumbnail and metadata. | Located at the end of the file. |

## 3. Main Header

*   **Detailed Structure (8 bytes):**
    *   `0x00`: `u16` Version (Always `0x0002` in modern files).
    *   `0x02`: `u16` Flags (e.g., `0x2000` for complex files, `0x0000` for simple ones).
    *   `0x04`: `u16` Width (BE).
    *   `0x06`: `u16` Height (BE).
*   **Endianness:** Big-Endian.

## 4. Identified Internal Structures

### 4.1. Metadata Blocks (Tagged Pairs)
Metadata blocks follow an identification pattern via tags:
*   **Block Header:**
    *   `u32 BE TotalSize`: Total block size (Tag + Payload).
    *   `4-char Tag`: ASCII identifier (e.g., `PCOL`).
    *   `u32 BE PayloadSize`: (Optional in some blocks) Actual data size.
*   **Common Tags:**
    *   `PCOL`: Paper Color (Usually 34 bytes).
    *   `FSKT`: Friskets (Protection masks).
    *   `ANNO`: Annotations (User annotations).
    *   `NOTE`: Note Text (Can include thumbnail dimension metadata).
    *   `ICCP`: ICC Profile (Embedded color profile).
    *   `BUMB`: Bump map/Texture (Surface data).

## 5. Endianness
*   **Principal:** Big-Endian.
*   **Evidence:** Dimensions like `01 04` (260) and `01 F4` (500) match the actual width and height when interpreted in Big-Endian.

## 6. Compression
*   **Indication:** The ratio between file size and pixel count ($Width \times Height$) indicates efficient compression.
*   **Estimated Algorithms:** Likely uses a variation of RLE or proprietary bitstream compression for brush and layer data.
*   **Thumbnails:** Use standard **JPEG** compression.

## 7. Image Data (Raster)
*   **Start:** Offset `0x08`.
*   **Structure:** Proprietary binary stream. Corel Painter stores not only pixels but also physical simulation properties (wetness, pigment).
*   **Differentiation:** The `LAYR` chunk can be used to separate individual layer data.

## 8. Embedded Thumbnail / Preview
*   **Is there a preview?** Yes, in most modern versions.
*   **Location:** Generally near the end of the file, before metadata blocks.
*   **Format:** **JPEG** (standard JFIF).
*   **Automatic Detection:** Search for the `FF D8 FF E0` signature (JPEG Start of Image).
*   **Extraction:** The JPEG block ends with the `FF D9` marker.

## 9. Metadata
*   **ICC Profiles:** Frequently embedded at the end of the file, following the International Color Consortium standard.
*   **Text Strings:** Found in `NOTE` or `ANNO` blocks in ASCII or UTF-16 format.

## 10. Structural Reverse Engineering
*   **Record Chaining:** The metadata segment is a sequence of `[Size][Tag][Data]` records.
*   **Container:** Functions as a simple linear container, where main data occupies the beginning and metadata is appended to the end.

## 11. Strategy for Parser Implementation
1.  **Validate Header:** Read the first 8 bytes and validate `Version == 2`.
2.  **Identify Thumbnail:** Perform a binary scan for `FF D8 FF E0` for immediate preview extraction.
3.  **Map Blocks:** Start sequential reading from the offset found after raster data until EOF.
4.  **Error Handling:** Ignore blocks with unknown tags or sizes exceeding the file limit.

## 12. Parser Pseudocode
```pseudo
open file
header = read(8)
if header.ver != 2: raise Error("Legacy or Invalid Format")

canvas_w = header.w
canvas_h = header.h

# Thumbnail extraction
pos = find_sequence(FF D8 FF E0)
if pos != -1:
    end_pos = find_sequence(FF D9 from pos)
    thumb_data = read(pos to end_pos)
    save_as_jpeg(thumb_data)

# Metadata parsing
seek(end_of_raster_data)
while not EOF:
    block_size = read_u32_be()
    block_tag = read_string(4)
    block_data = read(block_size - 4)
    process_metadata(block_tag, block_data)
```

## 13. Strategy for Thumbnail Generation
*   **Recommended Approach:** Extraction of the embedded JPEG. It is the fastest and most accurate way, as it reflects the document's saved state without processing the paint simulation.
*   **Fallback:** If no JPEG is present, decoding the main raster is discouraged due to the complexity of the proprietary rendering engine.

## 14. Strategy for Basic Visualization
*   Display the extracted JPEG thumbnail.
*   For real canvas visualization, an engine capable of interpreting proprietary data packets after the header would be required (high complexity O(N)).

## 15. Comparative Map Between Files
| File | Version | Resolution | Thumbnail | Extra Blocks |
| ------- | ------ | --------- | --------- | ------------- |
| `splat.rif` | 2 | 260x500 | N/A | NOTE, ANNO |
| `Line Sketches1.rif` | 2 | 826x1169| JPEG | FSPG, PCOL |
| `env.rif` | 2 | 826x1169| JPEG | ICCP, BUMB |

## 16. Uncertain Points
*   **Raster Compression Algorithm (Confidence: 10%):** The format of data between `0x08` and the Thumbnail is highly opaque and proprietary.
*   **Header Flags (Confidence: 40%):** The `0x2000` bit seems to indicate the presence of complex layers or active physical simulations.
*   **Tag BUMB (Confidence: 90%):** Related to "Bump" simulation (paint relief) characteristic of the software.

## 17. Technical Conclusion
The `.rif` (Painter 2.x) format is a binary container optimized for saving artistic simulation state. For external cataloging systems (like Mundam), thumbnail extraction is trivial via JPEG block scan, but total document reconstruction without the original software is a high-complexity reverse engineering challenge.
