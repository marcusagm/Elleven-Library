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

## 4. Quality Improvements (Step 2)
### 4.1. "Collect All" Strategy
- **Issue:** Legacy extraction was stopping at the first valid preview found (often a small thumbnail like `DISP` or `icp0`).
- **Solution:** 
  - Refactored `walk_riff_generic` to traverse the entire file and collect **all** potential image candidates (`DISP`, `bmp`, `imhd`, `icp0`).
  - Updated `scan_for_embedded_bmp` to return a list of all found BMPs.
  - The extractor now aggregates all candidates from all strategies and selects the **largest** image file by byte size.
  - This ensures that if a high-res preview exists deep in the file (e.g., inside a `cmpr` compressed chunk), it takes precedence over a low-res thumbnail found earlier.

### 4.2. Compression Support (`cmpr`)
- **Issue:** Files from CorelDRAW X3+ often use Zlib compression for `cmpr` chunks, hiding high-quality previews.
- **Solution:**
  - Implemented `cmpr` chunk handling using `flate2::read::ZlibDecoder`.
  - The extractor now decompresses these chunks in-memory and recursively parses them for hidden `DISP` or other image chunks.

### 4.3. Multi-Format & Dynamic Header Support
- **Issue:** 
  - `scan_for_embedded_bmp` only looked for BMPs.
  - `construct_bmp_from_dib` assumed a fixed 40-byte header, breaking modern BMPs (V4/V5) or causing color shifts.
  - **Correction (Step 3):** Initial attempt to scan for JPEG caused false positives (random binary data matching `FF D8`) in legacy files (`DRINKS.CDR`, `11-40.cdr`), breaking valid thumbnails.
- **Solution:**
  - **Disabled JPEG scanning** in fallback mode to prevent corruption.
  - Strengthened PNG scanning to require full 8-byte magic `89 50 4E 47 0D 0A 1A 0A` and valid `IEND` chunk.
  - Updated `construct_bmp_from_dib` to dynamically read the DIB header size (first 4 bytes), ensuring correct pixel offset calculation for any BMP version.
  - `walk_riff_generic` continues to recursively parse structure for legitimate embedded images.

## 5. Conclusion
- **Status:** **Resolved**.
- All sample files now generate valid thumbnails.
- **Quality Constraint:** "Terrible quality" on specific legacy files (e.g., `DRINKS.CDR`, `06-Business Card.cdr`) is confirmed to be an inherent limitation of the source file, which only contains a low-resolution (e.g., 128x128 1-bit) raster thumbnail. No higher-quality raster data exists in these files.
- **Recommendation:** If higher quality is required for these specific legacy files in the future, integration with a vector rendering engine (like `libcdr` or `uniconvertor`) will be necessary, as extraction has reached its theoretical limit.
