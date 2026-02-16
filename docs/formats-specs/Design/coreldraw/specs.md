# Technical Analysis: CorelDRAW (.cdr) Format

## 1. Format Overview

*   **Extension:** `.cdr`
*   **Software:** CorelDRAW.
*   **Category:** Vector Graphics.
*   **Versions:**
    *   **Legacy (v1 to v13/X3):** RIFF-based binary format.
    *   **Modern (v14/X4+):** ZIP-based container (similar to OpenDocument/OOXML).
*   **Signature (Modern):** `PK\x03\x04` (ZIP).
*   **Signature (Legacy):** `RIFF....CDRB` or `RIFF....CDR[v]` (where `[v]` is version indicator).

---

## 2. Structure (Modern X4+)

Modern `.cdr` files are **ZIP archives** containing XML metadata and binary assets.

| Path | Description |
| :--- | :--- |
| `mimetype` | Contains `application/x-vnd.corel.zcreate`. |
| `content/data/` | Private data blocks (e.g. `page1.dat`). |
| `previews/` | **Critical:** Contains thumbnail and page previews. |
| `META-INF/` | Metadata and manifest files. |

---

## 3. Thumbnail Extraction Strategies

### 3.1. Modern Versions (ZIP)

The thumbnail is a standard PNG or BMP file located inside the ZIP structure.
*   **Path:** `previews/thumbnail.png` (or `.bmp` for some versions).
*   **Fallback:** `previews/page1.png`.
*   **Method:** standard ZIP decompression.

### 3.2. Legacy Versions (RIFF)

Legacy files use the RIFF (Resource Interchange File Format) container.
*   **Chunk Identifying:** Look for a chunk with the four-letter ID `DISP` or `icp0`.
*   **Format:** Usually a Windows Metafile (WMF) or a Bitmap (BMP).
*   **Structure:**
    *   Header: `RIFF` (4) + Size (4) + `CDRB` (4).
    *   Walk chunks until a preview block is found.

---

## 4. Extraction Strategy Plan

### 4.1. Fast Determination
1.  Read the first 4 bytes.
2.  If `PK\x03\x04`: Use ZIP strategy.
3.  If `RIFF`: Use RIFF strategy (scan for `DISP` chunk).

### 4.2. ZIP Strategy (Primary)
1.  Open as ZIP.
2.  Extract `previews/thumbnail.png`.

---

## 5. Uncertainties

*   **Proprietary CDR Data:** The content inside `content/data/*.dat` is highly proprietary and version-dependent. Reconstruction of the vector artwork without CorelDRAW is extremely difficult.
*   **Legacy File Recovery:** Very old CorelDRAW files (v1-v5) might not have embedded previews if saved with specific options, making them invisible to explorer-style assistants without full vector rendering.
