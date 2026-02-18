# Technical Analysis: PaintTool SAI v2 (.sai2)

## 1. Format Overview
*   **Extension Name:** `.sai2`
*   **Possible Origin:** SYSTEMAX PaintTool SAI (Version 2).
*   **Category:** Multilayer Raster Graphics Document.
*   **Magic Signature (Hexadecimal):** `53 41 49 2D 43 41 4E 56 41 53` (`SAI-CANVAS`).
*   **Typical Size Observed:** 1.6 MB to 200 MB (with layers and high resolution).
*   **Variations Between Analyzed Files:** The header can vary between versions (e.g., `SAI-CANVAS-TYPE0`), and the chunk count field may be zero in recent versions, requiring tag scanning.

## 2. Global Binary Structure

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 64 bytes | `Header` | **File Header** | Identification and global metadata. | Fixed size. |
| `0x40` | N * 16 bytes | `List` | **Chunk List** | Block descriptor table. | N can be variable. |
| Variable | Variable | `Block` | **Chunk Data** | Raw data of sequenced chunks. | Variable alignment. |

## 3. Main Header
*   **Detailed Structure:**
    *   `0x00`: Magic (10 bytes) - `SAI-CANVAS`.
    *   `0x0A`: Type Suffix (6 bytes) - e.g., `-TYPE0`.
    *   `0x10`: Unknown (4 bytes) - Often `0x00004000`.
    *   `0x14`: Chunk Count Alt (4 bytes) - May contain the actual number of chunks if the official field is zero.
    *   `0x20`: Canvas Width (4 bytes) - Width in pixels (u32 LE).
    *   `0x24`: Canvas Height (4 bytes) - Height in pixels (u32 LE).
    *   `0x28`: Chunk Count (4 bytes) - Official field for the number of chunks (u32 LE).
*   **Endianness:** Little-Endian.

## 4. Identified Internal Structures

### 4.1. Chunk Descriptor (16 bytes)
*   **Tag (4 bytes):** ASCII identifier (e.g., `thum`, `view`, `layr`).
*   **ID/Flags (4 bytes):** Unique block identifier or state flags.
*   **Size (8 bytes):** Chunk data size in bytes (u64 LE).

### 4.2. Chunk Data (Canvas Entries)
Some chunks (`thum`, `layr`) use an internal entry structure:
*   **Type (4 bytes):** Entry type (e.g., `0x11` for Lossy Thumbnail).
*   **Size (4 bytes):** Entry size.
*   **Data (Variable):** Content.

## 5. Endianness
*   **Little-Endian:** Verified in width, height, and chunk size fields.
*   **Evidence:** The value `87 00 00 00` interpreted as `135` reflects the actual canvas dimensions observed in inspection tools.

## 6. Compression
*   **Zlib:** Evidence in some data blocks.
*   **DPCM (Differential Pulse Code Modulation):** Used for lossless thumbnails and layer pixel data (`lpix`). Requires reconstruction: `Pixel[n] = Pixel[n-1] + Delta[n]`.
*   **JPEG:** The `view` and `thum` chunks can contain JPEGs encapsulated in `JSSF` containers.

## 7. Image Data (Raster)
*   **Tiles:** SAI2 stores pixels in blocks (tiles), commonly 256x256 pixels.
*   **Channels:** Background storage format in BGRA (Blue, Green, Red, Alpha).
*   **Reconstruction:** Requires processing multiple associated `layr` and `lpix` chunks.

## 8. Embedded Thumbnail / Preview
*   **Is there a preview?** Yes, highly common.
*   **Chunk Tags:** `thum` (small thumbnail) and `view` (higher quality visualization).
*   **Format:**
    *   **Lossy:** JPEG stream within a `JSSF` header.
    *   **Lossless:** Raw DPCM data.
*   **Extraction:** Locate the `view` chunk in the descriptor list, look for the `JSSF` signature in the data, and extract the sequential JPEG.

## 9. Metadata
*   **History:** `hist` (or `normhist`) chunk contains UTF-16 strings with save and modification dates.
*   **Layer Names:** Stored in `layr` chunks.

## 10. Structural Reverse Engineering
*   **Block Container:** Extensible format via 4-letter tags.
*   **Resilient Scanning:** Due to the inconsistency of the `Chunk Count` field between versions, scanning the Chunk Table for known strings is mandatory for modern parsers.

## 11. Strategy for Parser Implementation
1.  **Validate Header:** Check for `SAI-CANVAS`.
2.  **Determine Chunk Count:** Test offset `0x28`. If zero, check `0x14` or perform a linear scan of ASCII tags starting at `0x40`.
3.  **Map Offsets:** Data Offset of a chunk `i` is `(Header + ListSize) + sum(Sizes i-1)`.
4.  **Prioritize Visualization:** Search for the `view` chunk. If it doesn't exist, use `thum`.

## 12. Parser Pseudocode
```pseudo
open file
read magic -> "SAI-CANVAS"
width = read_u32_le(0x20)
height = read_u32_le(0x24)

# Chunk list parsing
chunks = []
seek(0x40)
while current_pos < filesize:
    tag = read_string(4)
    if not is_ascii(tag): break
    id = read_u32_le()
    size = read_u64_le()
    chunks.append({tag, size})

# Offset calculation
data_start = current_pos
running_offset = data_start
for chunk in chunks:
    if chunk.tag == "view":
        seek(running_offset)
        extract_jssf_jpeg(chunk.size)
        return
    running_offset += chunk.size
```

## 13. Strategy for Thumbnail Generation
*   **High Fidelity:** Extract from the `view` chunk.
*   **Compatibility:** Implement the DPCM decoder for files saved in lossless mode.
*   **Pipeline:** `List Scan -> Offset Resolve -> JSSF Detect -> JPEG Decode`.

## 14. Strategy for Basic Visualization
*   Extraction of the embedded JPEG is the only practical way without implementing the proprietary tile and layer rendering engine.

## 15. Comparative Map Between Files
| File | Header Version | Detected Chunks | Resolution | Observations |
| :--- | :--- | :--- | :--- | :--- |
| `elfinha4.sai2` | TYPE0 | 135 | 135x276 | Official count at 0x28 is 0. |

## 16. Uncertain Points
*   **Field 0x14 (Confidence: 80%):** Appears to be an alternative counter or offset flag, given the value 2490 in a 135-chunk file.
*   **Per-Machine Encryption (Confidence: 60%):** The software has an option to save files that only open on the original PC; these files likely use the same container but with chunk data encrypted via AES or similar (based on hardware ID).

## 17. Technical Conclusion
The `.sai2` is a well-structured block format but with header variations that challenge static parsers. Thumbnail extraction via `view`/`thum` chunks is feasible and performant, provided the parser uses tag scanning to locate data correctly.
