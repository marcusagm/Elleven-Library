# Adobe Photoshop (.psd) File Format Technical Specification

## 1. Format Overview
*   **Extension Name:** `.psd` (Photoshop Document).
*   **Possible Origin:** Developed by Adobe Systems Inc.
*   **Category:** Multilayer Raster Image Document.
*   **Magic Signature (Hexadecimal):** `38 42 50 53` (`8BPS`).
*   **Typical Size Observed:** 1.6 MB to 120 MB in samples (can reach gigabytes in `.psb` format).
*   **Variations Between Analyzed Files:** All analyzed files (except low-resolution base samples) contain complex resource blocks including JPEG thumbnails and XMP metadata.

## 2. Global Binary Structure

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 26 bytes | `Header` | **File Header** | Basic document metadata. | Fixed size. |
| Variable | 4 + N | `Block`  | **Color Mode Data** | Indexed color table or duotone data. | Usually 0 for RGB/CMYK. |
| Variable | 4 + N | `Block`  | **Image Resources** | Metadata, previews, paths, etc. | Iterative structure based on IDs. |
| Variable | 4 + N | `Block`  | **Layer & Mask Info** | Data for all layers and masks. | Often the largest section. |
| Variable | 2 + N | `Data`   | **Image Data** | Final merged (composite) image. | Point of immediate visualization. |

## 3. Main Header
*   **Detailed Structure:**
    *   `0x00`: Signature (4 bytes) - `8BPS`.
    *   `0x04`: Version (2 bytes) - `1` for PSD, `2` for PSB.
    *   `0x06`: Reserved (6 bytes) - Must be zero.
    *   `0x0C`: Channels (2 bytes) - Number of color channels (1-56).
    *   `0x0E`: Height (4 bytes) - Height in pixels.
    *   `0x12`: Width (4 bytes) - Width in pixels.
    *   `0x16`: Depth (2 bytes) - Bits per channel (1, 8, 16, 32).
    *   `0x18`: Color Mode (2 bytes) - Color mode (3 = RGB, 4 = CMYK, etc).
*   **Endianness:** Big-Endian.
*   **Flags/Checksums:** There are no global checksums in the basic header.

## 4. Identified Internal Structures

### 4.1. Image Resource Block (8BIM)
*   **Initial Offset:** After the Color Mode Data section.
*   **Size:** Variable (defined at the beginning of the section).
*   **Internal Structure:**
    *   Signature (4 bytes): `8BIM`.
    *   ID (2 bytes): Resource identifier (e.g., 1036 for Thumbnail).
    *   Name (Pascal String): Resource name (aligned to 2 bytes).
    *   Size (4 bytes): Resource data length.
    *   Data (Variable): Resource payload (aligned to 2 bytes).
*   **Function:** Repeated N times to store thumbnails, ICC profiles, XMP metadata, and guides.

## 5. Endianness
*   **Big-Endian:** Absolutely all numeric fields (16, 32, and 64-bit integers) follow the most significant byte first format.
*   **Evidence Found:** The version field `00 01` and resolutions observed in the hexadecimal data confirm the Adobe Standard (Big-Endian) order.

## 6. Compression
*   **Indication:** The initial field of the *Image Data* section indicates the method.
*   **Algorithms:**
    *   `0`: Raw (No compression).
    *   `1`: RLE (PackBits).
    *   `2`: Zip without prediction.
    *   `3`: Zip with prediction.
*   **Strategy:** For RLE, each channel/row must be decompressed sequentially according to the row length table.

## 7. Image Data (Merged Composite)
*   **Offset:** Located in the final section of the file.
*   **Format:** Planar (Separate channels). If RGB, it stores all pixels of the R channel, followed by G, then B.
*   **Bit Depth:** 8-bit is most common, but 16-bit and 32-bit (seen in HDR) are supported.
*   **Reconstruction:** Interleave planar data into an RGBA/RGB buffer for display.

## 8. Embedded Thumbnail / Preview
*   **Is there a preview?** Yes, highly common.
*   **Offset:** Within the Image Resources section.
*   **Resource ID:** `1036` (or `1033`).
*   **Format:** Encapsulated JPEG (KJpegRGB).
*   **Extraction:** Locate resource 1036, skip the 28 bytes of fixed thumbnail header (dimensions and internal metadata), and extract the JPEG stream that starts immediately after.

## 9. Metadata
*   **XMP Metadata:** Found in Resource ID `1060`. Plain text XML formatted by Adobe.
*   **EXIF:** Frequently embedded in XMP metadata or specific resource blocks.
*   **Strings:** UTF-8 layer names found (within the layer section) and resource names in Pascal Strings.

## 10. Structural Reverse Engineering
*   **Recurring Patterns:** Blocks with signature `8BIM` followed by `Size` lengths.
*   **TLV (Type-Length-Value):** The entire internal architecture of resources and layers is based on TLV.
*   **Alignment:** Byte padding is necessary to ensure each block starts at an even offset.

## 11. Strategy for Parser Implementation
1.  **Order:** Header -> Skip ColorMode -> Iterate Resources (Target 1036) -> Layer Metadata.
2.  **Validations:** Check if the resource signature is `8BIM`. If it fails, the parser has lost alignment.
3.  **Error Handling:** Use the total section length to avoid reading beyond limits in malformed files.

## 12. Parser Pseudocode
```pseudo
open file
read magic (4 bytes) -> must be "8BPS"
read version (2 bytes) -> 1=PSD, 2=PSB
skip reserved(6)
width, height, depth = read_header_dims()

skip color_mode_data_len

resource_section_len = read_u32()
end_resource_offset = current_pos + resource_section_len

while current_pos < end_resource_offset:
    sig = read(4) # Expect "8BIM"
    id = read_u16()
    name = read_pascal_string_aligned() 
    data_size = read_u32()
    
    if id == 1036:
        # Extract Thumbnail
        skip(28) # Thumbnail header
        jpeg_buffer = read(data_size - 28)
        save jpeg_buffer as "preview.jpg"
        break
    
    skip(data_size + padding)
```

## 13. Strategy for Thumbnail Generation
*   **Best Approach:** Use Resource 1036 (JPEG). It is performant and reflects exactly the artist's intention upon saving.
*   **Fallback:** Decompress the merged image in the final section. Requires implementation of RLE or Zip decompression and channel planar recomposition.

## 14. Strategy for Basic Visualization
*   If the document is in RGB mode, the 1036 thumbnail is a ready-to-use JPEG file.
*   For high-fidelity visualization, render the *Image Data* section applying the color mode (RGB to RGB, CMYK to RGB via basic ICC profile).

## 15. Comparative Map Between Files
| File | Version | Resolution | Channels | Resources | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `SPC_8187.psd` | 1 | 4912x7360 | 4 | XMP + Thumb | Heavy professional file. |
| `sample.psd` | 1 | 758x960 | 4 | XMP + Thumb | Standard example. |
| `sample_640x426.psd`| 1 | 640x426 | 3 | ResolutionInfo | No embedded thumbnail. |

## 16. Uncertain Points
*   **Pascal String Alignment:** Some third-party softwares may not correctly align the resource name to 2 bytes (Confidence: 90%).
*   **Zlib Predictor:** The prediction algorithm in Zip mode (type 3) may vary slightly between Photoshop versions (Confidence: 85%).

## 17. Technical Conclusion
The PSD format is robust and extensible, using a modular resource architecture. Thumbnail extraction is simple due to the encapsulation of standard JPEG streams within ID-fixed blocks, allowing external tools to generate fast previews without processing the entire layer tree.
