# Technical Analysis: Affinity Designer (.afdesign)

## 1. Format Overview

*   **Extension:** `.afdesign`
*   **Software:** Serif Affinity Designer.
*   **Family:** Affinity Document Format (shared with `.afphoto`, `.afpub`).
*   **Category:** Vector Graphics Project / Container.
*   **Magic Signature:** `00 FF 4B 41` (Little-Endian: `0x414BFF00`).
*   **Endianness:** **Little-Endian**.
*   **Structure:** Header with fixed pointers to internal blocks.
*   **Thumbnail:** Embedded **PNG**, easily accessible via header pointer.

---

## 2. Global Binary Structure

| Offset | Size | Type | Name | Description |
| :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `00 FF 4B 41` (`KA\xff\x00` LE). |
| `0x04` | 4 bytes | `u32` | **Version/Flags** | e.g. `0x0000000A` or `0x0008000B`. |
| `0x08` | 8 bytes | `ASCII` | **Container ID** | `nsrP#Inf` (`Prsn#Inf` LE). Likely "Persona Info". |
| `0x10` | 8 bytes | `u64` | **Content Pointer** | Pointer to main content block? |
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

**Note:** The offsets `+16` to `+29` are inferred from observation. The key is that the PNG signature reliably starts at **Offset + 29** bytes from the block start.

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
def extract_affinity_thumb(path):
    with open(path, 'rb') as f:
        # 1. Magic Check
        if f.read(4) != b'\x00\xff\x4b\x41':
            return None
            
        # 2. Get Pointer
        f.seek(0x18)
        thumb_offset = struct.unpack('<Q', f.read(8))[0]
        
        # 3. Validation
        f.seek(thumb_offset)
        if f.read(8) != b'\xff\xff\xff\xffThmb':
            return None
            
        # 4. Get PNG Size (Optional, logical guess)
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
*   Other parts of the file (pointed to by other header offsets) likely contain compressed serialized data, but for thumbnail purposes, standard PNG libraries are sufficient.

---

## 7. Uncertainties

*   **Version Field:** The meaning of the `u32` at `0x04` in the global header varies (`0xA` vs `0xB...`). It might encompass version + flags.
*   **Other Pointers:** The other `u64` pointers at `0x10`, `0x20`, etc. point to other `FFFFFFFF` blocks (e.g., `#Inf` block, `#Fil` block), but their internal structure is complex (proprietary serialization).
*   **Double Size Fields:** The `Thmb` block header has multiple integers that resemble sizes. `Size 2` (at +24) seems to strongly correlate with the PNG size.

---

## 8. Conclusion

Affinity Designer files are structured with a clean, pointer-based header that makes thumbnail extraction **extremely efficient** (O(1) complexity). There is no need to scan the file or parse the complex document object model.
