# Technical Analysis: Affinity Photo (.afphoto)

## 1. Format Overview

*   **Extension:** `.afphoto`
*   **Software:** Serif Affinity Photo.
*   **Family:** Affinity Document Format (shared with `.afdesign`, `.afpub`).
*   **Category:** Layered Raster/Vector Image Document / Container.
*   **Magic Signature:** `00 FF 4B 41` (Little-Endian: `0x414BFF00`).
*   **Endianness:** **Little-Endian**.
*   **Structure:** Header with fixed pointers to internal serialized blocks.
*   **Thumbnail:** Embedded **PNG**, accessible via header pointer.

---

## 2. Global Binary Structure

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `00 FF 4B 41` (`KA\xff\x00` LE). |
| `0x04` | 4 bytes | `u32` | **Version/Flags** | e.g. `0x0000000B`. |
| `0x08` | 8 bytes | `ASCII` | **Container ID** | `nsrP#Inf` (`Prsn#Inf` LE). Likely "Persona Info". |
| `0x10` | 8 bytes | `u64` | **Content Pointer** | Pointer to main content block. |
| `0x18` | 8 bytes | `u64` | **Thumbnail Pointer** | **Critical:** Absolute offset to the Thumbnail Block. |
| `0x20` | ... | ... | **Other Pointers** | Sequence of `u64` pointers to other sections. |

---

## 3. Thumbnail Block Structure

Located at the offset defined at `0x18` in the file header.

| Relative Offset | Size | Type | Value/Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `+00` | 4 bytes | `u32` | **Block Marker** | `FF FF FF FF` (`0xFFFFFFFF`). |
| `+04` | 4 bytes | `ASCII`| **Signature** | `Thmb` (`bmhT` LE). |
| `+08` | 4 bytes | `u32` | **Version** | Usually `1`. |
| `+12` | 4 bytes | `u32` | **Size 1** | Size of payload + extra header? |
| `+16` | 4 bytes | `u32` | **Header Len?** | Usually `0x1D` (29)? |
| `+20` | 4 bytes | `u32` | **Zero** | `00 00 00 00` |
| `+24` | 4 bytes | `u32` | **Size 2** | Size of PNG data? |
| `+28` | 1 byte | `u8` | **Flag?** | `0x01` seen before PNG. |
| `+29` | Var | `PNG` | **Image Data** | Standard PNG stream (`89 50 4E 47 ...`). |

---

## 4. Parser Strategy

### 4.1. Identification
1.  Read first 4 bytes.
2.  Check for `00 FF 4B 41`.

### 4.2. Thumbnail Extraction
1.  Seek to `0x18` (24).
2.  Read `u64` (Little Endian) -> `ThumbOffset`.
3.  Validate `ThumbOffset` < File Size.
4.  Seek to `ThumbOffset`.
5.  Verify integrity:
    *   Read 4 bytes -> expect `FF FF FF FF`.
    *   Read 4 bytes -> expect `Thmb`.
6.  Seek to `ThumbOffset + 29`.
7.  Verify PNG Signature (`89 50 4E 47`).
8.  Extraction:
    *   Read until `IEND` chunk (robust method).
    *   OR: Read `Size 2` from `ThumbOffset + 24` (matches PNG size usually).

---

## 5. Pseudocode

```python
def extract_affinity_photo_thumb(path):
    with open(path, 'rb') as f:
        # 1. Magic Check
        if f.read(4) != b'\x00\xff\x4b\x41': # Little Endian 41 4B FF 00
            return None
            
        # 2. Get Pointer
        f.seek(0x18)
        thumb_offset = struct.unpack('<Q', f.read(8))[0]
        
        # 3. Validation
        f.seek(thumb_offset)
        # Check for FFFFFFFF Thmb
        if f.read(8) != b'\xff\xff\xff\xffThmb':
            return None
            
        # 4. Get PNG Size
        f.seek(thumb_offset + 24)
        png_size = struct.unpack('<I', f.read(4))[0]
        
        # 5. Extract
        f.seek(thumb_offset + 29)
        png_data = f.read(png_size)
        
        if png_data.startswith(b'\x89PNG'):
            return png_data
            
    return None
```

---

## 6. Compression & Decompression

*   The thumbnail is a standard **PNG**, so it uses Deflate (Zlib) internally.
*   The overall container uses a proprietary structure where internal blocks may be compressed or serialized objects.

---

## 7. Comparison Between Samples

| File | Sub-Version | Thumb Offset | Size | Observation |
| :--- | :--- | :--- | :--- | :--- |
| `sample_640x426.afphoto`| 524299 | `0x1D264` | 38.2 KB | Normal preview. |
| `sample.afphoto` | 524299 | `0x2A155` | 41.5 KB | Small design project. |
| `DSC...afphoto` | 11 | `0x1A22C` | 155 KB | High-resolution photography. |

---

## 8. Uncertainties

*   **Version Field:** Similar to `.afdesign`, the version field at `0x04` varies (`0xB...`).
*   **Header Pointers:** The exact function of pointers at `0x10` and `0x20` is inferred as Content/Object Store pointers, but reverse engineering their internal object graphs is complex and unnecessary for thumbnail extraction.

---

## 9. Conclusion

Affinity Photo files share the unified **Affinity Document Format**. Thumbnail extraction is identical to Affinity Designer: efficient, pointer-based, and relies on standard embedded PNGs. The process is robust and performant.
