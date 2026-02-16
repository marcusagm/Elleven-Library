
# Technical Analysis: Aseprite File Format (.aseprite / .ase)

## 1. Format Overview

*   **Extension Name:** `.aseprite` (preferred), `.ase` (legacy alternative).
*   **Origin:** [Aseprite](https://www.aseprite.org/) (David Capello). Open source specification.
*   **Category:** 2D Animation / Raster Graphics Editor Project File.
*   **Magic Signature:** `E0 A5` (at offset 4).
*   **Typical Size:** Few KB to several MB (highly dependent on frame count and resolution).
*   **Variations:** The format versioning is handled via fields in the header, but the structure (Header + Frames + Chunks) remains consistent across v1.0 - v1.3.

---

## 2. Global Binary Structure

The file is structured as a main header followed by a sequence of Frames.

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `DWORD` | **File Size** | Total size of the file in bytes. | |
| `0x04` | 2 bytes | `WORD` | **Magic Number** | `0xA5E0` | Critical validation. |
| `0x06` | 2 bytes | `WORD` | **Frames** | Total number of frames. | |
| `0x08` | 2 bytes | `WORD` | **Width** | Canvas width in pixels. | |
| `0x0A` | 2 bytes | `WORD` | **Height** | Canvas height in pixels. | |
| `0x0C` | 2 bytes | `WORD` | **Color Depth** | Bits per pixel (32, 16, or 8). | 32=RGBA, 16=Grayscale, 8=Indexed. |
| `0x0E` | 4 bytes | `DWORD` | **Flags** | Bitmask (Layer opacity valid = 1). | |
| `0x12` | 2 bytes | `WORD` | **Speed** | DEPRECATED (Frame duration). | Use Frame Header duration instead. |
| `0x14` | 4 bytes | `DWORD` | **0** | Reserved / Set to 0. | |
| `0x18` | 4 bytes | `DWORD` | **0** | Reserved / Set to 0. | |
| `0x1C` | 1 byte | `BYTE` | **Palette Entry** | Index of transparent color (8-bit only). | |
| `0x1D` | 3 bytes | `BYTE[]` | **Ignore** | Ignored bytes. | |
| `0x20` | 2 bytes | `WORD` | **Number of Colors** | Number of colors (0 means 256 for old files). | |
| `0x22` | 1 byte | `BYTE` | **Pixel Width** | Pixel aspect ratio width. | 0 means 1. |
| `0x23` | 1 byte | `BYTE` | **Pixel Height** | Pixel aspect ratio height. | 0 means 1. |
| `0x24` | 2 bytes | `SHORT` | **Grid X** | Grid position X. | |
| `0x26` | 2 bytes | `SHORT` | **Grid Y** | Grid position Y. | |
| `0x28` | 2 bytes | `WORD` | **Grid Width** | Grid width. | 16 pixels by default. |
| `0x2A` | 2 bytes | `WORD` | **Grid Height** | Grid height. | 16 pixels by default. |
| `0x2C` | 84 bytes | `BP` | **Reserved** | Reserved data (set to 0). | Bytes 44 to 127. |

The main header is strictly **128 bytes** long.

---

## 3. Main Header

The header contains global properties of the sprite.

*   **Endianness:** Little-endian (`<`).
*   **Magic Number:** `0xA5E0`.
*   **Color Depths Identifiable:**
    *   32 bpp: RGBA (Red, Green, Blue, Alpha).
    *   16 bpp: Grayscale (Value, Alpha).
    *   8 bpp: Indexed (Palette index).
*   **Flags:**
    *   `0x01`: Layer opacity has valid value.

---

## 4. Identified Internal Structures

After the 128-byte header, the file consists of a sequence of **Frames**.

### 4.1. Frame Header (16 bytes)

Repeats `Header.Frames` times.

| Relative Offset | Size | Type | Name | Function |
| :--- | :--- | :--- | :--- | :--- |
| `+0x00` | 4 bytes | `DWORD` | **Bytes in Frame** | Total size of this frame (Header + Chunks). |
| `+0x04` | 2 bytes | `WORD` | **Magic Number** | `0xF1FA`. |
| `+0x06` | 2 bytes | `WORD` | **Old Chunks** | Old chunk count (if new field is 0). |
| `+0x08` | 2 bytes | `WORD` | **Frame Duration** | Duration in milliseconds. |
| `+0x0A` | 2 bytes | `BYTE[]` | **Reserved** | Padding. |
| `+0x0C` | 4 bytes | `DWORD` | **New Chunks** | Number of chunks in this frame. |

### 4.2. Chunks

Inside each frame, there are N chunks. Each chunk has a strict header:

| Relative Offset | Size | Type | Name |
| :--- | :--- | :--- | :--- |
| `+0x00` | 4 bytes | `DWORD` | **Chunk Size** |
| `+0x04` | 2 bytes | `WORD` | **Chunk Type** |
| `+0x06` | Variable | - | **Chunk Data** |

#### Common Chunk Types Identified:

*   **0x2007 (Color Profile):** Defines sRGB or ICC profile.
*   **0x2019 (Palette):** Defines the color palette (entries).
*   **0x2004 (Layer):** Defines layer properties (Name, type, blend mode).
*   **0x2005 (Cel):** Defines the image content (pixels) for a specific layer at this frame.

---

## 5. Endianness

*   **Little-endian** is used for all integer fields (WORD, DWORD, SHORT).
*   **Evidence:**
    *   File size matches `ls -l` output when interpreted as Little-endian.
    *   Magic `E0 A5` corresponds to `0xA5E0`.
    *   Frame magic `FA F1` corresponds to `0xF1FA`.

---

## 6. Compression

*   **Usage:** Primarily in **Cel Chunks (0x2005, Type 2)**.
*   **Algorithm:** **ZLIB** (Deflate).
*   **Evidence:**
    *   Raw pixel data is usually compressed.
    *   Zlib headers `78 9C` (Default Compression) are present at the start of the compressed data stream within type 2 cels.
*   **Decompression:**
    *   Standard `zlib.decompress` or `inflate` handles the payload perfectly.

---

## 7. Image Data (If Existing)

Image data is fragmented into **Cels**. A specific frame is composed by stacking Cels.

*   **Location:** Inside Chunk `0x2005` (Cel Chunk).
*   **Structure of Cel Chunk (Type 2 - Compressed Image):**
    1.  **Layer Index:** 2 bytes (`WORD`).
    2.  **X Position:** 2 bytes (`SHORT`).
    3.  **Y Position:** 2 bytes (`SHORT`).
    4.  **Opacity:** 1 byte (`BYTE`).
    5.  **Cel Type:** 2 bytes (`WORD`). Value `2` = Compressed Image.
    6.  **Reserved:** 7 bytes.
    7.  **Width:** 2 bytes (`WORD`).
    8.  **Height:** 2 bytes (`WORD`).
    9.  **Pixel Data:** Remaining bytes (Zlib Compressed Stream).
*   **Pixel Packing (Decompressed):**
    *   **RGBA (32 bpp):** 4 bytes per pixel (Red, Green, Blue, Alpha).
    *   **Grayscale (16 bpp):** 2 bytes per pixel (Value, Alpha).
    *   **Indexed (8 bpp):** 1 byte per pixel (Index into Palette).
*   **Reconstruction:**
    *   Decompress Zlib stream.
    *   Read row-by-row (Row stride = Width * BytesPerPixel).
    *   Place at (X, Y) on the canvas.

---

## 8. Thumbnail / Embedded Preview

*   **Does it exist?** **NO**. There represents a "Project File", not a final render.
*   **Format:** N/A.
*   **Extraction:** Impossible directly.
*   **Generation Strategy:**
    1.  Parse Frame 0.
    2.  Identify all active Cels (Link them to Layers).
    3.  Decode Zlib pixel data for each Cel.
    4.  Composite Cels onto a blank canvas (using Painter's Algorithm + Blend Modes) based on Layer order.

---

## 9. Metadata

*   **Layer Names:** Found in Chunk `0x2004` (Layer Chunk). String is prefixed with `WORD` length.
*   **Tags/Frame Tags:** Chunk `0x2018`. Defines animation tags (e.g., "Run", "Jump") with start/end frames.
*   **User Data:** Chunk `0x2020`. Can contain custom text or color associated with the preceding chunk.
*   **Slice:** Chunk `0x2022`. Defines 9-patches or pivot points.

---

## 10. Structural Reverse Engineering

*   **Pattern:** Container of generic Chunks (TLV - Type-Length-Value style).
*   **Heuristic for robust parsing:**
    *   Read Frame Header -> Get `NewChunks` count.
    *   Loop `NewChunks` times.
    *   Read Chunk Size.
    *   If Chunk Type is unknown, `seek(Chunk Size - 6)` to skip.
    *   This ensures forward compatibility.

---

## 11. Strategy for Parser Implementation

1.  **Read Header (128 bytes):** Verify Magic `0xA5E0`. Extract `Width`, `Height`, `ColorDepth`.
2.  **Iterate Frames:**
    *   Read Frame Header (16 bytes).
    *   Verify Frame Magic `0xF1FA`.
    *   Read `Chunk Count`.
3.  **Iterate Chunks:**
    *   Switch on `Chunk Type`.
    *   **Crucial:** Parse `0x2019` (Palette) to build color lookup table for 8-bit mode.
    *   **Crucial:** Parse `0x2004` (Layer) to map Layer Index to Blending Mode/Opacity.
    *   **Crucial:** Parse `0x2005` (Cel) to get image data.
4.  **Buffer Handling:**
    *   Keep a "Frame Buffer" initialized to transparent.
    *   Decompress each Cel.
    *   Blit Cel pixels onto Frame Buffer at `(Cel.X, Cel.Y)`.

---

## 12. Parser Pseudocode

```python
def parse_aseprite(filepath):
    f = open(filepath, 'rb')
    
    # 1. Header
    file_size, magic, frames, width, height, depth, flags = unpack('<IHHHHHI', f.read(20))
    if magic != 0xA5E0: raise InvalidFormat()
    f.seek(128) # Skip to first frame
    
    layers = []
    palette = default_palette()
    
    # 2. First Frame (Thumbnail)
    frame_header = unpack('<IHHH2sI', f.read(16))
    chunk_count = frame_header[5]
    
    canvas = Image.new(mode, (width, height))
    
    for _ in range(chunk_count):
        chunk_size, chunk_type = unpack('<IH', f.read(6))
        chunk_start = f.tell() - 6
        
        if chunk_type == 0x2004: # Layer
            layers.append(parse_layer(f))
        
        elif chunk_type == 0x2019: # Palette
            palette = parse_palette(f)
            
        elif chunk_type == 0x2005: # Cel
            layer_index, x, y, opacity, type = unpack('<HhhBH', f.read(9))
            f.seek(7, 1) # Skip reserved
            
            if type == 2: # Compressed Image
                cel_w, cel_h = unpack('<HH', f.read(4))
                zlib_data = f.read(chunk_size - (f.tell() - chunk_start))
                raw_pixels = zlib.decompress(zlib_data)
                
                # Composite
                cel_image = Image.frombytes(raw_pixels, (cel_w, cel_h))
                blend_mode = layers[layer_index].blend_mode
                canvas.paste(cel_image, (x, y), blend_mode)
        
        f.seek(chunk_start + chunk_size) # Ensure alignment
        
    return canvas
```

---

## 13. Strategy for Thumbnail Generation

Since there is no embedded preview, **rendering is mandatory**.

*   **Performance:**
    *   Only Frame 0 needs to be parsed.
    *   Zlib decompression is fast.
    *   Compositing can be done in software (slow for large images) or GPU.
*   **Complexity:** Medium. Requires implementing:
    *   Zlib interactions.
    *   Basic blending modes (Normal is sufficient for 90% of thumbnails, but Multiply/Screen/Overlay might be needed for accuracy).
    *   Palette handling (for 8-bit files).

---

## 15. Comparative Map Between Files

| Analyzed File | Frames | Depth | Special Chunks | Observations |
| :--- | :--- | :--- | :--- | :--- |
| `_skeleton.aseprite` | 4 | 32 bpp | Profile, Palette, Layers, Cels | Standard animation. |
| `urban ninja.aseprite` | 44 | 32 bpp | Profile, Palette, Layers, Cels, Tags | Larger animation with tags ("Idle", "Attack"). |
| `c1.aseprite` | N/A | 32 bpp | - | Minimal valid file. |

---

## 16. Uncertain Points

1.  **Complex Blend Modes:** The exact math for some Aseprite blend modes (like "Addition", "Divide") might differ slightly from standard SVG/PDF blend modes.
2.  **Tileset (0x2023):** Not observed in the sample files, but exists in the spec for Tilemaps. Parsing logic would need to be extended to support tilemaps if encountered.
3.  **Color Space:** While 0x2007 defines profiles, many files might lack it and assume sRGB.

---

## 17. Technical Conclusion

The `.aseprite` format is **well-engineered, predictable, and robust**.

*   **Parsing Difficulty:** Low/Medium.
*   **Risks:** Low. The structure allows skipping unknown chunks gracefully.
*   **Dependence:** Requires `zlib`.
*   **Documentation:** High presence of implied knowledge (blend modes, stack order), but structure is strictly defined.

Implementation of a thumbnailer is feasible and reliable by rendering Frame 0.
