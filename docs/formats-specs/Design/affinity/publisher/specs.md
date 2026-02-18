# Technical Analysis: Affinity Publisher (.afpub)

## 1. Format Overview

*   **Extension:** `.afpub`
*   **Origin:** Serif Affinity Publisher.
*   **Category:** Desktop Publishing (DTP) Document / Container.
*   **Magic Signature (Hexadecimal):** `00 FF 4B 41` (Little-Endian: `0x414BFF00`).
*   **Typical Size:** Ranges from 150 KB to hundreds of MB, depending on linked or embedded images.
*   **Variations:** Shares the same base structure (Affinity Common Format) as `.afdesign` and `.afphoto`.

## 2. Global Binary Structure

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `00 FF 4B 41` | Affinity format identifier. |
| `0x04` | 4 bytes | `u32` | **Version/Flags** | Schema version or flags. | Ex: `0xB` (11) or `0x8000B`. |
| `0x08` | 8 bytes | `ASCII` | **Persona ID** | `nsrP#Inf` | "Persona Info" in Little-Endian (`Prsn#Inf`). |
| `0x10` | 8 bytes | `u64` | **Content Ptr** | Content Pointer | Absolute address of the main data block. |
| `0x18` | 8 bytes | `u64` | **Thumb Ptr** | Thumbnail Pointer | Absolute address of the thumbnail block. |
| `0x20` | ... | `u64` | **Other Ptrs** | Other Pointers | Sequence of addresses for additional blocks. |

## 3. Main Header

*   **Structure:** Initial 64-byte block containing signatures and the base addressing table.
*   **Endianness:** **Little-Endian** in all numeric fields.
*   **Critical Fields:** The pointer at `0x18` is the most relevant for fast visualization extraction.

## 4. Identified Internal Structures

Internal sections are organized in blocks with a standard 8-byte header:
*   `0xFFFFFFFF` (4 bytes)
*   Signature (4 bytes, e.g., `Thmb`, `Doc `, `Prop`)

### Thumbnail Block (`Thmb`)
*   **Offset:** Defined in the header at `0x18`.
*   **Structure:**
    *   `+00`: `FF FF FF FF` (Block Marker)
    *   `+04`: `Thmb` (Signature)
    *   `+08`: Version (u32, usually `1`)
    *   `+12`: Total Block Size (u32)
    *   `+16`: Header Length (u32, fixed at `29` or `0x1D`)
    *   `+20`: Zero (u32)
    *   `+24`: Payload Size (u32 - PNG size)
    *   `+28`: Flag (1 byte, e.g., `0x01`)
    *   `+29`: **PNG Data** (Starts with `89 50 4E 47`)

## 5. Endianness

*   **Little-Endian.**
*   **Evidence:** Offset pointers read as `u64` Little-Endian correctly point to data blocks at the end of the file, whereas Big-Endian reads would result in out-of-bounds addresses.

## 6. Compression

*   **Internal Structure:** The document data itself is compressed (likely Zlib) within content blocks.
*   **Thumbnail:** Uses standard **PNG** compression (Deflate/Zlib), facilitating extraction without proprietary libraries.

## 7. Image Data

*   The `.afpub` file does not store a single raw image (like a RAW file), but rather a page layout. However, it embeds a visualization (thumbnail) of the first page or current spread.

## 8. Embedded Thumbnail / Preview

*   **Is there a preview?** Yes.
*   **Format:** Standard **PNG**.
*   **Automatic Detection:**
    1.  Read 8 bytes at `0x18` (Offset `T`).
    2.  Seek to `T`.
    3.  Confirm `FFFFFFFF` + `Thmb`.
    4.  Extract stream starting at `T + 29`.

## 9. Metadata

*   Contains references to external files (Linked assets) and fonts.
*   Strings identified in the header suggest the use of an internal "Object Store" where document properties are serialized.

## 10. Structural Reverse Engineering

*   **Block Container:** The format is essentially a directory of binary blocks accessed by a pointer table at the start of the file.
*   **Resilience:** The use of pointers instead of fixed offsets allows software to append data to the end of the file without rewriting the entire content.

## 11. Strategy for Parser Implementation

1.  Validate Magic `00 FF 4B 41`.
2.  Read thumbnail pointer at `0x18`.
3.  Jump to the read offset.
4.  Validate `Thmb` block header.
5.  Read PNG size at `Offset + 24`.
6.  Extract the buffer and save with `.png` extension.

## 12. Parser Pseudocode

```pseudo
open file
read magic (4 bytes)
if magic != 0x414BFF00: fail

seek to 0x18
thumb_ptr = read_u64_le()

seek to thumb_ptr
block_magic = read_u32()
block_sig = read_string(4)
if block_sig != "Thmb": fail

seek relative +16
png_size = read_u32_le()

seek relative +1
png_data = read(png_size)
save png_data as preview.png
```

## 13. Strategy for Thumbnail Generation

*   **Approach:** Direct extraction of the `Thmb` block.
*   **Complexity:** O(1) - requires only two seeks in the file, regardless of total size.
*   **Pipeline:** `Header Read -> PTR Seek -> Block Validate -> Stream Copy`.

## 14. Strategy for Basic Visualization

*   Render by extracting the embedded PNG. Due to the DTP nature of the file, rendering the full content would require rebuilding the entire layout engine, which is not feasible without the original software. The embedded thumbnail is the intended faithful representation.

## 15. Comparative Map Between Files

| File | Header Version | Thumbnail Ptr | Thumb Size | Observations |
| :--- | :--- | :--- | :--- | :--- |
| `handbook.afpub` | 524299 | `0x17BF55` | 53.8 KB | Manual mock-up. |
| `Flyer German.afpub`| 524299 | `0x19BCE` | 45.4 KB | Simple document. |
| `evermore.afpub` | 11 | `0x18FEBAA`| 8.3 KB | Large document, small thumb (icon). |

## 16. Uncertain Points

*   **Version Field (0x04):** Value varies significantly between software revisions (e.g., `11` vs `524299`). May include compatibility flags.
*   **Content Structure:** The `Prop` (Properties) block contains the serialized object tree, but the serialization format is opaque and proprietary.

## 17. Technical Conclusion

The `.afpub` format is highly structured and efficient for random read operations. Thumbnail extraction is simple and follows a solid industrial pattern, allowing interoperability with assistants and file explorers without risk of corruption or need for heavy dependencies.
