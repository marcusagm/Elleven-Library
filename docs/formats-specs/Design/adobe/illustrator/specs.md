# Adobe Illustrator (.ai) File Format Technical Specification

## 1. Format Overview
*   **Extension Name:** `.ai` (Adobe Illustrator Artwork).
*   **Possible Origin:** Developed by Adobe Inc.
*   **Category:** Vector / Container (PDF-Hybrid).
*   **Magic Signature (Hexadecimal):** `25 50 44 46` (`%PDF`) for modern files (v9.0+). Legacy versions (v1-v8) use `25 21 50 53` (`%!PS`).
*   **Typical Size Observed:** 60 KB to 2 MB (depending on complexity and whether PDF compatibility is enabled).
*   **Variations Between Analyzed Files:** All analyzed samples follow the PDF container structure (PDF 1.5/1.6), acting as a "Dual Format" containing standard PDF data and encapsulated proprietary Illustrator data.

## 2. Global Binary Structure
The format functions as a PDF container that "hides" original Illustrator data within metadata streams and private objects.

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 8 bytes | `ASCII` | **Magic Header** | `%PDF-1.x` | Identified as PDF. |
| Variable | Variable | `Object` | **PDF Body** | Standard PDF objects. | Catalog, Pages, Streams. |
| Variable | Variable | `Metadata` | **XMP Block** | XML with metadata. | Contains Base64-encoded thumbnails. |
| Variable | Variable | `Stream` | **Private Data** | `%AI12_CompressedData` data. | Where the actual vector resides. |
| EOF | 5 bytes | `ASCII` | **Trailer** | `%%EOF` | End of PDF file. |

## 3. Main Header
*   **Detailed Structure:** Follows the PDF specification. The file begins with the version marker.
*   **Identified Fields:** PDF version (e.g., `1.5`, `1.6`).
*   **Endianness:** Big-endian (Network/PDF standard).
*   **Special Marker:** Immediately after the header, there is usually a binary block `%âãÏÓ` to indicate the file contains 8-bit binary data.

## 4. Identified Internal Structures

### 4.1. XMP Block (Extensible Metadata Platform)
*   **Initial Offset:** Variable (searched by the `<x:xmpmeta>` tag).
*   **Function:** Stores structured metadata in XML.
*   **Thumbnail:** Located within `<xmpGImg:image>` tags in Base64-encoded JPEG format.

### 4.2. Private Data Block (Illustrator Proprietary)
*   **Signature:** `%AI12_CompressedData` (or similar depending on version).
*   **Structure:** A FlateDecode (Zlib) stream containing the original Illustrator vector object graph.
*   **Function:** Allows Illustrator to reopen the file with all layers and filters editable, even if the standard PDF does not support all features.

### 4.3. Legacy Preview Block (AI7_Thumbnail)
*   **Signature:** `%AI7_Thumbnail`.
*   **Structure:** Contains width, height, bit depth, and a hexadecimal stream (`%%BeginData`).
*   **Function:** Used by older versions or fast preview plugins.

## 5. Endianness
*   **Big-endian:** Standard inherited from PostScript and adopted by PDF.
*   **Evidence Found:** All stream length values and object identifiers in the PDF container follow big-endian order.

## 6. Compression
*   **Algorithm:** **Zlib / FlateDecode**.
*   **Signature:** `78 9C` (Zlib Default Compression) frequently found after commands like `/Filter/FlateDecode`.
*   **Usage:** Applied to private data streams and page content objects.

## 7. Image Data (Pre-render)
*   **Dimensions:** Defined in the PDF `/MediaBox` and the `/Page` dictionary.
*   **Bit Depth:** Generally 8 bits per channel for previews.
*   **Reconstruction:** Basic visualization is performed by rendering the standard PDF stream contained in the file.

## 8. Embedded Thumbnail / Preview
*   **Existence:** Yes, multiple levels.
*   **Extraction:**
    1.  **Via XMP:** Decode Base64 from the `<xmpGImg:image>` tag.
    2.  **Via /Thumb:** PDF attribute referencing an image object.
    3.  **Via AI7:** Parse `%AI7_Thumbnail` and convert the hex stream.
*   **Format:** JPEG (in XMP) or Indexed/RGB Bitmap (in AI7).

## 9. Metadata
*   **Strings Found:** "Adobe Illustrator", creator version (e.g., Adobe Illustrator 24.1), creation date, layer titles.
*   **Structure:** XML (XMP) and PDF `/Info` dictionaries.

## 10. Structural Reverse Engineering
*   **Container:** The file is a hybrid. If renamed to `.pdf`, it opens in common readers.
*   **TLV:** PDF uses an indexed object structure (`xref table`) that functions as internal pointers.
*   **Redundancy:** Illustrator saves the drawing twice: once as simple PDF objects (for compatibility) and once as its proprietary compressed format (for editing).

## 11. Strategy for Parser Implementation
1.  **Header Validation:** Check for `%PDF-`.
2.  **Metadata Scanning:** Search for `xmp:Thumbnails` for ultra-fast preview extraction without processing the vector.
3.  ** /Thumb Object Location:** Check the first page dictionary.
4.  **Incremental Parsing:** If original vector reconstruction is needed, locate the `/Filter /FlateDecode` stream associated with the `%AIXX_CompressedData` marker.

## 12. Parser Pseudocode
```pseudo
open file
read magic ("%PDF-")
if not found, check legacy magic ("%!PS-Adobe")

find cross-reference table (xref) at end of file
locate Catalog object
locate Metadata stream

# Extract Thumbnail
search for "<xmpGImg:image>" in entire file (fast scan)
if found:
    extract Base64 content
    decode to JPEG buffer
    return thumbnail

# Fallback
search for "/Page" objects
check for "/Thumb" key
extract referenced Image object stream
return image
```

## 13. Strategy for Thumbnail Generation
*   **Best Approach:** Use the embedded XMP preview. It is a pre-rendered JPEG image and easily accessible.
*   **Complexity:** Low (Regex to find tags + Base64 decoding).
*   **Pipeline:** `Find Tag -> Extract -> Base64 Decode -> Save as .jpg`.

## 14. Strategy for Basic Visualization
*   Use standard PDF libraries (Poppler, PDF.js, MuPDF) to render page 1.
*   It is not necessary to implement Adobe's proprietary vector engine for simple visualization.

## 15. Comparative Map Between Files
| File | Structure | PDF Version | Thumbnail | Observations |
| :--- | :--- | :--- | :--- | :--- |
| `Logo.ai` | Hybrid | 1.6 | XMP/Base64 | Modern structure. |
| `sample.ai` | Hybrid | 1.5 | XMP/Base64 | Follows Adobe CC standard. |
| `Cake box...` | Hybrid | 1.5 | AI7/XMP | Contains legacy and modern previews. |

## 16. Uncertain Points
*   **PGF (Progressive Graphics File):** Some files cite `Adobe_Direct_PGF`. The internal structure of this binary stream is opaque (Confidence: 30%).
*   **Proprietary Blending Modes:** Certain Illustrator transparency effects may not appear correctly in generic PDF renderers if they are only in the Private Data (Confidence: 80%).

## 17. Technical Conclusion
The contemporary `.ai` format is a classic example of encapsulating proprietary data in an open container (PDF). Thumbnail extraction is facilitated by XMP metadata redundancy, while full vector parsing requires a complete PDF engine and knowledge of Adobe's private extensions for perfect reconstruction.
