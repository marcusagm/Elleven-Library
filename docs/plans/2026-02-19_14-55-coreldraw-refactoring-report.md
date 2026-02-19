# Refactoring Report: CorelDRAW Thumbnail Extraction Improvements

**Date:** 2026-02-19
**Task:** Enhance CorelDRAW (.cdr) support to fix missing/low-quality thumbnails for legacy and specific modern files.
**Executor:** Antigravity Agent

## 1. Problem Statement
The initial implementation of the CorelDRAW extractor failed to generate thumbnails for several files in the sample dataset.
- **Modern Files (ZIP):** Some files were not showing expected previews or were picking low-resolution thumbnails when higher-quality page previews were available.
- **Legacy Files (RIFF):** Many files failed to extract the `DISP` chunk correctly due to unrecognized headers or deep nesting.
- **Ultra-Legacy Files (WL):** Files with `WL` signature (CorelDRAW v3-v5) were completely unsupported.

## 2. Investigation Findings

### 2.1. Modern Formats (ZIP-based, X4+)
- **Issue:** The extractor was blindly looking for specific paths like `previews/thumbnail.png`.
- **Finding:**
  - Files often contain multiple potential preview images: `previews/thumbnail.png`, `previews/page1.png`, `content/preview.png`.
  - `thumbnail.png` is often very small (e.g., 3KB), while `page1.png` is much larger and higher quality (e.g., 13KB).
- **Solution:** Implement a "Best Quality" selector that scans all candidates and chooses the largest file.

### 2.2. Legacy Formats (RIFF-based, v6-X3)
- **Issue:** `DISP` chunks were not being found, or their content was not recognized as BMP.
- **Findings:**
  - **Recursion:** The `DISP` chunk is often nested deep within `LIST` chunks of types `doc `, `page`, `gobj`, or even at the top level.
  - **Headers:** The `DISP` chunk data does not always start with a clean BMP header.
    - **Variant A:** Starts with `08 00 00 00 28 00 00 00`. The `28` at offset 4 indicates a 40-byte `BITMAPINFOHEADER`.
    - **Variant B (e.g., 03- Design.cdr):** Starts with `2C 28 00 00`. The `28` is at offset 1.
    - **Variant C:** Embedded `imhd` chunk containing a direct BMP stream.
- **Solution:**
  - Added recursive RIFF walker for `doc `, `page`, `gobj`.
  - Implemented flexible header parsing to construct a valid BMP from raw DIB data found in `DISP` chunks.
  - Added support for `CDRD` RIFF signature (CorelDRAW 4/5).

### 2.3. Ultra-Legacy Formats (WL-based, v3-v5)
- **Issue:** Files starting with `57 4C` (WL) were treated as invalid/unknown.
- **Findings (Hex Dump Analysis):**
  - **Header:** Fixed header starting with `WL` (`57 4C`).
  - **Dimensions:** Width at offset `0x48`, Height at offset `0x4A`.
  - **Endianness:** **Big Endian** (Critical finding! `00 5A` = 90).
  - **Data:** 1-bit monochrome bitmap data starts at offset `0x56`.
- **Solution:** Implemented `extract_wl_thumbnail` to read the header, parse Big Endian dimensions, and wrap the raw 1-bit data into a valid BMP container with a Black/White palette.

## 3. Implementation Details

### 3.1. Architecture
The `extract_coreldraw_preview` function now routes to three distinct strategies:
1.  **Modern ZIP:** `extract_zip_best_quality`
2.  **Legacy RIFF:** `extract_riff_preview_recursive`
3.  **Legacy WL:** `extract_wl_thumbnail` (with fallback to `scan_for_embedded_bmp`)

### 3.2. Verification
All sample files provided by the user now generate thumbnails:
- `quadriculados2.cdr`: High-quality PNG from ZIP.
- `11-40 Vector Designs.cdr`: BMP from RIFF `DISP` (Standard variant).
- `03- Design.cdr`: BMP from RIFF `DISP` (Top-level, offset variant).
- `FLAG.CDR`: 1-bit BMP from WL header (90x90).
- `DRINKS.CDR`: 1-bit BMP from WL header (128x128).
- `FOX.CDR`: BMP from `imhd` chunk.

## 4. Next Steps
- Monitor for other `DISP` header variants.
- Consider using an external converter (like `uniconvertor` or `inkscape`) for vectors if higher quality is needed for legacy files.
