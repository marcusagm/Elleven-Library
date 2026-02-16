# Technical Analysis: Paint Tool SAI 2 (.sai2) Format

## 1. Format Overview

*   **Extension:** `.sai2`
*   **Software:** Systemax Paint Tool SAI 2.
*   **Category:** Layered Raster Image.
*   **Signature:** `SAI-CANVAS-TYPE0` (`53 41 49 2d 43 41 4e 56 41 53 2d 54 59 50 45 30`).
*   **Endianness:** **Little-Endian**.
*   **Container:** Chunk-based binary format.

---

## 2. Structure

The file starts with a fixed signature followed by a series of chunks. Each chunk has an ID and a size.

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 16 bytes | `ASCII` | **Magic** | `SAI-CANVAS-TYPE0` |
| `0x10` | 4 bytes | `u32` | **Canvas Width** | Width in pixels. |
| `0x14` | 4 bytes | `u32` | **Canvas Height** | Height in pixels. |
| `0x18` | ... | ... | **Chunks** | Sequence of data blocks. |

---

## 3. Thumbnail Extraction

Thumbnails in SAI2 are stored in a specialized chunk, typically with ID `0x1010` (Virtual ID) or identifiable by its position early in the file.

### 3.1. Compression
SAI2 thumbnails often use a custom **DPCM (Differential Pulse Code Modulation)** algorithm combined with a lossless compressor.
*   **Algorithm:** Predictor-based encoding where each pixel is stored as a difference from the previous one.
*   **Color Space:** Usually **RGBA** or **BGRA**.

### 3.2. Extraction Challenges
*   **No Standard Metadata:** SAI2 does not use EXIF or XMP.
*   **Proprietary Decoding:** Reaching the thumbnail requires iterating through binary chunks and applying the proprietary DPCM restoration logic.
*   **DPCM Details:**
    *   The first pixel is raw.
    *   Subsequent pixels are `Current = (Previous + Delta) % 256`.

---

## 4. Implementation Strategy

### 4.1. Fast Scanning
1.  Verify the `SAI-CANVAS-TYPE0` header.
2.  Search for the thumbnail chunk (usually follows the canvas properties).
3.  Read the compressed chunk data.
4.  Apply DPCM decoding to the byte stream to reconstruct the pixels.

---

## 5. Uncertainties
*   **Chunk IDs:** The exact mapping of chunk IDs to data types is partially reverse-engineered but not officially documented.
*   **DPCM Variants:** Different versions of SAI2 (Technical Preview vs Stable) might use slightly different predictor algorithms or headers within the thumbnail chunk.
