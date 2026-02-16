# Technical analysis of MediBang Paint (.mdp) Format

## 1. Format Overview

*   **Extension:** `.mdp`
*   **Software:** MediBang Paint, FireAlpaca.
*   **Category:** Layered Raster Image Project.
*   **Signature:** `mdipack` (followed by null or version).
*   **Structure:** Custom binary container with an embedded XML header and a sequence of named binary blocks.
*   **Endianness:** **Little-Endian** for all integer fields.
*   **Compression:** Heavy use of **Zlib**.

---

## 2. Global Binary Structure

The file consists of a fixed global header, a variable-length XML metadata block, and a sequence of data blocks (PAC Blocks).

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 8 bytes | `ASCII` | **Magic** | `mdipack\x00` |
| `0x08` | 4 bytes | `u32` | **Unknown 1** | Usually `0x00`. |
| `0x0C` | 4 bytes | `u32` | **XML Size** | Size of the XML metadata block in bytes. |
| `0x10` | 4 bytes | `u32` | **Binary Size** | Total size of the data blocks section. |
| `0x14` | `XML Size` | `UTF-8` | **Metadata** | Main project structure in XML format. |
| `...` | `Binary Size` | `Blocks` | **Data Section** | Sequence of `PAC` Blocks (Thumbnail, Layers). |

---

## 3. Metadata (XML)

Starting at offset `0x14` (20), the file contains a standard XML document describing the project.

**Key Elements:**
*   `<Mdiapp>`: Root element. Attributes for `width`, `height`, `dpi`.
*   `<Thumb>`: Attributes `width`, `height`, and `bin="thumb"`. Defines the thumbnail block name.
*   `<Layer>`: Attributes defining layer properties (`name`, `opacity`, `visible`, `type`).
    *   `bin`: The name of the binary block containing this layer's pixel data (e.g., `bin="layer0img"`).

---

## 4. Data Section: PAC Blocks

The Data Section is a continuous sequence of **PAC Blocks**. Each block follows a strict 132-byte header structure followed by the payload.

### 4.1. PAC Header (132 bytes)

| Offset (internal) | Size | Type | Field | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `ASCII` | **Magic** | `50 41 43 20` ("PAC "). |
| `0x04` | 4 bytes | `u32` | **Block Size** | Total size of the block (Header + Payload). |
| `0x08` | 4 bytes | `u32` | **Type?** | Usually `0x01` in observed files. |
| `0x0C` | 4 bytes | `u32` | **Payload Size** | Size of the actual data following the header (`Block Size - 132`). |
| `0x10` | 32 bytes | - | **Reserved** | Usually zeroes. |
| `0x30` | 64 bytes | `ASCII` | **Block Name** | Null-terminated string (e.g., "thumb", "layer0img"). Matches XML `bin` attribute. |
| `0x70` | 20 bytes | - | **Padding** | Padding to reach 132 bytes. |

### 4.2. Block Payload

The content of the payload depends on the block name.

---

## 5. Specific Block Formats

### 5.1. Thumbnail Block (`thumb`)

*   **Identified by:** Name "thumb" in header.
*   **Referenced by:** XML `<Thumb bin="thumb" />`.
*   **Payload Format:** **Zlib Compressed Raw Bitmap**.
*   **Extraction:**
    1.  Read payload (Offset +132).
    2.  Decompress using Zlib.
    3.  Result is raw **32-bit RGBA** (or ARGB/BGRA) pixel data.
    4.  Dimensions: Defined in XML `<Thumb>` attributes (e.g., 256x256).
    5.  Size Check: `Width * Height * 4` bytes.

### 5.2. Layer Image Block (`layerNimg`)

*   **Identified by:** Name "layerXimg" (referenced by `<Layer bin="...">`).
*   **Payload Format:** **Custom Header + Zlib Stream**.
*   **Internal Structure:**
    *   **Header (24 bytes):**
        *   `0x00` (4 bytes): **Tile Count** (u32).
        *   `0x04` (4 bytes): **Tile Size** (u32, usually 128).
        *   `0x08` (16 bytes): Padding/Unknown.
    *   **Data:** **Zlib Compressed Stream**.
        *   Decompressed stream contains the tiled pixel data.

---

## 6. Thumbnail Extraction Strategy

The `.mdp` format is highly optimized for quick thumbnail retrieval since the thumbnail is usually the **first block** in the data section.

### Algorithm:

1.  **Read Global Header (20 bytes):**
    *   Verify Magic `mdipack\x00`.
    *   Read `XML Size` at `0x0C`.
2.  **Read XML:**
    *   Parse dimensions from `<Thumb>` tag (`width`, `height`).
    *   Confirm thumbnail block name (usually "thumb").
3.  **Jump to Data Section:**
    *   Seek to `20 + XML Size`.
4.  **Parse First Block:**
    *   Verify `PAC ` magic.
    *   Check Name at relative offset `0x30`. If "thumb":
    *   Read `Block Size` at `0x04`.
    *   Read Payload starting at relative `0x84` (132).
    *   Payload Length = `Block Size - 132`.
5.  **Decompress & Render:**
    *   Zlib Decompress the payload.
    *   Interpret as Raw RGBA pixels.
    *   Encode to PNG/JPG.

---

## 7. Pseudocode Implementation

```python
def extract_mdp_thumbnail(filepath):
    with open(filepath, 'rb') as f:
        # Check Magic
        if f.read(8) != b'mdipack\x00': return None
        
        # Read XML Size
        f.seek(12)
        xml_size = struct.unpack('<I', f.read(4))[0]
        
        # Parse XML for dimensions
        f.seek(20)
        xml_root = ET.fromstring(f.read(xml_size))
        thumb_node = xml_root.find("Thumb")
        if thumb_node is None: return None
        
        t_w = int(thumb_node.attrib['width'])
        t_h = int(thumb_node.attrib['height'])
        
        # Seek to First Block (Data Section)
        f.seek(20 + xml_size)
        
        # Read PAC Header
        pac_magic = f.read(4) # b'PAC '
        block_size = struct.unpack('<I', f.read(4))[0]
        
        # Read Name
        f.seek(40, 1) # Skip to Name (Offset 48 from start of block)
        name = f.read(64).strip(b'\x00')
        
        if name != b'thumb':
            # Scan other blocks if necessary
            return None 
            
        # Read Payload
        f.seek(20 + xml_size + 132)
        payload = f.read(block_size - 132)
        
        # Decompress
        raw_pixels = zlib.decompress(payload)
        
        return create_image_from_rgba(raw_pixels, t_w, t_h)
```

---

## 8. Uncertainties

*   **Layer Data Tiling:** The Zlib stream in layer blocks likely decompresses to a proprietary serialized format of tiles. Reconstruction of full canvas from layers requires reverse engineering this inner stream format (likely trivial sequence of bitmaps).
*   **Color Profiles:** `ICC` profiles are likely stored in their own PAC blocks if present (e.g., named "icc"?), but were not observed in the small samples.
*   **Bit Depth:** 8-bit layers are confirmed. 1-bit or 16-bit support is managed via the `type` attribute in XML, potentially changing the raw pixel format in the Zlib payload.
