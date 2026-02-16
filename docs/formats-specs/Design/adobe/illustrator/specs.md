# Technical Analysis: Adobe Illustrator (.ai) Format

## 1. Format Overview

*   **Extension:** `.ai` (Adobe Illustrator Artwork).
*   **Software:** Adobe Illustrator.
*   **Category:** Vector Graphics.
*   **Versions:**
    *   **Legacy (v1.0 to v8.0):** PostScript-based flat file.
    *   **Modern (v9.0+):** PDF-based container (usually PDF 1.4+).
*   **Signature (Modern):** `%PDF-` (`25 50 44 46`).
*   **Signature (Legacy):** `%!PS-Adobe-` (`25 21 50 53 2D 41 64 6F 62 65 2D`).

---

## 2. Structure (Modern PDF-based)

Modern `.ai` files are valid **PDF files** that contain additional private Adobe data hidden in specific PDF objects.

| Section | Description |
| :--- | :--- |
| **PDF Body** | Standard PDF objects (Catalog, Pages, Streams). |
| **Private Data** | Encapsulated PostScript (EPS) or proprietary data used by Illustrator to reconstruct the vector artwork. |
| **PGF Section** | "Progressive Graphics File" - a proprietary Adobe format sometimes embedded within the PDF. |

---

## 3. Thumbnail / Preview Strategies

There are three primary locations where a thumbnail can be found in a modern `.ai` file:

### 3.1. Standard PDF Thumbnail (`/Thumb`)

The PDF standard defines a `/Thumb` attribute in the Page object dictionary.
*   **Location:** Referenced in a `/Page` dictionary.
*   **Format:** A PDF Image Stream (usually DCTDecode/JPEG or FlateDecode/Raw).
*   **Note:** This is the most "legit" way to get a preview for any PDF-based tool.

### 3.2. Adobe XMP Metadata (`<xmp:Thumbnails>`)

Illustrator often embeds XMP metadata (XML-based) inside a PDF Metadata stream.
*   **Location:** Inside the first `/Metadata` stream object.
*   **Format:** Base64 encoded JPEG in the `xmp:Thumbnails` property.

### 3.3. Legacy AI7 Thumbnail (`%AI7_Thumbnail`)

Carried over from the legacy PostScript format, this is often present in the private data stream for backwards compatibility.
*   **Format:** Hex-encoded raw RGB or Indexed data.
*   **Header:** `%AI7_Thumbnail: [width] [height] [depth]`.

---

## 4. Extraction Strategy (Modern Files)

Since modern `.ai` files are PDF-compatible, the best strategy for a fast thumbnail in an agentic assistant is:

1.  **PDF Parsing:** Identify the first `/Page` object.
2.  **Locate `/Thumb`:** If a `/Thumb` key exists, extract the referenced stream.
3.  **JPEG Fallback:** If the `/Thumb` stream uses `DCTDecode`, it's a standard JPEG.
4.  **XMP Fallback:** If PDF thumbnailing is not implemented, scan for `<xmpGImg:image>` tags in the XML metadata section and decode the Base64 payload.

---

## 5. Mappings across Versions

| Feature | Legacy (< 9.0) | Modern (9.0+) |
| :--- | :--- | :--- |
| **Container** | PostScript | PDF |
| **Thumbnail** | `%AI7_Thumbnail` | PDF `/Thumb` or XMP |
| **Compression** | None / RLE | Flate (Zlib) / DCT (JPEG) |

---

## 6. Uncertainties

*   **PDF Compatibility:** Users can choose to save `.ai` files "without PDF compatibility". In this case, the file does NOT start with `%PDF` and instead relies entirely on the proprietary Adobe content. These files are much harder to parse and usually use the older binary PGF format.
*   **PGF Data:** The PGF format is undocumented and used for fast previews inside Illustrator. If the PDF portion is missing, extracting a thumbnail requires specialized PGF parsing.
