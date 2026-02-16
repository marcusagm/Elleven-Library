
# Reverse Engineering Analysis: GIMP XCF Format

## 1. Technical Overview

*   **Format Name:** GIMP XCF (eXperimental Computing Facility).
*   **Origin:** GIMP (GNU Image Manipulation Program). Native project format.
*   **Category:** Layered Raster Image Editor Project.
*   **Magic Signature:** `gimp xcf ` followed by version string (e.g., `v001`, `v011`).
*   **Typical Size:** Large (Uncompressed or RLE/ZLIB), typically 10MB - 500MB+.
*   **Complexity:** Very High. Full DOM dump of the editor state.
*   **Documentation:** Implicit (Source code is the documentation), but version evolution makes parsing complex (32-bit vs 64-bit offsets).

---

## 2. Structural Hex Map

A high-level map of the file layout based on parsing analysis:

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 9 bytes | `ASCII` | **Magic** | `gimp xcf ` |
| `0x09` | 4 bytes | `ASCII` | **Version** | `v001`, `v002`, ... `v011` ... |
| `0x0D` | 1 byte | `0x00` | **Null** | Null terminator. |
| `0x0E` | 4 bytes | `UINT32` | **Width** | Canvas width (Big Endian). |
| `0x12` | 4 bytes | `UINT32` | **Height** | Canvas height (Big Endian). |
| `0x16` | 4 bytes | `UINT32` | **Base Type** | 0=RGB, 1=Grayscale, 2=Indexed. |
| `0x1A` | Variable | - | **Properties** | List of generic properties (TLV). |
| *Var* | 4/8 bytes | `PTR` | **Layer Ptr** | Offset to Layer Index List. |
| *Var* | 4/8 bytes | `PTR` | **Channel Ptr** | Offset to Channel Index List. |
| ... | ... | ... | **Layer Data** | Structures scattered throughout file. |

**Crucial Note on Offsets (`PTR`):**
*   **Versions < 011:** Pointers are **4 bytes** (32-bit).
*   **Versions >= 011:** Pointers are **8 bytes** (64-bit). This is a rigid breaking change.

---

## 3. Entropy Segmentation

*   **Header & Properties:** Low entropy. Structured integers and ASCII strings.
*   **Layer Headers:** Low entropy.
*   **Tile Offsets:** High density (tables of pointers).
*   **Pixel Data (Tiles):** High entropy.
    *   **RLE (Legacy):** Medium-High entropy.
    *   **ZLIB (Modern):** High entropy (Deflate stream signatures `78 9C` may appear inside tile blocks).

---

## 4. Header Analysis (Detailed)

All integers are **Big-Endian** (`>I`, `>Q`).

1.  **Magic:** `67 69 6D 70 20 78 63 66 20` ("gimp xcf ").
2.  **Version:** `76 30 31 31` ("v011"). The numeric part `011` determines parsing logic.
3.  **Canvas Size:** Width (`00 00 07 80` -> 1920) and Height (`00 00 04 38` -> 1080).
4.  **Base Mode:** `0` (RGB).

---

## 5. Internal Structures

### 5.1. Property List (TLV)

Found in Main Header, Layer Headers, and Channel Headers.

| Field | Size | Type | Description |
| :--- | :--- | :--- | :--- |
| **Type** | 4 bytes | `UINT32` | Property ID (e.g., 21 = PARASITES). |
| **Length** | 4 bytes | `UINT32` | Payload length in bytes. |
| **Value** | `Length` | - | Data. |

**Terminator:** Type `0`.

**Common Properties:**
*   `PROP_COMPRESSION (17)`: 1 byte value. 0=None, 1=RLE, 2=ZLIB, 3=Fractal.
*   `PROP_RESOLUTION (19)`: X/Y resolution floats.
*   `PROP_OPACITY (6)`: 0-255 layer opacity.
*   `PROP_OFFSETS (15)`: **Critical**. Contains pointers to Hierarchy, Mask, etc. Size depends on version (4 vs 8 bytes).

### 5.2. Layer Definition

Pointed to by the "Layer Index Pointer".

1.  **Layer List:** Sequence of `PTR` (offsets to layers), terminated by `0`.
2.  **Layer Structure:**
    *   **Width:** 4 bytes.
    *   **Height:** 4 bytes.
    *   **Type:** 4 bytes (e.g., 0=RGB, 1=RGBA, 2=Gray, 3=GrayA...).
    *   **Name:** Pascal-style string (UInt32 Length + Bytes + Null).
    *   **Properties:** TLV list.
    *   **Hierarchy Pointer:** Found in `PROP_OFFSETS` (Type 15).

### 5.3. Hierarchy & Tiles

The image data is not a contiguous RAW block but a **Tile Hierarchy**.

*   **Hierarchy:** Contains a list of **Levels** (Mipmaps). Level 0 is full res.
*   **Level:** Contains a list of **Tiles** (typically 64x64 pixels).
*   **Tile:**
    *   Usually compressed (RLE default).
    *   In v11+, can be ZLIB.
    *   Coordinates are implicit based on tile index (Row-major: 0,0 -> 1,0 ...).

---

## 6. RAW Sensor Data Region

**Does not exist.**
XCF is a "baked" project format. There is no Bayer pattern or Camera RAW data. The pixel data is "Cooked" (De-mosaiced RGB or Grayscale), split into tiles, and compressed.

*   **Pixel Packing:**
    *   **RGB:** 3 bytes (R, G, B) or 4 bytes (R, G, B, A).
    *   **Precision:** Standard XCF is 8-bit per channel. Version 11+ supports high bit-depth (16-bit, 32-bit float) via `GEGL` extensions, which changes the Tile encoding significantly.

---

## 7. Theoretical Image Reconstruction

To render the image, one must replicate the GIMP compositing engine:

1.  **Traverse Layers:** Bottom-up order.
2.  **For Each Layer:**
    *   Locate Level 0 in Hierarchy.
    *   Iterate Tiles.
    *   **Decompress:** RLE algorithm (standard GIMP RLE) or ZLIB.
    *   **Place:** Blit 64x64 chunks into a layer buffer.
3.  **Composite:** Apply Layer Mode (Normal, Multiply, etc.) and Opacity to merge layer buffer onto canvas.

**Complexity:** Extremely High for a parser.

---

## 8. Forensic Parser Pseudocode

```python
def parse_xcf(file):
    magic = file.read(9)
    version = int(file.read(4)[1:])
    ptr_size = 8 if version >= 11 else 4
    
    width, height = read_u32(), read_u32()
    
    # Skip global props
    read_props()
    
    # Get Layer List
    layer_ptr_offset = read_ptr(ptr_size)
    file.seek(layer_ptr_offset)
    
    layers = []
    while True:
        off = read_ptr(ptr_size)
        if off == 0: break
        layers.append(parse_layer(off, ptr_size))
        
    return layers

def parse_layer(offset, ptr_size):
    file.seek(offset)
    w, h, type = read_u32(), read_u32(), read_u32()
    name = read_string()
    
    props = read_props()
    # Extract Hierarchy PTR from PROP_OFFSETS (Type 15)
    hierarchy_ptr = props[15].data
    
    return Layer(name, w, h, hierarchy_ptr)
```

---

## 9. Strategy for Thumbnail Generation

XCF files **rarely contain a dedicated full-file embedded preview** (unlike RAW or TIFF).

*   **Common Scenarios:**
    1.  **No Thumbnail:** Standard case.
    2.  **External Thumbnail:** Linux desktop environments often save `~/.thumbnails/....png` separately.
    3.  **Parsite Thumbnail:** Some files may contain a `gimp-thumbnail-pixbuf` parasite (Property 21), but it's small/unreliable.
    
*   **Recommended Strategy:**
    *   **Partial Parsing:** Parse ONLY the **Top-Visible Layer** (if it covers the canvas) or a "Background" layer.
    *   **Full Render (Fallback):** Necessary for 99% accuracy. Parse generic layers, decompress RLE tiles, compose.
    *   **Optimization:** Render only the "Composite" projection if GIMP saved it? (No standard "Composite" stream exists; GIMP recalculates it).
    
    *Heuristic:* Many users flatten the image before saving or have a background layer. Rendering just the bottom-most unique opaque layer might yield a "good enough" preview for a file manager.

---

## 10. Comparative Table

| File | Version | Pointer Size | Base Type | Observations |
| :--- | :--- | :--- | :--- | :--- |
| `default_icon.xcf` | v011 | 8 bytes | 0 (RGB) | Small icon, likely 1 layer. |
| `gimp-splash.xcf` | v011 | 8 bytes | 0 (RGB) | Full HD, complex layer stack. |
| Legacy Files | v001-v003 | 4 bytes | Any | Older GIMP versions. |

---

## 11. Unidentified Fields

*   **Property Payloads:** Many property types (e.g., `PROP_PARASITES`) contain blobs of serialized GIMP internal structs (GimpParasite), often string-keyed dictionaries. Parsing them requires specific struct definitions.
*   **GEGL Data:** In v11+, highly precise data is stored in generic tile buffers that are managed by GEGL, potentially using different compression/encoding than the standard RLE.

---

## 12. Final Evaluation

*   **Implementation Difficulty:** **9/10**.
*   **Structural Robustness:** High (Pointer-based, allows random access).
*   **Production Recommendation:**
    *   **Do not write a custom parser** unless absolutely necessary.
    *   **Use `xcftools`** (CLI utility) or `libgimp` if possible to convert to PNG/JPG for thumbnails.
    *   If Rust/Native implementation is mandatory: Implement a reader for **v003 (32-bit)** and **v011 (64-bit)** that extracts the **first visible layer** for a quick preview, acknowledging it might be incomplete (missing layers).
