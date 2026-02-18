# Technical Specification: Penpot Project File (.penpot)

## 1. Format Overview
*   **Extension Name:** `.penpot`
*   **Possible Origin:** [Penpot](https://penpot.app/), an open-source design and prototyping platform.
*   **Category:** Collaborative Design Project / Container.
*   **Magic Signature (Hexadecimal):**
    *   **V1 (Legacy/Standard):** `50 4B 03 04` (ZIP Container).
    *   **V2 (Modern/Optimized):** `01 0B 1A 86 50 63 A1 5F C5 00 00 00 00 00 00 00 01` (Binary Header).
*   **Typical Size Observed:** 30 KB (templates) to 60+ MB (complex UI kits).
*   **Variations Between Analyzed Files:** Coexistence of traditional ZIP containers and a new single-file format based on Zstandard compression was observed.

## 2. Global Binary Structure (V1 - ZIP)

| Offset | Size | Type | Field Name | Description | Observations |
| ------ | ------- | ---- | ------------- | --------- | ----------- |
| `0x00` | 4 bytes | `u32` | **ZIP Magic** | `0x04034B50`. | Standard PKZIP local header. |
| `Var`  | Var     | `Path`| **files/**    | JSON project metadata. | Nested UUID structure. |
| `Var`  | Var     | `Path`| **objects/**  | Binary assets. | PNG, JPG, SVG, and icons. |

## 2.1 Global Binary Structure (V2 - Zstd)

| Offset | Size | Type | Field Name | Description | Observations |
| ------ | ------- | ---- | ------------- | --------- | ----------- |
| `0x00` | 17 bytes| `Raw` | **Magic Header**| `01 0B 1A 86...` | Proprietary Penpot marker. |
| `0x11` | Var     | `Zstd`| **Zstd Payload**| Compressed data stream. | Magic `0xFD2FB528`. |

## 3. Main Header (V2)

*   **Structure (17 bytes):**
    *   `0x00`: `01` Version?
    *   `0x01 - 0x09`: Core signature `0B 1A 86 50 63 A1 5F C5`.
    *   `0x0A - 0x0F`: Null padding `00 00 00 00 00 00`.
    *   `0x10`: `01` Segment count or flag.
*   **Endianness:** Little-Endian for Zstd headers.

## 4. Identified Internal Structures

### 4.1. Project Metadata (V1)
Located at `files/<PROJECT_UUID>.json`. Defines project name, schema version (`version`), and enabled resources (`features`).

### 4.2. Asset Storage (V1)
Located at `objects/`. Files are named via UUIDs. Frame thumbnails are stored here as PNGs.

### 4.3. Clojure Serialization (V2)
Decompressed Zstd data uses a serialization based on **Transit-MP** or Nigiri. Recurring patterns include:
*   `clj/keyword`: Lisp identifiers.
*   `java/instant`: timestamps.
*   `clj/vector`: data arrays.

## 5. Endianness
*   **ZIP Level:** Little-Endian.
*   **Zstd Level:** Little-Endian.
*   **Binary Blobs:** Byte order varies by asset type (PNG is Big-Endian).

## 6. Compression
*   **V1:** Standard ZIP Deflate.
*   **V2:** **Zstandard (zstd)**. Compressed payload starts immediately after the 17-byte header.

## 7. Image Data
Penpot is not a pure raster format. It contains references to external images (JPG/PNG) or vector geometry described in JSON.
*   **Reconstruction:** Requires a Penpot rendering engine to compose JSON into SVG/Canvas.

## 8. Embedded Thumbnail / Preview
*   **Is there a preview?** Yes, for frames and pages.
*   **V1 Strategy:**
    1.  Read `files/<UUID>/thumbnails/frame/<PAGE_UUID>/<FRAME_UUID>.json`.
    2.  Extract `mediaId` field.
    3.  Locate `objects/<mediaId>.png`.
*   **V2 Strategy:** Requires parsing the Transit stream to locate binary blobs. Direct extraction is complex without the serialization schema.

## 9. Metadata
UTF-8 readable strings within JSONs:
*   `name`: Filename.
*   `modifiedAt`: Modification date.
*   `projectId`: Unique ID.

## 10. Structural Reverse Engineering
The format is a snapshot of Penpot's database state (PostgreSQL/Datomic), serialized as individual JSON documents (V1) or a continuous binary stream (V2).

## 11. Strategy for Parser Implementation
1.  **Detection:** Check if file starts with `PK` or the proprietary header.
2.  **V1 (ZIP):**
    *   Use a ZIP stream to list `files/*.json`.
    *   Locate the root UUID JSON.
3.  **V2 (Zstd):**
    *   Skip 17 bytes.
    *   Decompress via `zstd`.
    *   Process as JSON/Nigiri (depending on version).

## 12. Parser Pseudocode
```pseudo
open file
magic = read(4)

if magic == "PK\x03\x04":
    # Version 1 (ZIP)
    zip = open_as_zip(file)
    root_json_path = find_in_zip("files/*.json" - not nested)
    root_meta = parse_json(zip.read(root_json_path))
    return root_meta

else if magic == "\x01\x0b\x1a\x86":
    # Version 2 (Zstd)
    skip(17)
    decompressed = zstd_decompress(rest_of_file)
    # Transit-Clojure parsing
    data = transit_decode(decompressed)
    return data.metadata
```

## 13. Strategy for Thumbnail Generation
*   **V1:** Prioritize extraction of `objects/*.png` referenced in `thumbnails/`. If multiple, use the largest or the first frame of the first page.
*   **V2:** Fallback to rendering or metadata extraction if binary thumbnail extraction fails.

## 14. Strategy for Basic Visualization
Display the SVG/PNG thumbnail extracted from objects. Interactive visualization requires the Penpot engine (Web-based).

## 15. Comparative Map Between Files
| File | Structure | Style | Observations |
| ------- | --------- | ------ | ----------- |
| `Eisenhower Matrix.penpot` | ZIP (V1) | Multidocument | Classic export structure. |
| `Cartas Creativas.penpot` | Zstd (V2) | Single Stream | Optimized/modern export. |
| `Material Design 3.penpot` | ZIP (V1) | Complex | Contains hundreds of objects and pages. |

## 16. Uncertain Points
*   **Nigiri Format (Confidence: 70%):** V2 serialization is proprietary but based on Transit principles. Exact binary blob extraction may require knowledge of Penpot's type tags.
*   **Font Embedding (Confidence: 60%):** Font embedding was not observed in samples (only references to Google or system fonts).

## 17. Technical Conclusion
.penpot is a high-fidelity container format. Transiting to **Zstd** (V2) indicates an optimization for performance in large UI kit exports. For cataloging, compatibility with the ZIP version should be the primary focus, given its wide presence in community repositories.
