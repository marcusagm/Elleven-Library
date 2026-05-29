# Bitmap Image File (`.bmp`) File Format Technical Specification

## Format Overview

* **Extension Name**:

  * `.bmp`
  * `.dib` (Device Independent Bitmap)
* **Possible Origin**: Developed by Microsoft and IBM for Microsoft Windows and OS/2 graphical subsystems
* **Category**: Raster Bitmap Image Format
* **LibRaw Support**: No
* **FFMPEG Support**: Yes (broad decoding and encoding support)
* **Rust alternative converters**:

  * `image`
  * `bmp`
  * `fast_image_resize`
  * `rgb`
  * `webp`
  * `ravif`
  * External tools:

    * `ffmpeg`
    * `ImageMagick`
    * `GraphicsMagick`
    * `vips`
    * `stb_image`
    * `Pillow`
    * `GDI+`

BMP is one of the oldest and simplest mainstream raster image formats.

It was originally designed for:

* Windows graphics APIs
* framebuffer-oriented rendering
* device-independent image storage

BMP is fundamentally:

```text id="bmp01"
a relatively simple raster bitmap container
```

compared to:

* TIFF
* AVIF
* JPEG XL
* RAW formats

Historically BMP prioritized:

* simplicity
* fast memory mapping
* easy rendering
* low CPU decode cost

rather than:

* compression efficiency
* metadata richness
* HDR workflows

---

# File structure

## High-Level Container Layout

Typical BMP structure:

```text id="bmp02"
+----------------------+
| BMP File Header      |
+----------------------+
| DIB Header           |
+----------------------+
| Color Table          |
+----------------------+
| Bit Masks            |
+----------------------+
| Pixel Array          |
+----------------------+
```

BMP is mostly:

```text id="bmp03"
header + raw pixel storage
```

with optional compression.

---

# BMP File Header

## BITMAPFILEHEADER

Classic structure:

```c id="bmp04"
struct BITMAPFILEHEADER {
    uint16 bfType;
    uint32 bfSize;
    uint16 bfReserved1;
    uint16 bfReserved2;
    uint32 bfOffBits;
}
```

---

## Magic Bytes

Standard BMP signature:

```hex id="bmp05"
42 4D
```

ASCII:

```text id="bmp06"
BM
```

Other historical signatures exist:

* BA
* CI
* CP
* IC
* PT

but modern BMP files almost always use:

```text id="bmp07"
BM
```

---

# DIB Header

## Device Independent Bitmap Header

The DIB header defines:

* dimensions
* pixel format
* compression
* color masks
* color depth

Many BMP variants differ primarily in:

```text id="bmp08"
DIB header version
```

---

# Common DIB Header Types

## 1. BITMAPCOREHEADER

Old OS/2 format.

Very limited.

---

## 2. BITMAPINFOHEADER

Most common BMP header.

Structure:

```c id="bmp09"
struct BITMAPINFOHEADER {
    uint32 biSize;
    int32  biWidth;
    int32  biHeight;
    uint16 biPlanes;
    uint16 biBitCount;
    uint32 biCompression;
    uint32 biSizeImage;
    int32  biXPelsPerMeter;
    int32  biYPelsPerMeter;
    uint32 biClrUsed;
    uint32 biClrImportant;
}
```

---

## 3. BITMAPV4HEADER

Adds:

* color masks
* color space support
* gamma

---

## 4. BITMAPV5HEADER

Adds:

* ICC profiles
* rendering intent
* advanced color metadata

Most modern advanced BMP workflows use:

```text id="bmp10"
BITMAPV5HEADER
```

---

# Pixel Storage

## Bottom-Up Orientation

Classic BMP images are:

```text id="bmp11"
stored upside-down
```

Meaning:

* first row = bottom scanline
* last row = top scanline

Positive height:

```text id="bmp12"
bottom-up bitmap
```

Negative height:

```text id="bmp13"
top-down bitmap
```

Critical implementation detail.

---

# Row Alignment

BMP scanlines are aligned to:

```text id="bmp14"
4-byte boundaries
```

Each row is padded.

Example:

| Width | RGB24 Bytes | Stored Row Bytes |
| ----- | ----------- | ---------------- |
| 1     | 3           | 4                |
| 2     | 6           | 8                |
| 3     | 9           | 12               |

Very important for parsing correctness.

---

# Color Depth Support

BMP supports many bit depths:

| Depth  | Description        |
| ------ | ------------------ |
| 1-bit  | Monochrome         |
| 4-bit  | Indexed            |
| 8-bit  | Indexed            |
| 16-bit | High color         |
| 24-bit | True color         |
| 32-bit | True color + alpha |

---

# Indexed Color Modes

## Palette-Based BMP

1/4/8-bit BMP files use:

```text id="bmp15"
palette lookup tables
```

Palette entries usually:

```c id="bmp16"
struct RGBQUAD {
    uint8 blue;
    uint8 green;
    uint8 red;
    uint8 reserved;
}
```

---

# True Color Modes

## 24-bit BMP

Typical pixel order:

```text id="bmp17"
BGR
```

NOT RGB.

Very important.

---

## 32-bit BMP

Typical layout:

```text id="bmp18"
BGRA
```

Alpha handling varies across implementations.

---

# Compression Support

BMP supports several compression modes.

---

## 1. BI_RGB

Value:

```text id="bmp19"
0
```

Meaning:

```text id="bmp20"
uncompressed
```

Most common BMP mode.

---

## 2. BI_RLE8

Value:

```text id="bmp21"
1
```

8-bit RLE compression.

---

## 3. BI_RLE4

Value:

```text id="bmp22"
2
```

4-bit RLE compression.

---

## 4. BI_BITFIELDS

Value:

```text id="bmp23"
3
```

Uses explicit channel masks.

Common for:

* RGB565
* ARGB1555
* BGRA layouts

---

## 5. BI_JPEG / BI_PNG

Rare.

BMP container may embed:

* JPEG
* PNG

Not widely used.

---

# Channel Masks

## BITFIELDS Compression

Example RGB565 masks:

```hex id="bmp24"
Red:   0xF800
Green: 0x07E0
Blue:  0x001F
```

ARGB8888 example:

```hex id="bmp25"
Red:   0x00FF0000
Green: 0x0000FF00
Blue:  0x000000FF
Alpha: 0xFF000000
```

Mask interpretation is:

```text id="bmp26"
critical
```

for correct rendering.

---

# Alpha Channel Support

## Historical BMP Issues

Classic BMP:

```text id="bmp27"
did not reliably support alpha
```

Modern BMP variants:

* V4
* V5

support alpha more consistently.

However ecosystem behavior remains inconsistent.

---

# Color Management

## ICC Profile Support

BMP V5 supports:

* ICC profiles
* rendering intent
* color spaces

In practice:

```text id="bmp28"
rarely used
```

Most BMPs are effectively:

```text id="bmp29"
sRGB-like
```

without explicit metadata.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Direct Decode + Resize

BMP is simple enough that:

```text id="bmp30"
full decode is usually inexpensive
```

Pipeline:

```text id="bmp31"
BMP decode
 → normalize pixel format
 → resize
 → encode WebP
```

---

## Tier 2 — Streaming Decode

Useful for:

* gigantic BMPs
* scientific BMPs
* memory-constrained environments

Pipeline:

```text id="bmp32"
scanline decode
 → incremental resize
 → WebP encode
```

---

# Recommended Rust Thumbnail Pipeline

## Suggested Crates

```toml id="bmp33"
image
bmp
fast_image_resize
webp
rayon
```

---

## Ideal WebP Settings

### Gallery thumbnails

```text id="bmp34"
Quality: 70–85
Lossy WebP
```

### Archival previews

```text id="bmp35"
Quality: 90–100
Lossless WebP
```

---

# Strategy for Visualization

## Important Principle

BMP rendering complexity depends mainly on:

* DIB header version
* bit depth
* compression mode
* channel masks

BMP is:

```text id="bmp36"
structurally simple but historically inconsistent
```

---

# Recommended Visualization Pipeline

## Stage 1 — Header Parsing

Read:

* BMP header
* DIB header
* file offsets
* compression mode

Recommended crates:

```toml id="bmp37"
binrw
nom
```

Preferred:

```toml id="bmp38"
binrw
```

---

## Stage 2 — DIB Variant Detection

Determine:

* COREHEADER
* INFOHEADER
* V4
* V5

because field interpretation differs.

---

## Stage 3 — Compression Handling

Dispatch based on:

```text id="bmp39"
biCompression
```

Possible paths:

* raw
* RLE4
* RLE8
* BITFIELDS

---

## Stage 4 — Palette Decode

Required for:

* 1-bit
* 4-bit
* 8-bit BMP

Pipeline:

```text id="bmp40"
indexed pixel
 → palette lookup
 → RGB conversion
```

---

## Stage 5 — Bitfield Decode

Critical for:

* 16-bit BMP
* 32-bit BMP

Pipeline:

```text id="bmp41"
masked integer
 → extract channels
 → normalize
 → RGBA
```

---

## Stage 6 — Orientation Handling

Very important.

If height > 0:

```text id="bmp42"
bottom-up image
```

If height < 0:

```text id="bmp43"
top-down image
```

---

## Stage 7 — Row Padding Handling

Each row aligned to:

```text id="bmp44"
4-byte boundaries
```

Incorrect handling causes:

* shifted rows
* corrupted colors
* image skew

---

## Stage 8 — Alpha Reconstruction

32-bit BMP alpha behavior varies.

Possible cases:

* valid alpha
* unused alpha
* garbage alpha
* premultiplied alpha

Robust heuristics recommended.

---

## Stage 9 — Color Space Conversion

Usually:

```text id="bmp45"
assume sRGB
```

unless:

* ICC profile exists
* V5 metadata specifies otherwise

---

## Stage 10 — GPU Upload

Preferred formats:

```text id="bmp46"
RGBA8
```

Usually sufficient.

HDR workflows are uncommon for BMP.

---

# Streaming Decode Strategy

## Recommended for Huge BMPs

BMP supports:

```text id="bmp47"
very efficient scanline reading
```

Pipeline:

```text id="bmp48"
read scanline
 → decode
 → upload/process
 → discard
```

Advantages:

* low memory usage
* simple architecture

---

# Suggested Rust Architecture

## Module Layout

```text id="bmp49"
bmp/
 ├── file_header
 ├── dib_headers
 ├── compression
 ├── palettes
 ├── bitfields
 ├── scanlines
 ├── alpha
 ├── color
 ├── thumbnail
 ├── webp_export
 └── cache
```

---

# Recommended Initial Strategy

## Phase 1 — Common BMP Support

Implement:

* BI_RGB
* 24-bit BMP
* 32-bit BMP
* palette BMP
* WebP export

This covers:

```text id="bmp50"
most BMP files in the wild
```

---

## Phase 2 — Extended Compatibility

Add:

* RLE4
* RLE8
* BITFIELDS
* V4/V5 headers

---

## Phase 3 — Advanced Support

Implement:

* ICC workflows
* streaming decode
* GPU acceleration
* malformed file tolerance

---

# Performance Characteristics

## Decode Speed

BMP decode is:

```text id="bmp51"
extremely fast
```

because:

* minimal compression
* simple layout
* scanline organization

---

## File Size

BMP files are often:

```text id="bmp52"
very large
```

compared to:

* PNG
* WebP
* AVIF
* JPEG XL

especially:

* 24-bit uncompressed BMP

---

## Memory Usage

Memory usage primarily depends on:

* resolution
* bit depth

Not on decompression complexity.

---

# BMP Variant Ecosystem

## Windows BMP

Most common implementation.

---

## OS/2 BMP

Older format variants.

Potential differences:

* header structure
* palette handling
* compression semantics

Rare today.

---

# Transparency Considerations

## Alpha Ambiguity

Some BMP writers:

* ignore alpha
* misuse alpha
* leave alpha uninitialized

Recommended heuristic:

```text id="bmp53"
if alpha channel entirely zero
 → treat as opaque
```

in some workflows.

---

# Recommended Internal Pixel Formats

## Standard Processing

Use:

```text id="bmp54"
RGBA8
```

Usually sufficient.

---

## High Precision Conversion

Optional:

```text id="bmp55"
RGBA16
```

mostly useful when:

* converting to HDR pipelines
* preserving precision after transforms

---

# Recommended Cache Formats

## Thumbnail cache

```text id="bmp56"
WebP lossy
```

## Lossless cache

```text id="bmp57"
Lossless WebP
PNG
```

## GPU visualization

```text id="bmp58"
RGBA8
```

---

# Uncertain Points

## 1. Alpha Semantics

Older BMP files lack consistent alpha interpretation.

Possible behaviors:

* ignored alpha
* valid alpha
* garbage alpha

---

## 2. BITFIELDS Interpretation

Some encoders use:

* unusual masks
* undocumented layouts
* invalid masks

Robust validation required.

---

## 3. RLE Edge Cases

Malformed RLE streams may:

* overflow rows
* violate scanline boundaries
* contain invalid escape codes

---

## 4. Embedded JPEG/PNG BMP

Rare variants may embed:

* JPEG
* PNG

Support is inconsistent.

---

## 5. ICC Metadata Usage

BMP V5 supports color management but:

```text id="bmp59"
real-world usage is uncommon
```

---

# Other informations

## MIME Types

Common:

```text id="bmp60"
image/bmp
```

Also observed:

```text id="bmp61"
image/x-bmp
image/x-ms-bmp
```

---

# Typical Use Cases

BMP commonly appears in:

* Windows APIs
* retro software
* game textures
* screenshots
* embedded systems
* low-complexity graphics pipelines

---

# BMP vs PNG

## BMP Advantages

* simpler decode
* extremely fast parsing
* minimal CPU overhead

---

## BMP Disadvantages

* huge files
* weak compression
* limited metadata
* inconsistent alpha support

---

# Recommended Development Priorities

## Most Important

### 1. Correct row padding handling

Critical.

### 2. Bottom-up orientation support

Very important.

### 3. Bitfield support

Needed for compatibility.

### 4. Palette handling

Required for older BMPs.

### 5. Robust malformed-file handling

BMPs are frequently malformed.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="bmp62"
full decode + resize
```

because BMP decode cost is usually trivial.

## High-quality rendering

Use:

```text id="bmp63"
correct bitfield interpretation + proper alpha heuristics
```

## Native `.bmp` decoder implementation

Should be considered:

```text id="bmp64"
low-to-medium complexity
```

because BMP is:

* structurally simple
* lightly compressed
* scanline-oriented
* easy to stream

while the main implementation challenges are:

* historical DIB variants
* bitfield interpretation
* alpha inconsistency
* row padding correctness
* bottom-up storage semantics
* malformed legacy encoders.
