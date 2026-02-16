# Technical Analysis: Adobe Photoshop (.psd) Format

## 1. Format Overview

*   **Extension:** `.psd` (Photoshop Document).
*   **Software:** Adobe Photoshop.
*   **Category:** Layered Raster Image.
*   **Magic Signature:** `38 42 50 53` (`8BPS`).
*   **Endianness:** **Big-Endian** (Standard for Adobe formats).
*   **Structure:** Five major sections: File Header, Color Mode Data, Image Resources, Layer and Mask Information, and Image Data.

---

## 2. Global Binary Structure

| Section | Size | Description |
| :--- | :--- | :--- |
| **File Header** | 26 bytes | Basic dimensions, depth, color mode. |
| **Color Mode Data** | 4 + N bytes | Length + data (e.g. Indexed color table). |
| **Image Resources** | 4 + N bytes | Length + sequence of Image Resource Blocks. |
| **Layer and Mask Info** | 4 + N bytes | Length + individual layer data and global masks. |
| **Image Data** | 2 + N bytes | Compression code + merged image pixels. |

---

## 3. Header Principal (26 bytes)

| Offset | Size | Type | Name | Value / Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `ASCII` | **Signature** | `8BPS` |
| `0x04` | 2 bytes | `u16` | **Version** | `1` (PSD) or `2` (PSB). |
| `0x06` | 6 bytes | - | **Reserved** | Must be zero. |
| `0x0C` | 2 bytes | `u16` | **Channels** | Number of color channels (1 to 56). |
| `0x0E` | 4 bytes | `u32` | **Height** | Image height in pixels. |
| `0x12` | 4 bytes | `u32` | **Width** | Image width in pixels. |
| `0x16` | 2 bytes | `u16` | **Depth** | Bits per channel (1, 8, 16, 32). |
| `0x18` | 2 bytes | `u16` | **ColorMode** | 0=Bitmap, 1=Grayscale, 2=Indexed, 3=RGB, 4=CMYK, 7=Multichannel, 8=Duotone, 9=Lab. |

---

## 4. Image Resources (Embedded Previews)

Photoshop stores previews, thumbnails, and metadata (EXIF, XMP, IPTC) inside **Image Resource Blocks**.

### 4.1. Image Resource Block Structure

| Size | Type | Name | Description |
| :--- | :--- | :--- | :--- |
| 4 bytes | `ASCII` | **Signature** | `8BIM` (most common) or `MeSa`. |
| 2 bytes | `u16` | **ID** | Unique ID for the resource type. |
| Var | `Pascal String` | **Name** | Null-padded to even length. |
| 4 bytes | `u32` | **Size** | Size of the resource data. |
| `Size` | - | **Data** | Resource payload (padded to even). |

### 4.2. Thumbnail Resource (ID: 1033 or 1036)

*   **ID 1033 (0x0409):** Thumbnail resource for Photoshop 4.0.
*   **ID 1036 (0x040C):** Thumbnail resource for Photoshop 5.0 and later (Standard).

**Thumbnail Data Header:**
| Offset | Size | Type | Description |
| :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Format**: `1` = kJpegRGB, `0` = kRawRGB. |
| `0x04` | 4 bytes | `u32` | **Width** (pixels). |
| `0x08` | 4 bytes | `u32` | **Height** (pixels). |
| `0x0C` | 4 bytes | `u32` | **WidthBytes**: Padded row length. |
| `0x10` | 4 bytes | `u32` | **TotalSize**: Size of data + header. |
| `0x14` | 4 bytes | `u32` | **CompressedSize**: Size after compression. |
| `0x18` | 2 bytes | `u16` | **BitsPerPixel**: Usually 24. |
| `0x1A` | 2 bytes | `u16` | **Planes**: Usually 1. |
| `0x1C` | `Var` | - | **JPEG Stream**: If Format=1, standard JPEG data starts here. |

---

## 5. Image Data Section

Located at the end of the file. Contains the fully merged (flattened) image preview.

*   **Compression Types:**
    *   `0`: Raw data.
    *   `1`: RLE (PackBits).
    *   `2`: Zip without prediction.
    *   `3`: Zip with prediction.

---

## 6. Thumbnail Extraction Strategy

### Algorithm:
1.  Verify `8BPS` magic.
2.  Skip Header (26 bytes).
3.  Read Color Mode Data length and skip.
4.  Iterate through Image Resource Blocks:
    *   Find block with signature `8BIM` and ID `1036`.
    *   Read the thumbnail data header.
    *   If Format is `1`, extract the JPEG stream directly starting at offset 28 from the resource data start.
5.  If no thumbnail resource is found, the merged image at the end of the file can be used as a fallback (requires RLE/Zip decoding).

---

## 7. Pseudocode

```python
def get_psd_thumbnail(f):
    f.seek(0)
    if f.read(4) != b'8BPS': return None
    f.seek(26)
    
    # Skip Color Mode
    cmd_len = read_u32_be(f)
    f.seek(cmd_len, 1)
    
    # Parse Resources
    res_len = read_u32_be(f)
    end_res = f.tell() + res_len
    
    while f.tell() < end_res:
        sig = f.read(4)
        rid = read_u16_be(f)
        name = read_pascal_string(f) # Pascal string, even padded
        size = read_u32_be(f)
        
        if sig == b'8BIM' and rid == 1036:
            f.seek(28, 1) # Skip thumb header (id, dim, etc)
            return f.read(size - 28) # JPEG data
        
        f.seek(size + (size % 2), 1) # Skip and align
```

---

## 8. Uncertainties

*   **Resource Alignment:** While most documentation states 2-byte alignment for resource blocks, some non-Adobe writers might use 4-byte alignment, causing parsing shifts.
*   **Multiple Previews:** Files can sometimes contain both ID 1033 and 1036. 1036 should always be prioritized as it is higher quality.
*   **Composite Fallback:** If the file was saved without a thumbnail (saving option), the merged image at the end is the only way to get a preview, but decoding it for large files is computationally expensive due to RLE/Zip.
