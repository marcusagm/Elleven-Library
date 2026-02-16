# Technical Analysis: Autodesk SketchBook (.tif / .skba) Format

## 1. Format Overview

*   **Extension:** `.tif` (Legacy/Desktop), `.skbp` (modern), `.skba` (archive).
*   **Software:** Autodesk SketchBook / Sketchbook.
*   **Category:** Layered Raster Image.
*   **Signature:** `II*` (Little-Endian TIFF) or `MM*` (Big-Endian TIFF).
*   **Container:** **TIFF** (Tagged Image File Format).

---

## 2. Structure

SketchBook uses the TIFF structure to store both the merged preview and the individual layer data.

| Component | TIFF Tag / IFD | Description |
| :--- | :--- | :--- |
| **Merged Image** | 1st IFD (Default) | Standard flattened version of the artwork. |
| **Thumbnail** | SubIFD or Private Tag | Smaller version for file explorer. |
| **Layer Data** | Private Tags (34377, 37724) | Proprietary blocks containing layer transparency, blend modes, and pixel data. |
| **App Metadata** | Tag 50648 | Contains version info like "Sketchbook Pro 9.3". |

---

## 3. Thumbnail Extraction Strategy

Since the format is a standard TIFF:
1.  **Standard TIFF Decoding:** Most image libraries will automatically read the first IFD, which contains the merged preview.
2.  **Fast Thumbnails:** Extracting the first IFD with a reduced scale is the most compatible method.
3.  **Private Data:** The layer data is stored in binary blobs within private tags. Extracting individual layers requires specialized TIFF tag parsing.

---

## 4. Implementation Strategy

### 4.1. Fast Extraction
1.  Verify `II*` or `MM*` header.
2.  Use a TIFF-capable library to read the primary image data.
3.  No proprietary decompression is usually required for the merged image (often use LZW or PackBits).

---

## 5. Uncertainties
*   **Mobile Versions:** Sketchbook Mobile sometimes uses a ZIP-based format (`.skba`) instead of TIFF. These are ZIP archives containing a `manifest.json` and several PNG/TIFF assets.
*   **Blend Modes:** While the merged image is easy to extract, the individual layer blend modes are stored in the private metadata and may not follow standard PDF/Photoshop blending math.
