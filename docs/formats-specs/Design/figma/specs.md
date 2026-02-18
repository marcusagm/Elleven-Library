# Technical Specification: Figma Project File (.fig)

## 1. Format Overview
*   **Extension Name:** `.fig`
*   **Possible Origin:** [Figma](https://www.figma.com/), a collaborative interface design tool.
*   **Category:** Design Document (Container).
*   **Magic Signature (Hexadecimal):**
    *   **V2 (Modern/Exported):** `50 4B 03 04` (ZIP Container).
    *   **Internal Blob:** `66 69 67 2d 6B 69 77 69` (String: `fig-kiwi`).
*   **Typical Size Observed:** 100 KB to 50+ MB depending on embedded image assets.
*   **Variations Between Analyzed Files:** Most exported files are ZIP archives containing a `canvas.fig` file. Standalone `canvas.fig` files may exist in internal caches or specific version control exports.

## 2. Global Binary Structure (ZIP Container)

The standard `.fig` file is a ZIP archive containing specific structured files.

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **ZIP Magic** | `0x04034B50` | Standard ZIP local header signature. |
| `Var` | Var | File | **meta.json** | Metadata in JSON format. | Details about export time, BG color, and name. |
| `Var` | Var | File | **thumbnail.png** | High-level preview image. | Usually 400px maximum dimension. |
| `Var` | Var | File | **canvas.fig** | Main design data. | Binary blob in Figma's custom Kiwi format. |
| `Var` | Var | Folder | **images/** | Image asset storage. | Contains PNG/JPG files named with their SHA-1 hash. |

## 3. Main Header (canvas.fig)

The `canvas.fig` file (and standalone `.fig` blobs) follows a custom binary serialization format known as "Kiwi".

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 8 bytes | `string` | **Signature** | `fig-kiwi` | Fixed magic signature. |
| `0x08` | 4 bytes | `u32 LE` | **Schema Version** | Internal format iteration. | Examples: `48`, `70`, `101`. |
| `0x0C` | 4 bytes | `u32 LE` | **Data Length** | Size of the primary payload. | Used to jump to secondary streams or metadata. |
| `0x10` | Var | `Blob` | **Payload** | Kiwi-encoded design tree. | Binary serialization of nodes, layers, and properties. |

## 4. Identified Internal Structures

### 4.1. meta.json
A flat JSON structure containing:
*   `file_name`: Original name of the project.
*   `exported_at`: Timestamp (ISO 8601).
*   `client_meta`: Contains `thumbnail_size` and `background_color`.

### 4.2. images/ folder
Images inside the ZIP are referenced by their SHA-1 hash string within the `canvas.fig` binary data. This deduplicates identical images across components.

## 5. Endianness
*   **ZIP Headers:** Little-Endian (LE).
*   **Kiwi Headers:** Little-Endian (LE) for length and version fields.

## 6. Compression
*   **ZIP Level:** Standard Deflate. Note that `canvas.fig` and images are often stored without additional ZIP compression (Compression Method: Store) because they are already compressed internally.
*   **Internal:** Kiwi data payloads can contain **Zlib/Deflate** chunks (`78 9C`) or **Zstandard** streams.

## 7. Image Data
Vector data is stored as nodes in the Kiwi tree. Raster images are stored as independent files in the `images/` folder within the ZIP.

## 8. Embedded Thumbnail / Preview
*   **Is there a preview?** Yes, via the **ZIP entry** `thumbnail.png`.
*   **Format:** PNG.
*   **Extraction:** Standard ZIP extraction of `thumbnail.png`.
*   **Automatic Detection:** Check for ZIP magic, list contents to find `thumbnail.png`.

## 9. Metadata
Metadata is exclusively stored in `meta.json`. Custom properties or plugin data are likely embedded within the `canvas.fig` Kiwi stream.

## 10. Structural Reverse Engineering
Figma uses a schema-based binary format (Kiwi) similar to Protocol Buffers but optimized for designer workflows.
*   **Pointer System:** Nodes reference each other via GUIDs.
*   **Asset Management:** Referenced via `images/` folder hashes.

## 11. Strategy for Parser Implementation
1.  **Validate ZIP:** Check for `PK\x03\x04`.
2.  **Extract meta.json:** Fast parse for filename and BG color.
3.  **Extract thumbnail.png:** Fast preview generation.
4.  **Parse canvas.fig (Optional):** Requires a Kiwi schema to walk the node tree.

## 12. Parser Pseudocode
```pseudo
open file
magic = read(4)
if magic == "PK\x03\x04":
    # Exported container
    zip = open_zip(file)
    metadata = parse_json(zip.read("meta.json"))
    thumbnail = zip.read("thumbnail.png")
    image_list = zip.list_directory("images/")
    return { metadata, thumbnail, image_list }
else if magic == "fig-kiwi":
    # Internal component
    version = read_u32_le()
    length = read_u32_le()
    payload = read(length)
    return { version, payload }
```

## 13. Strategy for Thumbnail Generation
The **best and fastest approach** is to extract the pre-rendered `thumbnail.png` from the ZIP container. This represents a high-fidelity snapshot of the main canvas.

## 14. Strategy for Basic Visualization
Standard image viewers should display the `thumbnail.png`. Full vector visualization requires a specialized renderer capable of interpreting the Kiwi binary scene graph and fetching assets from the `images/` directory.

## 15. Comparative Map Between Files
| File | Structure | Schema Version | Observations |
| :--- | :--- | :--- | :--- |
| `Apple.fig` | ZIP | 101 | Recent export, minimalist. |
| `Hotel booking app.fig` | ZIP | 101 | Complex design, 80+ image assets. |
| `example.canvas.fig` | Binary | 48 | Older internal blob, no container. |
| `Upload file ui kit.fig` | ZIP | 70 | Intermediate version. |

## 16. Uncertain Points
*   **Kiwi Schema Details (Confidence: 90%)**: The mapping between binary codes and design properties (like "Frame", "Vector") is proprietary and changes with schema versions.
*   **Secondary Streams (Confidence: 75%)**: Some standalone files contain extra bytes after the primary payload length, likely for caching or undo history.

## 17. Technical Conclusion
The `.fig` format is a **sophisticated ZIP-based document container**. Its internal `canvas.fig` uses a high-performance binary serialization (Kiwi) that allows Figma to handle massive design systems efficiently. For the purpose of indexing and previewing, the format is highly implementation-friendly due to the inclusion of a standard `thumbnail.png` and `meta.json`.
