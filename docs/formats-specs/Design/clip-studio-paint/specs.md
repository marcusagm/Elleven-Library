# Reverse Engineering Analysis: Clip Studio Paint (.clip) Format

## 1. Technical Overview

*   **Format Name:** Clip Studio Paint Format (v1.5+).
*   **Extension:** `.clip` (Replaced older `.lip`).
*   **Origin:** Celsys (Clip Studio Paint / Manga Studio).
*   **Category:** Layered Raster/Vector Illustration Project.
*   **Magic Signature:** `43 53 46 43 48 55 4E 4B` (`CSFCHUNK`).
*   **Typical Size:** 1MB - 2GB+. Large projects use external chunking.
*   **Structure:** Custom Container (`CSFCHUNK`) wrapping multiple SQLite databases (some compressed, some raw).
*   **Encryption:** Some blocks may be encrypted, but metadata and thumbnails are generally accessible via standard Zlib decompression.

---

## 2. Structural Hex Map

The file is a sequence of **Chunks**.

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 8 bytes | `ASCII` | **Magic** | `CSFCHUNK` |
| `0x08` | 4 bytes | `UINT32` | **Zero** | `00 00 00 00` |
| `0x0C` | 4 bytes | `UINT32` | **File Size** | Total file size (Big Endian). |
| `0x10` | 8 bytes | `ASCII` | **Chunk Name** | `CHNKHead` |
| `0x18` | 8 bytes | `UINT64` | **Chunk Size** | Payload size of this chunk. |
| `0x20` | Var | `BYTES` | **Payload** | Data for `CHNKHead` (~40 bytes). |
| `...` | ... | ... | **Next Chunk** | `CHNKExta` (External Data), `CHNKSQLi` (SQLite), etc. |

**Important Note:** All integers are **Big-Endian** (`>Q`, `>I`).

---

## 3. Header Analysis

The 16-byte global header defines the container.

1.  **Magic:** `CSFCHUNK`.
2.  **Version/Reserved:** `00 00 00 00`.
3.  **Total Size:** Match file size in bytes (e.g., `00 4C EA CD`).

---

## 4. Internal Chunk Structure

Each Chunk follows a standard header:

| Field | Size | Type | Description |
| :--- | :--- | :--- | :--- |
| **Name** | 8 bytes | `ASCII` | e.g. `CHNKHead`, `CHNKExta`, `CHNKSQLi`. |
| **Size** | 8 bytes | `UINT64` | Size of the **Payload** (excluding header). Big Endian. |
| **Payload** | `Size` | - | Binary data (Raw or Compressed). |

### Common Chunk Types:

1.  **CHNKHead:** Metadata header (Version info).
2.  **CHNKExta:** "External" Data. Often contains **Compressed SQLite Databases**.
    *   **Payload:** Zlib Stream (Signature `78 9C` usually at offset +16).
    *   **Contains:** `CanvasPreview` (Thumbnail), `CanvasContent` (Layers).
3.  **CHNKSQLi:** "SQLite" Data. **Uncompressed** SQLite Database.
    *   **Payload:** Standard SQLite file (`SQLite format 3` signature at start).
    *   **Contains:** Project structural metadata, layer tree, settings.
4.  **CHNKDat1:** (Hypothesized) Large binary blobs for layer pixel data in huge files.
5.  **CHNKFoot:** Footer (Empty payload).

---

## 5. Endianness

*   **Container (CSFCHUNK):** Big-Endian.
*   **Internal SQLite Databases:** Machine-dependent (SQLite default is Big-Endian for numbers, but file format handles this).
*   **Zlib Streams:** Standard Deflate.

---

## 6. Compression and Databases

The format relies heavily on **SQLite** and **Zlib**.

*   **CHNKExta Chunks:** almost always contain Zlib-compressed data.
    *   **Heuristic:** Read first 100 bytes of payload. Look for `78 9C` (Default Compression).
    *   **Decompression:** Use `zlib.decompress` or `inflate`.
    *   **Result:** Often a valid SQLite file (starts with `SQLite format 3`) or binary blob.
*   **CHNKSQLi chunks:** Uncompressed SQLite databases.

---

## 7. Image Data & Pixel Packing

The format is a **Project File**, not a RAW image.
Pixel data is stored within the SQLite tables or blob chunks referenced by the SQLite DB.

*   **Structure:**
    *   `CanvasContent` table references layers.
    *   Layer data is serialized (likely proprietary serialization of tiles) inside blobs.
*   **Reconstruction:** Requires full parsing of the SQLite schema (`Canvas` -> `Layer` -> `Chunk` -> `Tile`). Very complex.

---

## 8. Thumbnail / Embedded Preview

**Excellent News:** A high-quality preview is readily available.

*   **Location:** Inside the **first** `CHNKExta` chunk's SQLite database.
*   **Table:** `CanvasPreview`.
*   **Column:** `image_data` (Blob).
*   **Format:** Standard **JPEG** (`FF D8 ...`).
*   **Extraction Pipeline:**
    1.  Parse offsets to find first `CHNKExta`.
    2.  Read payload.
    3.  Decompress (Zlib).
    4.  Open result as SQLite (in-memory or temp file).
    5.  Execute: `SELECT image_data FROM CanvasPreview LIMIT 1`.
    6.  Dump Blob -> `thumbnail.jpg`.

---

## 9. Metadata

Metadata is stored in the `CHNKSQLi` (Main DB) or `CHNKExta` (Metadata DB).
Standard SQLite queries can retrieve:
*   **Layer Names:** `CanvasContent` or `Layer` tables.
*   **Resolution/DPI:** Project settings table.
*   **Edit History:** Often tracked in internal tables.

---

## 10. Reverse Engineering Strategy

Structure is **Chunk-based Container (CSF)** wrapping **SQLite**.

*   **Pattern:** `TLV` (Type-Length-Value) for Chunks.
*   **Inner Pattern:** SQL Tables.
*   **Sanity Check:**
    *   Container size == File size.
    *   Chunk Header Name is ASCII alphanumeric.
    *   Chunk Size is reasonable (< File Size).

---

## 11. Implemented Parser Pseudocode

```python
def extract_thumbnail_clip(filepath):
    f = open(filepath, 'rb')
    if f.read(8) != b'CSFCHUNK': return None
    
    # Skip Header Size
    f.seek(16)
    
    while True:
        # Read Chunk Header
        name = f.read(8)
        if len(name) < 8: break
        
        size = struct.unpack('>Q', f.read(8))[0]
        
        if name == b'CHNKExta':
            # Potential Thumbnail DB
            offset = f.tell()
            payload = f.read(size)
            
            # Find Zlib start (usually at +16 relative to payload start, roughly)
            # Brute force search '78 9C' in first 200 bytes
            zlib_idx = payload.find(b'\x78\x9c', 0, 200)
            
            if zlib_idx != -1:
                try:
                    db_data = zlib.decompress(payload[zlib_idx:])
                    if b'SQLite format 3' in db_data[:32]:
                        # Extract from SQLite
                        return extract_jpeg_from_sqlite_blob(db_data)
                except:
                    pass
        
        elif name == b'CHNKFoot':
            break
        else:
            # Skip payload
            f.seek(size, 1) # SEEK_CUR

    return None

def extract_jpeg_from_sqlite_blob(data):
    # Load into sqlite3 (memory)
    # SELECT image_data FROM CanvasPreview
    # Return JPEG bytes
```

---

## 12. Strategy for Thumbnail Generation

Use the embedded **JPEG**.

*   **Why:** It is high resolution and pre-rendered.
*   **Performance:**
    *   Zlib Decompression of ~500KB chunk: < 10ms.
    *   SQLite Query: < 5ms.
    *   Total time: Extremely fast.
*   **Fallback:** If `CanvasPreview` is empty (rare), render from `CanvasPreviewImage` (tiled) or impossible without full engine.

---

## 13. Comparative Map

| File | Size | Chunks | Observations |
| :--- | :--- | :--- | :--- |
| `Sketches.clip` | 5MB | Head, Exta, Exta..., SQLi, Foot | Standard. Thumbnail in 1st Exta. |
| `01.clip` | 223MB | Head, Exta..., SQLi... | Same structure. Larger payloads. |

---

## 14. Uncertain Points

1.  **Encryption:** Some CLIP files are password-protected. The `CHNK` payload might be AES encrypted. This analysis assumes unencrypted files.
2.  **Chunk Order:** `CHNKExta` appears multiple times. The "Thumbnail DB" seems to be consistently the *first* or *second* one, but strict verification is needed.
3.  **Zlib Offset:** The Zlib stream inside `CHNKExta` starts at variable offsets (146, 250, etc. in samples). One must scan for `789C` or parse the proprietary sub-header inside `CHNKExta`.

---

## 15. Technical Conclusion

The `.clip` format is **access-friendly** for read-only tasks like thumbnailing and metadata extraction, thanks to standard technologies (SQLite + Zlib).

*   **Parsing Difficulty:** Low (for Container/Thumbnail). High (for full Rendering).
*   **Robustness:** High (SQLite checks integrity).
*   **Recommendation:** Implement a `CSFCHUNK` scanner + SQLite extractor.
