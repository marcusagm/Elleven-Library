# Technical Analysis: Encapsulated PostScript (.eps) Format

## 1. Format Overview

*   **Extension:** `.eps` (Encapsulated PostScript).
*   **Software:** Adobe Illustrator, CorelDRAW, various DTP software.
*   **Category:** Vector Graphics / Page Description.
*   **Versions:** EPSF 1.2, 2.0, 3.0.
*   **Signatures:**
    *   **Plain Text:** `%!PS-Adobe-` (`25 21 50 53 2D 41 64 6F 62 65 2D`).
    *   **Binary (with preview):** `C5 D0 D3 C6` (`0xC6D3D0C5` in little-endian order).

---

## 2. Structure (Binary EPS)

Binary EPS files contain a 30-byte header followed by raw PostScript code and optional binary previews (TIFF or WMF).

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `C5 D0 D3 C6`. |
| `0x04` | 4 bytes | `u32` | **PS Offset** | Offset to start of PostScript code. |
| `0x08` | 4 bytes | `u32` | **PS Size** | Length of PostScript code. |
| `0x0C` | 4 bytes | `u32` | **Meta Offset** | Offset to Metafile preview (WMF). |
| `0x10` | 4 bytes | `u32` | **Meta Size** | Length of Metafile preview. |
| `0x14` | 4 bytes | `u32` | **TIFF Offset** | Offset to TIFF preview. |
| `0x18` | 4 bytes | `u32` | **TIFF Size** | Length of TIFF preview. |
| `0x1C` | 2 bytes | `u16` | **Checksum** | Header checksum. |

---

## 3. Thumbnail Extraction Strategies

### 3.1. Binary Header (Recommended)
If the file starts with `C5 D0 D3 C6`:
1.  Read the header (30 bytes).
2.  If `TIFF Size > 0`: Extract the segment at `TIFF Offset` for `TIFF Size`. This is a valid TIFF image.
3.  If `Meta Size > 0`: Extract the segment at `Meta Offset`. This is a WMF (usually needs conversion).

### 3.2. Plain Text (XMP)
Modern EPS files created by Adobe Illustrator often contain XMP metadata in comments.
1.  Scan for `%%BeginMetadata` or `<xmp:Thumbnails>`.
2.  Extract the Base64-encoded JPEG from the XML.

### 3.3. EPSI (Interchange)
Some old files have a hex-encoded preview in the PostScript comments (`%%BeginPreview`).
1.  Parse `%%BeginPreview: [width] [height] [depth] [lines]`.
2.  Hex-decode the subsequent lines into a bitmap.

---

## 4. Extraction Strategy Plan

### 4.1. Identification
1.  Read 4 bytes.
2.  If `C5 D0 D3 C6`: Absolute offset extraction (Fastest).
3.  If `%!PS`: Text parsing (O(N) complexity).

### 4.2. Fallback
If no binary preview or XMP is found, the file must be rendered using a PostScript interpreter (e.g., Ghostscript), which is computationally heavy.

---

## 5. Uncertainties
*   **Dual Previews:** Some files contain both WMF and TIFF. Multi-platform assistants should prioritize TIFF.
*   **Embedded PostScript Complexity:** Parsing PostScript comments is non-trivial as they can be nested or malformed by different exporters.
