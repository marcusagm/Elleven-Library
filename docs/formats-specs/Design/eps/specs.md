# Technical Analysis: Encapsulated PostScript (.eps) File Format

## 1. Format Overview
*   **Extension Name:** `.eps` (Encapsulated PostScript).
*   **Possible Origin:** Developed by Adobe Systems in 1987.
*   **Category:** Vector Graphics Document / Container.
*   **Magic Signature (Hexadecimal):**
    *   **Binary EPS:** `C5 D0 D3 C6` (Little-Endian: `0xC6D3D0C5`).
    *   **ASCII EPS:** `25 21 50 53` (`%!PS`).
*   **Typical Size Observed:** 3 KB to 10 MB (depending on vector complexity and the presence of TIFF previews).
*   **Variations Between Analyzed Files:** Both purely textual files (Pure PostScript) and binary files (Adobe Generic Header) that embed ASCII PostScript along with binary thumbnails were observed.

## 2. Global Binary Structure

### 2.1. Binary EPS (Adobe Generic)
Files that use the binary header facilitate fast visualization without the need for a full PostScript interpreter.

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `C5 D0 D3 C6`. | Little-endian order. |
| `0x04` | 4 bytes | `u32` | **PS Offset** | Start of PostScript code. | Usually 30 (immediately after header). |
| `0x08` | 4 bytes | `u32` | **PS Size** | PostScript code size. | |
| `0x0C` | 4 bytes | `u32` | **WMF Offset** | Start of WMF preview. | 0 if not present. |
| `0x10` | 4 bytes | `u32` | **WMF Size** | WMF preview size. | |
| `0x14` | 4 bytes | `u32` | **TIFF Offset** | Start of TIFF preview. | Commonly located after PostScript. |
| `0x18` | 4 bytes | `u32` | **TIFF Size** | TIFF preview size. | |
| `0x1C` | 2 bytes | `u16` | **Checksum** | Header checksum. | Frequently `0xFFFF`. |

### 2.2. ASCII EPS (Raw PostScript)
Follows Document Structuring Conventions (DSC).

| Offset | Size | Type | Description |
| :--- | :--- | :--- | :--- |
| `0x00` | Var | `ASCII` | Starts with `%!PS-Adobe-3.0 EPSF-3.0`. |
| Var    | Var | `Comment`| DSC Comments (e.g., `%%Title`, `%%BoundingBox`). |
| Var    | Var | `Metadata` | XMP block (XML) embedded in comments. |
| Var    | Var | `Code` | PostScript operators (e.g., `moveto`, `lineto`). |

## 3. Main Header
### 3.1. Binary Header
*   **Structure:** 30 fixed bytes.
*   **Fields:** Absolute pointers to the three possible segments (PostScript, Windows Metafile, TIFF).
*   **Endianness:** Little-endian.

### 3.2. ASCII Header
*   **Structure:** Free text following proprietary conventions.
*   **Fields:** EPSF version, Creator, Date, Bounding Box.
*   **Endianness:** N/A (Textual).

## 4. Identified Internal Structures

### 4.1. PostScript Block (Mandatory)
*   Contains the actual vector description.
*   In binary files, the offset points to this block.
*   Ends with the `showpage` operator or `%%EOF`.

### 4.2. TIFF Preview Block (Optional)
*   **Signature:** `49 49 2A 00` (II) or `4D 4D 00 2A` (MM).
*   **Function:** A low-resolution raster image for fast display in design software that does not render PostScript in real-time.

### 4.3. XMP Metadata Block (Modern)
*   **Location:** Frequently within the PostScript section as an XML block.
*   **Thumbnail:** `<xmpGImg:image>` tags contain a Base64 string representing a JPEG.

## 5. Endianness
*   **Binary Header:** **Little-Endian**.
*   **Embedded TIFF:** Can be **Little-Endian (II)** or **Big-Endian (MM)**.
*   **PostScript:** The code itself is textual.

## 6. Compression
*   **Indication:** PostScript can use filters like `/FlateDecode` (Zlib) or `/ASCII85Decode`.
*   **Previews:** TIFF preview might be compressed with PackBits or LZW.
*   **XMP Thumbnails:** Standard Base64-encoded JPEGs.

## 7. Image Data
*   **Vector:** Represented by coordinates and PS commands.
*   **Embedded Raster:** Can exist via `image` command in PostScript or via binary preview.

## 8. Embedded Thumbnail / Preview
*   **How to automatically detect:**
    1.  Check for `C5 D0 D3 C6` magic. If so, read offset at `0x14`.
    2.  If ASCII, search for the `<xmpGImg:image>` string for modern thumbnails.
    3.  If legacy ASCII, search for `%%BeginPreview`.

## 9. Metadata
*   **DSC:** `%%Title`, `%%Creator`, `%%CreationDate`.
*   **XMP:** Embedded XML block with rich authorship and editing history information (Adobe Creative Cloud).

## 10. Structural Reverse Engineering
*   **Hybrid Container:** EPS is one of the rare formats that mixes fixed-length binary headers with variable-length textual data bodies.
*   **Pointer System:** The binary header uses an absolute offset table system, allowing to skip PostScript and go straight to the preview.

## 11. Strategy for Parser Implementation
1.  **Differentiation:** Read first 4 bytes.
2.  **Binary Case:** Extract header 30 bytes, validate TIFF offset, and extract sub-file.
3.  **ASCII Case:**
    - Scan for metadata markers (`%%BeginMetadata`, `<x:xmpmeta>`).
    - Decode XMP Thumbnail if present (Base64 -> JPEG).
4.  **Error Handling:** Validate if offsets read in the binary header do not exceed the total file size.

## 12. Parser Pseudocode
```pseudo
open file
read magic(4)
if magic == 0xC6D3D0C5:
    header = read(30)
    tiff_offset = header.get_u32(20)
    tiff_size = header.get_u32(24)
    if tiff_size > 0:
        seek(tiff_offset)
        return extract_tiff(tiff_size)

else if (magic == "%!PS"):
    content = read_all()
    if find("<xmpGImg:image>"):
        b64_data = extract_between_tags("<xmpGImg:image>", "</xmpGImg:image>")
        return decode_base64_to_jpeg(b64_data)
    else if find("%%BeginPreview"):
        # Legacy hex preview parsing
        return parse_hex_preview()
```

## 13. Strategy for Thumbnail Generation
*   **High Speed:** Prioritize binary structure TIFF preview or XMP JPEG.
*   **Complexity:**
    - TIFF (Binary): Low.
    - XMP (ASCII): Medium (requires Base64 decoding).
    - Pure PostScript (Without Preview): High (requires vector renderer like Ghostscript).

## 14. Strategy for Basic Visualization
*   Upon finding binary TIFF, display it as a standard image.
*   Otherwise, render only visual metadata if available.

## 15. Comparative Map Between Files
| File | Structure | Preview | Observations |
| :--- | :--- | :--- | :--- |
| `Quran 27-40...` | Binary | TIFF (66 KB) | High-fidelity preview present. |
| `i18k_e46y...` | ASCII | XMP/JPEG | Modern Adobe format. |
| `knightstour.eps` | ASCII | None | Pure PostScript, simple vector. |

## 16. Uncertain Points
*   **WMF Compatibility:** WMF (Windows feature) preview has fallen out of use, but old files may still contain it (Confidence: 100% of presence, 40% of modern utility).
*   **Checksum Calculation:** The 2-byte checksum field is rarely validated by modern software, which relies only on offsets (Confidence: 80%).

## 17. Technical Conclusion
The `.eps` is a well-documented transition format, but requires dual treatment to handle its binary and textual variants. Thumbnail extraction is extremely efficient in files generated by professional software (Adobe/Corel), but manually generated minimalist files require full PostScript code rendering.
