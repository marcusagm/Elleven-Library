# Technical Specification: Autodesk SketchBook TIFF (.tif)

## 1. Format Overview
*   **Extension Name:** `.tif` / `.tiff`
*   **Possible Origin:** Autodesk (formerly Alias systems).
*   **Category:** Multilayer Raster Image.
*   **Magic Signature (Hexadecimal):** `49 49 2A 00` (Little-Endian TIFF) or `4D 4D 00 2A` (Big-Endian TIFF).
*   **Typical Size:** 100 KB (sketches) to 50+ MB (complex art).
*   **Variations:** Used across Windows, macOS, iOS, and Android versions of SketchBook Pro.

## 2. Global Binary Structure

SketchBook uses a standard TIFF container where layers are stored as **SubIFDs** (Sub-Image File Directories). Standard image viewers see only the first (composite) image.

| Offset | Size | Type | Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 2 bytes | String | **Byte Order** | `II` (Little-Endian) or `MM` (Big-Endian). | Mostly `II` on modern systems. |
| `0x02` | 2 bytes | `u16` | **Magic** | Always `42` (`0x2A`). | Standard TIFF marker. |
| `0x04` | 4 bytes | `u32` | **1st IFD Offset**| Pointer to the main image directory. | Composite rendered image. |
| `Var` | Var | IFD | **Primary IFD** | Root directory of the file. | Contains software markers. |
| `Var` | Var | IFD | **SubIFDs** | Individual layer directories. | Referenced by Tag 330. |

## 3. Main Header
Follows the standard TIFF specification. The first IFD is the "Main" image, which SketchBook keeps as a flattened preview of all visible layers.

*   **Identifiable Field:** Tag `305` (Software) always contains `Alias MultiLayer TIFF V1.1` or similar.

## 4. Internal Structures (IFD Entries)

### 4.1. Primary IFD (Index 0)
Contains standard tags (`256` Width, `257` Height, `273` StripOffsets, etc.) and the key SketchBook marker:

| Tag | Name | Type | Description |
| :--- | :--- | :--- | :--- |
| `330` | **SubIFDs** | `u32[]` | Array of offsets to layer-specific IFDs. |
| `305` | **Software** | `string` | Software signature ("Alias MultiLayer TIFF..."). |
| `50790` | **App Vers** | `string` | SketchBook version and OS info. |

### 4.2. Layer IFDs (SubIFDs)
Each layer is stored as a separate TIFF IFD. If a layer is smaller than the canvas, its dimensions (`256`, `257`) and offsets (`286`, `287`) describe the active bounding box.

| Tag | Name | Type | Description |
| :--- | :--- | :--- | :--- |
| `285` | **PageName** | `string` | User-defined name of the layer. |
| `286` | **XPosition** | `rational` | Horizontal offset from canvas top-left. |
| `287` | **YPosition** | `rational` | Vertical offset from canvas top-left. |
| `50784`| **Metadata** | `string` | CSV string: `Opacity, BlendMode, Visibility, ...`|

## 5. Endianness
*   **Little-Endian (`II`)**: Standard for most desktop and mobile versions.
*   **Big-Endian (`MM`)**: Possible in older Macintosh-saved files.

## 6. Compression
*   **LZW (5)**: Most common for high-fidelity saves.
*   **PackBits (32773)**: Sometimes used for faster, less efficient compression.
*   **Uncompressed (1)**: Rarely used.

## 7. Image Data
*   **Encoding:** Strips (standard TIFF).
*   **Alpha Channel:** Stored via Tag `338` (ExtraSamples) set to 1 (Associated Alpha / Pre-multiplied) or 2 (Unassociated Alpha).

## 8. Embedded Thumbnail / Preview
*   **Exists:** **Yes**, implicitly.
*   **Source:** The **First IFD (Index 0)** is designed to be the high-resolution composite preview.
*   **Extraction:** Any standard TIFF loader that reads the first page (IFD 0) effectively extracts the document thumbnail/preview.

## 9. Metadata
Metadata is distributed across:
1.  **Tag 50790**: System environment (e.g., `V1_Windows_Sketchbook Pro_9.3.21`).
2.  **Tag 50784**: Per-layer properties.
    *   Example: `1.000, 00000000, 1, 0, 1, ...`
    *   Field 1: Opacity (0.0 to 1.0).
    *   Field 2: Blend Mode (as hexadecimal ID).
    *   Field 3: Visibility (bit 0/1).

## 10. Structural Reverse Engineering
*   **Recursive Structure:** The file is effectively a "stacked TIFF".
*   **Optimization:** SketchBook only saves pixels for the modified area of a layer, updating the `XPosition/YPosition` and `Width/Height` in the SubIFD to minimize file size.

## 11. Strategy for Parser Implementation
1.  **Validate TIFF:** Check for magic `49 49 2A 00`.
2.  **Check Signature:** Verify Tag 305 for "Alias MultiLayer TIFF".
3.  **Composite Extraction:** Extract IFD 0 for the preview.
4.  **Layer Enumeration:** Read Tag 330. Iterate through offsets to find all layers and their metadata.

## 12. Parser Pseudocode
```pseudo
open file
read TIFF_HEADER
if magic != 42: abort

first_ifd = read_ifd(header.offset)
if first_ifd.get_tag(305) contains "Alias":
    is_sketchbook = true

# Extract Preview
pixels = decode_strips(first_ifd)
save_as_png(pixels)

# Extract Layers
if first_ifd.has_tag(330):
    layer_offsets = first_ifd.get_tag_values(330)
    for offset in layer_offsets:
        layer_ifd = read_ifd(offset)
        name = layer_ifd.get_string(285)
        props = layer_ifd.get_string(50784).split(",")
        print("Layer:", name, "Opacity:", props[0])
```

## 13. Strategy for Thumbnail Generation
Fastest and most compatible: **Read IFD 0**. This avoids the need to blend individual layers and handle proprietary metadata (50784).

## 14. Strategy for Basic Visualization
Display the result of IFD 0. For layer inspection, render individual SubIFDs as cropped bitmaps positioned according to their `XPosition/YPosition` tags.

## 15. Comparative Map Between Files
| File | Layers | Software Version | Observations |
| :--- | :--- | :--- | :--- |
| `3_frames.tif` | 3 | Pro 9.3 | Simple layering. |
| `Wing2.tif` | 27 | Pro 9.3 | High complexity, mixed group/mask indicators. |

## 16. Uncertain Points
*   **Blend Mode IDs (Confidence 80%)**: Field 2 in Tag 50784 is likely a hex mapping to Photoshop-style blend modes (Normal, Multiply, etc.).
*   **Masking (Confidence 70%)**: SubIFDs without names (PageName) may represent clipping masks or selection channels.

## 17. Technical Conclusion
The SketchBook TIFF is a **highly conformant extension of the TIFF standard**. Its use of `SubIFDs` makes it robustly backward-compatible—any standard image viewer can see the final artwork, while SketchBook maintains full editing capabilities through standard but deep TIFF features.
