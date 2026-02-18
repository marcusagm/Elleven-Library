# CorelDRAW (.cdr) File Format Technical Specification

## 1. Format Overview
*   **Extension Name:** `.cdr`
*   **Possible Origin:** Corel Corporation.
*   **Category:** Vector Graphics Document.
*   **Magic Signature (Hexadecimal):**
    *   **Modern (X4+):** `50 4B 03 04` (ZIP container).
    *   **Legacy (X3 and below):** `52 49 46 46` (RIFF container).
    *   **Ultra Legacy (v5-):** `57 4C 6D 00` (`WLm.` proprietary header).
*   **Typical Size Observed:** 3 KB (minimalist) to 3 MB in samples.
*   **Variations Between Analyzed Files:** A clear transition from RIFF containers to ZIP containers was observed. Very old files use a direct binary format without a standard container.

## 2. Global Binary Structure

### 2.1. Modern Structure (ZIP-based)
| Offset | Size | Type | Field Name | Description |
| ------ | ------- | ---- | ------------- | --------- |
| `0x00` | 4 bytes | `u32` | **ZIP Magic** | `50 4B 03 04`. |
| Var    | Var     | `File`| **mimetype**  | `application/x-vnd.corel.zcreate`. |
| Var    | Var     | `Dir` | **previews/** | Folder containing PNG/BMP images. |
| Var    | Var     | `Dir` | **content/**  | Vector data in XML or proprietary binary format. |

### 2.2. Legacy Structure (RIFF-based)
| Offset | Size | Type | Field Name | Description |
| ------ | ------- | ---- | ------------- | --------- |
| `0x00` | 4 bytes | `ASCII`| **RIFF Magic**| `RIFF`. |
| `0x04` | 4 bytes | `u32`  | **FileSize**  | Total file size - 8. |
| `0x08` | 4 bytes | `ASCII`| **CDR Signature**| `CDR ` or `CDRB`. |
| `0x0C` | Var     | `Chunk`| **Chunks**    | Sequence of sub-RIFF blocks. |

## 3. Main Header

### 3.1. Modern (X4+)
*   **Structure:** Follows the PKZIP standard.
*   **Endianness:** Little-endian.
*   **Fields:** Local File Headers, Central Directory, End of Central Directory.

### 3.2. Legacy (RIFF)
*   **Structure:**
    *   `0x08`: Identifier `CDR ` (CorelDRAW) or `CDRB` (Compressed versions).
    *   **Version:** Frequently found in the `vrsn` sub-chunk.
*   **Endianness:** Little-endian.

## 4. Identified Internal Structures

### 4.1. RIFF Chunks (Legacy)
*   **vrsn:** Contains 2 bytes indicating the software version (e.g., `02 00` for v2, `0D 00` for X3).
*   **DISP:** (Display) Block containing the preview for Windows Explorer (usually WMF or Bitmap).
*   **icp0:** Chunk that stores icon/thumbnail for some versions.

### 4.2. ZIP Paths (Modern)
*   `previews/thumbnail.png`: Default document thumbnail (PNG).
*   `content/data/page1.dat`: Binary data for the first page.
*   `color/color.xml`: Color profile definitions.

## 5. Endianness
*   **Little-endian:** Verified in RIFF chunk sizes and ZIP headers.
*   **Evidence:** `ABCNEWS.CDR` (Legacy) file shows offsets and sizes in increasing order of significance.

## 6. Compression
*   **Modern:** Standard ZIP **Deflate** compression.
*   **Legacy (CDRB):** Uses customized LZW or RLE compression algorithms within vector data chunks to reduce RIFF size.

## 7. Image Data
*   **Vector:** The core of the format describes Bézier curves, gradient fills, and styles.
*   **Embedded Bitmaps:** Stored as binary chunks or separate files in the ZIP (e.g., `content/data/Bitmaps.dat`).

## 8. Embedded Thumbnail / Preview
*   **Modern:** `previews/thumbnail.png` file inside the ZIP.
*   **Legacy:** `DISP` or `icp0` chunk in the RIFF container.
*   **Preview Format:**
    *   Modern: **PNG**.
    *   Legacy: **BMP** or **WMF** (Windows Metafile).
*   **Detection:**
    *   Extract file from ZIP.
    *   Search for chunk ID `DISP` in the RIFF binary stream.

## 9. Metadata
*   **Modern:** Located in `META-INF/metadata.xml`.
*   **Legacy:** Located in the `LIST` chunk of type `INFO`. Contains fields like `INAM` (Name), `ICOP` (Copyright).

## 10. Structural Reverse Engineering
*   **Container Switch:** CorelDRAW abandoned the opaque binary RIFF in favor of ZIP (XML/Binary) in version X4 to improve extensibility and compatibility.
*   **Pointer System:** The ZIP container uses the Central Directory at the end of the file to locate members. The RIFF container uses sequential offsets based on block sizes.

## 11. Strategy for Parser Implementation
1.  **Differentiation:** Check bytes `0x00-0x03`.
2.  **If ZIP:** Use standard decompression library and search the `previews/` directory.
3.  **If RIFF:** Implement a "Chunk Walker" that reads ID and Size, skipping unrecognized blocks.
4.  **If WLm.:** Treat as legacy binary format (high complexity, fallback to specific libraries like `libcdr` is recommended).

## 12. Parser Pseudocode
```pseudo
open file
header = read(4)

if header == "PK\x03\x04":
    # Modern ZIP
    zip = open_as_zip(file)
    if "previews/thumbnail.png" exists:
        return extract_file("previews/thumbnail.png")
    else if "previews/page1.png" exists:
        return extract_file("previews/page1.png")

else if header == "RIFF":
    # Legacy RIFF
    skip(4) # skip size
    type = read(4)
    while not EOF:
        chunk_id = read(4)
        chunk_size = read_u32_le()
        if chunk_id == "DISP":
            skip(4) # skip type/flags
            return read_image_data(chunk_size - 4)
        skip(chunk_size + alignment)
```

## 13. Strategy for Thumbnail Generation
*   **Modern:** Direct extraction from ZIP (O(1) complexity).
*   **Legacy:** Parsing of RIFF chunks. If `DISP` contains WMF, it may require extra rendering. Prioritize BMP extraction if available in the `icp0` chunk.

## 14. Strategy for Basic Visualization
*   Display the extracted PNG thumbnail.
*   Full vector rendering: Requires parsing multiple `.dat` or `.xml` files and implementing a vector rendering engine compatible with Corel specifications (extremely complex).

## 15. Comparative Map Between Files
| File | Structure | Estimated Version | Thumbnail | Observations |
| ------- | --------- | --------------- | --------- | ----------- |
| `example.cdr`| ZIP | X4+ | PNG (previews/) | Complete modern structure. |
| `ABCNEWS.CDR`| WLm. | v5 or lower | N/A | Raw proprietary binary format. |
| `01-Receipt...`| RIFF | X3 or lower | DISP Chunk | Legacy financial document. |

## 16. Uncertain Points
*   **WLm. Format:** Practically undocumented. Based on direct memory dumps of software structures from the 80s/90s (Confidence: 20%).
*   **Custom RIFF Chunks:** Users can extend the format with private non-standard chunks by Corel (Confidence: 90%).

## 17. Technical Conclusion
The `.cdr` is a format that evolved from a proprietary binary structure to a standard container (ZIP). Parsing ease for thumbnail purposes is high in modern versions but drops drastically for legacy files, where support tends to depend on reverse engineering libraries like `libcdr` from the LibreOffice project.
