# Technical Specification: Krita Document (.kra)

## 1. Format Overview

*   **Extension Name:** `.kra`
*   **Origin:** [Krita Foundation](https://krita.org/) (KDE / Calligra Suite).
*   **Category:** Layered Raster/Vector Image Project / Container.
*   **Magic Signature:** `50 4B 03 04` (ZIP Local File Header). The file is a **standard ZIP container**.
*   **Typical Size:** Few MB to several GB (depends on layer count and resolution).
*   **Variations:** The internal structure (ODF-based) has evolved, but the container remains ZIP.
    *   **Krita 4.x / 5.x:** Stores layers in `projectname/layers/` or similar internal paths.

---

## 2. Global Binary Structure

The file strictly follows the ZIP format (PKWARE).

| Offset | Size | Type | Field Name | Description | Observations |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Local File Header Signature** | ZIP Signature (`0x04034B50`). | Start of the archive. |
| `...` | Variable | - | **File Data** | Compressed (Deflate) or stored data. | Content of XML, PNG, binary layer data. |
| `...` | Variable | - | **Central Directory** | Index of all files in the ZIP. | Located at the end of the file. |
| `EOF-22` | 22 bytes | - | **EOCD Record** | End of Central Directory Record. | End of ZIP marker. |

**Note:** Being a ZIP file, random access is possible via the Central Directory.

---

## 3. Main Header

Not applicable in the traditional sense. The "Header" is the ZIP structure itself.
However, Krita files usually start with the `mimetype` file as the **first entry** in the ZIP, uncompressed, to allow easy identification by file command (ODF convention).

*   **Identification File:** `mimetype`
*   **Content:** `application/x-krita`

---

## 4. Identified Internal Structures

Standard internal file layout:

| File path | Function | Format | Criticality |
| :--- | :--- | :--- | :--- |
| `mimetype` | Format Identification | ASCII Text | High (Identification) |
| `maindoc.xml` | Project Metadata & Structure | XML | High (Parsing) |
| `mergedimage.png` | **Full Canvas Render** | PNG | Medium (High-Res Preview) |
| `preview.png` | **Thumbnail** | PNG | High (Quick Preview) |
| `documentinfo.xml`| Author & Copyright Info | XML | Low |
| `[projectname]/layers/layerN` | Layer Pixel Data | Binary (LZF/Tiled) | High (Editing) |
| `[projectname]/layers/layerN.icc` | Layer Color Profile | ICC Profile | Low |
| `[projectname]/annotations/` | Vector/Text Annotations | XML/SVG | Low |

---

## 5. Endianness

*   **ZIP Container:** Little-endian (PKWARE standard).
*   **XML Files:** Logic/Encoding based (UTF-8).
*   **Binary Layer Data:** Typically Little-endian for length fields if raw, but often handled by specific libraries (LZF).

---

## 6. Compression

*   **Container:** ZIP (Deflate).
*   **Images:** PNG (Deflate).
*   **Layer Data:**
    *   Older versions: Raw or Gzip.
    *   Newer versions: **LZF** compression for tiles.
    *   The binary files inside `layers/` are often tiles of raw pixel data compressed with LZF.

---

## 7. Image Data

The final rendered image is stored as `mergedimage.png`.
Raw layer composition requires parsing `maindoc.xml` to understand the stack (opacity, blending modes) and reading `layerN` files.

*   **Dimensions:** Defined in `maindoc.xml` (`<IMAGE width="..." height="..." />`).
*   **Reconstruction:**
    *   **Simple:** Read `mergedimage.png`.
    *   **Complex:** Parse XML -> Iterate Layers -> Decompress Tiles -> Composite.

---

## 8. Embedded Thumbnail / Preview

The format contains TWO previews.

### 8.1. `preview.png`
*   **Usage:** Thumbnail for file managers.
*   **Size:** Usually small (e.g., 256x256 or similar aspect ratio).
*   **Format:** PNG.
*   **Location:** Root of ZIP.

### 8.2. `mergedimage.png`
*   **Usage:** Full resolution composite.
*   **Size:** Equal to canvas size (e.g., 6000x4000).
*   **Format:** PNG.
*   **Location:** Root of ZIP.
*   **Note:** This file might be missing in very old versions or if disabled in settings, but is standard in default Krita.

---

## 9. Metadata

The `maindoc.xml` contains the true project state.

**Structure:**
*   Root: `<DOC>` or namespaced equivalent.
*   `<IMAGE>`: Attributes `width`, `height`, `mime`, `name`.
*   `<LAYERS>`: Recursive structure defining groups and layers.
    *   `<LAYER filename="layer2" ...>`: Links to binary file.

**Sample Tags:**
```xml
<IMAGE width="3000" height="2000" mime="application/x-kra" name="Unnamed">
  <layers>
    <layer name="Layer 1" filename="layer2" x="0" y="0" opacity="255" ... />
  </layers>
</IMAGE>
```

---

## 10. Structural Reverse Engineering

*   **Pattern:** ODF-like ZIP Container.
*   **Storage Strategy:** Separation of Metadata (XML) and Data (Binary/PNG).
*   **Reference:** Layers are referenced by filename in the XML, pointing to files inside the ZIP (often inside a folder with the same name as the project/image).

---

## 11. Strategy for Parser Implementation

1.  **Open ZIP:** Validate magic `PK\x03\x04`.
2.  **Read `mimetype`:** Confirm `application/x-krita`.
3.  **Extract `maindoc.xml`:** Parse dimensions and layer count.
4.  **Extract `preview.png`:** For thumbnails.
5.  **Extract `mergedimage.png`:** For full viewer.

---

## 12. Parser Pseudocode

```python
def parse_krita(filepath):
    if not is_zip_file(filepath):
        raise InvalidFormat()
        
    with ZipFile(filepath) as zf:
        # Validation
        if "mimetype" in zf.namelist():
             if zf.read("mimetype").decode() != "application/x-krita":
                  warn("Unknown mimetype")
                  
        # Dimensions
        xml_data = zf.read("maindoc.xml")
        width, height = extract_xml_dimensions(xml_data)
        
        # Metadata
        meta = {
            "width": width,
            "height": height,
            "layers": extract_layer_info(xml_data)
        }
        
        # Preview
        if "preview.png" in zf.namelist():
            return Preview(zf.read("preview.png"), meta)
        elif "mergedimage.png" in zf.namelist():
            # Resize merged image if thumbnail needed
            full_img = zf.read("mergedimage.png")
            return Resize(full_img, 256), meta
            
    raise NoPreviewFound()
```

---

## 13. Strategy for Thumbnail Generation

ALWAYS use `preview.png` or `mergedimage.png`.

*   **Pros:** O(1) extraction complexity. No rendering needed.
*   **Cons:** `preview.png` might be low res. `mergedimage.png` requires reading large data.
*   **Pipeline:**
    1.  Check `preview.png`. If exists -> Return.
    2.  Check `mergedimage.png`. If exists -> Load -> Resize -> Return.
    3.  If both fail, file is likely corrupted or very old.

---

## 14. Comparative Map Between Files

| File | XML Version | Preview Size | Merged Image? | Layers |
| :--- | :--- | :--- | :--- | :--- |
| `2024-11-03_for_Huion.kra` | 5.x | ~70KB | Yes (27MB) | Multiple (Binary Files) |
| `2023-03-30...Save-Point.kra` | 5.x | ~120KB | Yes (11MB) | Multiple |

---

## 16. Uncertain Points

1.  **Layer Compression:** The exact compression of the pixel tiles in `layers/` (LZF vs others) depends on the exact Krita version.
2.  **Color Space:** ICC profiles are stored separately (`.icc`). Rendering the raw layers correctly requires applying these profiles, which adds significant complexity compared to using `mergedimage.png` (which is usually sRGB or pre-converted for display).

---

## 17. Technical Conclusion

The `.kra` format is extremely developer-friendly for extraction purposes.
*   **Parsing:** Trivial (Standard ZIP/XML).
*   **Preview:** Immediate (`preview.png`).
*   **Full Render:** Immediate (`mergedimage.png`).
*   **Risk:** Very low.

Recommendation: Utilize standard ZIP libraries to extract `preview.png` for thumbnails and `mergedimage.png` for detailed views.
