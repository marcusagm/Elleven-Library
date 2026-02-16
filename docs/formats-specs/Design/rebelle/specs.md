
# Technical Specification: Rebelle Project (.reb)

## 1. Format Overview

*   **Extension Name:** `.reb`
*   **Origin:** [Escape Motions](https://www.escapemotions.com/), **Rebelle** software.
*   **Category:** Digital Art Project / Container.
*   **Magic Signature:** `50 4B 03 04` (ZIP Local File Header) or `50 4B 05 06` (ZIP End of Central Directory). The file is a **standard ZIP container**.
*   **Typical Size:** Tens to hundreds of Megabytes (depends on resolution and number of layers).
*   **Variations:** The internal file structure (names and XML) seems consistent between version 5 and recent versions.

---

## 2. Global Binary Structure

The file strictly follows the ZIP format (PKWARE).

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Local File Header Signature** | ZIP Signature (`0x04034B50`). | Start of the first file in the archive. |
| `...` | Variable | - | **File Data** | Compressed (Deflate) or stored data. | Content of XML, PNG, DAT files. |
| `...` | Variable | - | **Central Directory** | Index of all files in the ZIP. | Located at the end of the file. |
| `EOF-22` | 22 bytes | - | **EOCD Record** | End of Central Directory Record. | End of ZIP marker. |

**Note:** As it is a ZIP, there is no proprietary `.reb` "Global Header" at offset 0. Identification must be done by the presence of **specific files** within the container (e.g., `artwork.xml`, `canvas.png`).

---

## 3. Main Header

Not applicable (see section 2). The project's logical "Header" is contained in the `artwork.xml` file within the archive.

*   **Identification File:** `artwork.xml`
*   **Root Tag:** `<aquarelle_artwork>`
*   **Important Attributes:**
    *   `version`: Integer representing the version (e.g., `511` for 5.1.1).
    *   `file_format_version`: Structural version of the file (e.g., `5`).

---

## 4. Identified Internal Structures

Inside the ZIP, the following files are standard:

| File (Pattern) | Function | Internal Format | Observations |
| :--- | :--- | :--- | :--- |
| `artwork.xml` | Project Metadata | XML (UTF-8) | Contains dimensions, layer list, color history. |
| `canvas.png` | **Composite Preview** | PNG (Standard) | The final rendered image ("merged"). Essential for thumbnail. |
| `paper.png` | Paper Texture | PNG (Standard) | Background texture used in simulation. |
| `layer{N}.png` | Image Layer | PNG (Standard) | Color data (RGB+A) of layer N. |
| `layer{N}_flow.dat` | Fluid Map | Proprietary Binary | Simulation data (wetness, pigment). Signature `BBOX`. |
| `layer{N}_structure.dat` | Structure Map | Proprietary Binary | Height/impasto data of the paint. |
| `profile.icc` | Color Profile | ICC Profile | Color management (optional). |

---

## 5. Endianness

*   **ZIP Container:** Little-endian (PKWARE standard).
*   **Internal .dat Files:**
    *   Analysis of `layer0_flow.dat`:
    *   Signature `BBOX` (4 bytes).
    *   Following values seem to be 32-bit Little-endian integers.
    *   Example: `00 00 01 59` -> 345 (visual endianness break, but consistent with width/height).
    *   **Conclusion:** Predominantly **Little-endian** for binary metadata.

---

## 6. Compression

*   **Container:** ZIP (Deflate).
*   **Images:** PNG (Deflate).
*   **Binary Data (.dat):** Appear to have uncompressed headers (`BBOX`, `UCHA`), followed by data that may be compressed (zlib) or raw arrays of floats/integers. High entropy suggests compression or dense floating-point data.

---

## 7. Image Data

The final image and layers are stored as **standard PNGs**.

*   **Dimensions:** Defined in `artwork.xml` (`<canvas width='...' height='...'/>`) and match the IHDR headers of the PNGs.
*   **Bit Depth:** Typically 8 or 16 bits per channel (depending on project configuration).
*   **Color Type:** RGB or RGBA (Alpha channel is common for transparent layers).
*   **Reconstruction:**
    *   For quick viewing: Use `canvas.png`.
    *   For faithful editable reconstruction: Stack `layer{N}.png` respecting the order and blending modes (`blending_mode`) defined in `artwork.xml` (`<layer ... blending_mode='NORMAL' .../>`).

---

## 8. Thumbnail / Embedded Preview

The format **possesses** a high-quality preview ready for use.

*   **Target File:** `canvas.png`
*   **Location:** Root of the ZIP.
*   **Format:** PNG.
*   **Extraction:**
    1.  Open ZIP.
    2.  Locate entry `canvas.png`.
    3.  Decompress stream.
*   **Automatic Detection:** Check for existence of entry `canvas.png` or `preview.png` (in older or future versions).

---

## 9. Metadata

The `artwork.xml` file is the primary source.

**Main Tags:**
*   `<canvas width='1654.000000' height='2339.000000' .../>`: Physical dimensions.
*   `<paper name='HP01 Hot Pressed' .../>`: Type of paper used.
*   `<layer ... name='Layer 1' type='FLUID' opacity='1' blending_mode='NORMAL' .../>`: Layer definition.
*   `<reference_colors_history>`: Recently used color palette (Hex codes).
*   `<speedpaint_recording .../>`: If timelapse recording is configured.

---

## 10. Structural Reverse Engineering (.dat Files)

Preliminary analysis of `_flow.dat` files:

*   **Header:** 16 bytes.
    *   Magic: `42 42 4F 58` ("BBOX" - ASCII).
    *   Unknown (4 bytes): `00 00 01 59` (Probable Coordinate or Dimension?).
    *   Unknown (4 bytes): `00 00 00 B9`.
    *   Unknown (4 bytes): `00 00 09 B3`.
    *   *Hypothesis:* Bounding Box Coordinates (X1, Y1, X2, Y2) to optimize processing only in the painted area.
*   **Next Block:**
    *   Magic: `55 43 48 41` ("UCHA" - ASCII).
    *   Size/Length: 4 subsequent bytes (`00 6C 96 A5` in analyzed example).
    *   Data: Dense binary content (probably array of floats for fluid simulation).

---

## 11. Strategy for Parser Implementation

For cataloging purposes (Mundam), it is not necessary to parse `.dat` files.

**Suggested Pipeline:**

1.  **Quick Validation:** Check Magic Bytes of the file (`PK\x03\x04`).
2.  **Central Directory Scan:** List internal files.
3.  **Identification:** Look for `artwork.xml` and `canvas.png`. If absent, it is not a valid Rebelle file (or is an unknown version).
4.  **Preview Extraction:** Extract `canvas.png`.
5.  **Metadata Extraction (Optional):** Parse `artwork.xml` (SAX API or simple DOM) to obtain exact dimensions and version.

---

## 12. Parser Pseudocode

```python
def parse_rebelle(filepath):
    if not is_zip_file(filepath):
        raise InvalidFormatException("Not a ZIP container")

    with ZipFile(filepath) as zf:
        file_list = zf.namelist()
        
        # Format validation
        if "artwork.xml" not in file_list:
            raise InvalidFormatException("Missing artwork.xml")
            
        # Basic Metadata Extraction
        with zf.open("artwork.xml") as meta_file:
            xml_tree = parse_xml(meta_file)
            width = xml_tree.find("canvas").attr("width")
            height = xml_tree.find("canvas").attr("height")
            version = xml_tree.root.attr("version_str")

        # Image Extraction
        if "canvas.png" in file_list:
            preview_data = zf.read("canvas.png")
            return {
                "metadata": {
                    "width": width, 
                    "height": height, 
                    "software": f"Rebelle {version}"
                },
                "preview_blob": preview_data
            }
        else:
            # Fallback (unlikely in valid files)
            raise MissingPreviewException()
```

---

## 13. Strategy for Thumbnail Generation

The best approach is to **always use the internal preview (`canvas.png`)**.

*   **Reason:** Rendering the raw file would require re-implementing the fluid simulation engine from Escape Motions (impossible without deep reverse engineering and access to proprietary algorithms).
*   **Complexity:**
    *   **Extraction:** O(1) (direct access to ZIP stream).
    *   **Decode:** O(N) (where N is the size of canvas.png).
    *   **Resize:** O(M) (where M are the pixels of the image).
*   **Performance:** Very high. `canvas.png` usually has a few MBs, while processing `layer_flow.dat` would be unfeasible.

---

## 14. Strategy for Basic Visualization

RAW conversion is not necessary. `canvas.png` is already in sRGB color space (usually, check `profile.icc` if absolute color precision is critical).

*   **Pipeline:**
    `ZIP Extract -> PNG Decode -> (Optional ICC Transform) -> Display`

---

## 15. Comparative Map Between Files

| Analyzed File | Version (XML) | Size | Extra Fields | Observations |
| :--- | :--- | :--- | :--- | :--- |
| `Gordin.reb` | 5.1.1 | 37MB | `struct_flow.dat` | Standard version 5. |
| `portrait.reb` | 5.1.1 | 34MB | `struct_flow.dat` | Structure identical to the previous one. |

Apparently, the structure is stable in the 5.x series.

---

## 16. Uncertain Points

1.  **Exact meaning of .dat:** The `_flow.dat` and `_structure.dat` files contain the Rebelle "magic" (wetness, ink spreading). Their exact structure has not been completely reversed here (only BBOX/UCHA headers identified). *Confidence that they are not needed for thumbnail: 100%.*
2.  **Old Versions:** Files from version 3 or 4 were not analyzed. There might be differences in the preview file name (e.g., `merged.png` instead of `canvas.png`), but the ZIP standard should hold.

---

## 17. Technical Conclusion

The `.reb` format is **friendly for integration**.
Its nature based on ZIP + XML + PNG removes the need for complex binary parsing for archiving and viewing tasks. The guaranteed presence of `canvas.png` makes thumbnail generation trivial and performant.

*   **Parsing Complexity:** Low.
*   **Implementation Risks:** Low (standard ZIP/XML/PNG dependencies).
*   **Recommendation:** Implement via direct extraction of `canvas.png` ignoring proprietary simulation data.
