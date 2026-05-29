# Tagged Image File Format (`.tiff` / `.tif`) File Format Technical Specification

## Format Overview

* **Extension Name**:

  * `.tiff`
  * `.tif`
* **Possible Origin**: Developed by Aldus Corporation and Microsoft in the 1980s; later maintained by Adobe Systems
* **Category**: General-purpose Raster Image Container / Scientific Imaging / Archival Image Format
* **LibRaw Support**: Partial; TIFF itself is not RAW, but many RAW formats are TIFF-derived (DNG, CR2, NEF variants, ORF variants, etc.)
* **FFMPEG Support**: Yes (broad support for decoding and encoding many TIFF variants)
* **Rust alternative converters**:

  * `tiff`
  * `image`
  * `fast_image_resize`
  * `ravif`
  * `webp`
  * `kamadak-exif`
  * `libheif-rs`
  * External tools:

    * `ImageMagick`
    * `GraphicsMagick`
    * `ffmpeg`
    * `vips`
    * `tiffcp`
    * `tiffinfo`
    * `exiftool`

TIFF is one of the most important and flexible raster image container formats ever created.

It is widely used in:

* photography
* scientific imaging
* medical imaging
* archival storage
* publishing
* GIS systems
* VFX pipelines
* RAW camera formats
* texture pipelines

TIFF is:

```text id="tiff01"
a container format, not a codec
```

Meaning:

* the container can hold many image encodings
* multiple compression methods are supported
* many pixel formats are possible

TIFF supports:

* lossy compression
* lossless compression
* uncompressed images
* integer and floating point pixels
* multi-page images
* metadata
* thumbnails
* layers (limited ecosystems)
* tiles
* strips
* HDR imagery

---

# File structure

## High-Level Container Layout

Typical TIFF structure:

```text id="tiff02"
+----------------------+
| TIFF Header          |
+----------------------+
| IFD #0               |
+----------------------+
| Image Data           |
+----------------------+
| IFD #1               |
+----------------------+
| Thumbnail Data       |
+----------------------+
| Metadata Blocks      |
+----------------------+
| Additional IFDs      |
+----------------------+
```

TIFF is fundamentally based on:

```text id="tiff03"
tagged directory structures
```

called:

```text id="tiff04"
Image File Directories (IFDs)
```

---

# TIFF Header

## Standard TIFF Header

Little-endian:

```hex id="tiff05"
49 49 2A 00
```

Big-endian:

```hex id="tiff06"
4D 4D 00 2A
```

Structure:

```c id="tiff07"
struct TIFF_HEADER {
    uint16 endian;
    uint16 magic;
    uint32 first_ifd_offset;
}
```

---

# BigTIFF

## BigTIFF Extension

Classic TIFF has:

```text id="tiff08"
32-bit offsets
```

which limits file size.

BigTIFF introduces:

```text id="tiff09"
64-bit offsets
```

allowing:

* extremely large images
* scientific datasets
* massive textures
* microscopy datasets

BigTIFF magic:

```hex id="tiff10"
49 49 2B 00
```

or

```hex id="tiff11"
4D 4D 00 2B
```

---

# Image File Directories (IFDs)

## Core TIFF Concept

Each IFD contains:

* image metadata
* offsets
* compression info
* pixel layout

Conceptual structure:

```c id="tiff12"
struct IFD_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

BigTIFF expands:

* counts
* offsets
* directory sizes

to 64-bit.

---

# TIFF Data Types

Common TIFF types:

| Type     | Meaning         |
| -------- | --------------- |
| BYTE     | 8-bit unsigned  |
| ASCII    | String          |
| SHORT    | 16-bit unsigned |
| LONG     | 32-bit unsigned |
| RATIONAL | Fraction        |
| FLOAT    | IEEE float      |
| DOUBLE   | IEEE double     |

BigTIFF adds:

* LONG8
* SLONG8
* IFD8

---

# Core TIFF Tags

Important tags:

| Tag                       | Meaning            |
| ------------------------- | ------------------ |
| ImageWidth                | Width              |
| ImageLength               | Height             |
| BitsPerSample             | Bit depth          |
| Compression               | Compression method |
| PhotometricInterpretation | Color model        |
| StripOffsets              | Strip locations    |
| SamplesPerPixel           | Channel count      |
| RowsPerStrip              | Strip organization |
| TileWidth                 | Tile size          |
| TileOffsets               | Tile organization  |

---

# Image Storage Modes

## 1. Strips

Traditional TIFF layout.

Image divided into:

```text id="tiff13"
horizontal strips
```

Advantages:

* simpler implementation
* sequential reading

Disadvantages:

* inefficient random access

---

## 2. Tiles

Modern high-performance layout.

Image divided into:

```text id="tiff14"
2D rectangular tiles
```

Advantages:

* efficient zoom rendering
* partial decoding
* GPU-friendly

Essential for:

* huge images
* GIS
* scientific imaging
* VFX textures

---

# Compression Methods

TIFF supports many compression methods.

---

## 1. Uncompressed

Compression tag:

```text id="tiff15"
1
```

Advantages:

* simplest decoding
* maximum compatibility

Disadvantages:

* huge files

---

## 2. LZW

Compression tag:

```text id="tiff16"
5
```

Lossless dictionary compression.

Very common.

---

## 3. Deflate / ZIP

Compression tag:

```text id="tiff17"
8
```

or:

```text id="tiff18"
32946
```

Lossless.

Excellent balance between:

* speed
* compression ratio
* compatibility

Recommended default.

---

## 4. PackBits

Compression tag:

```text id="tiff19"
32773
```

Simple RLE compression.

Fast but weak compression ratio.

---

## 5. JPEG-in-TIFF

Compression tag:

```text id="tiff20"
7
```

Stores JPEG-compressed tiles/strips.

Advantages:

* smaller files

Disadvantages:

* lossy
* implementation complexity

---

## 6. CCITT Fax Compression

Used for:

* monochrome scans
* documents

Not relevant for photography workflows.

---

## 7. Modern Extensions

Some ecosystems support:

* JPEG2000
* WebP-in-TIFF
* ZSTD-in-TIFF
* LERC
* JPEG XL proposals

Support is fragmented.

---

# Pixel Formats

TIFF supports enormous flexibility.

---

## Integer Formats

Common:

* 8-bit
* 10-bit
* 12-bit
* 14-bit
* 16-bit
* 32-bit integer

---

## Floating Point Formats

Supports:

* 16F
* 32F
* 64F

Used in:

* HDR
* VFX
* scientific imaging

---

## Channel Layouts

Possible:

* Grayscale
* RGB
* RGBA
* CMYK
* Lab
* YCbCr
* multispectral
* arbitrary channel counts

---

# PhotometricInterpretation

Critical TIFF tag.

Examples:

| Value | Meaning     |
| ----- | ----------- |
| 0     | WhiteIsZero |
| 1     | BlackIsZero |
| 2     | RGB         |
| 5     | CMYK        |
| 6     | YCbCr       |
| 8     | CIELab      |

---

# Metadata Support

TIFF supports:

* EXIF
* IPTC
* XMP
* ICC profiles
* GPS metadata

---

# EXIF Support

EXIF itself is TIFF-derived.

Common EXIF block:

```text id="tiff21"
APP1 → TIFF structure
```

Rust recommendations:

```toml id="tiff22"
kamadak-exif
little_exif
```

---

# ICC Color Profiles

TIFF supports embedded:

* ICC profiles
* wide gamut workflows
* print color management

Critical for:

* professional imaging
* archival fidelity

---

# Multi-Page TIFF

TIFF can contain:

* multiple images
* animation-like stacks
* document pages
* microscopy slices

Common in:

* scanning
* scientific imaging
* medical imaging

---

# RAW-in-TIFF Ecosystems

Many RAW formats are TIFF-derived:

* DNG
* CR2
* some NEF
* ORF
* RW2
* ARW variants

TIFF knowledge is foundational for:

```text id="tiff23"
RAW decoder development
```

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded Thumbnail Extraction

Many TIFF files contain:

* reduced-resolution IFDs
* JPEG previews
* pyramid levels

Pipeline:

```text id="tiff24"
TIFF
 └── parse IFD hierarchy
      └── locate preview image
           └── decode preview
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* low CPU usage

---

## Tier 2 — Pyramid Level Extraction

Scientific/VFX TIFFs may contain:

* mip chains
* resolution pyramids

Preferred for:

* huge images
* zoom systems

---

## Tier 3 — Full Decode

Required when:

* no preview exists
* exact fidelity required
* scientific correctness matters

Pipeline:

```text id="tiff25"
TIFF decode
 → decompress strips/tiles
 → color transform
 → resize
 → WebP
```

---

# Recommended Rust Thumbnail Pipeline

## Suggested Crates

```toml id="tiff26"
tiff
image
fast_image_resize
webp
rayon
kamadak-exif
```

---

## Ideal WebP Settings

### Gallery thumbnails

```text id="tiff27"
Quality: 70–85
Lossy WebP
```

### Archival previews

```text id="tiff28"
Quality: 90–100
Lossless or near-lossless
```

---

# Strategy for Visualization

## Important Principle

TIFF visualization depends heavily on:

* compression type
* pixel format
* color space
* metadata interpretation

TIFF is:

```text id="tiff29"
not a single rendering pipeline
```

---

# Recommended Visualization Pipeline

## Stage 1 — Header Detection

Identify:

* TIFF vs BigTIFF
* endian mode
* first IFD

---

## Stage 2 — IFD Parsing

Read:

* dimensions
* compression
* pixel layout
* tile/strip organization
* metadata

Recommended crates:

```toml id="tiff30"
binrw
nom
tiff
```

Preferred:

```toml id="tiff31"
tiff
```

for standard workflows.

---

## Stage 3 — Compression Decode

Dispatch decoder based on:

```text id="tiff32"
Compression tag
```

Examples:

* LZW
* Deflate
* JPEG
* PackBits

---

## Stage 4 — Tile/Strip Assembly

Reconstruct:

* full raster image
* tile maps
* strip sequences

For huge TIFFs:

```text id="tiff33"
avoid full-image allocation
```

when possible.

Use:

* tiled decode
* region decode
* streaming decode

---

## Stage 5 — Color Interpretation

Critical.

Interpret:

* photometric model
* ICC profile
* gamma
* transfer curves

---

## Stage 6 — Bit Depth Handling

TIFF frequently uses:

* 16-bit
* float
* HDR

Recommended internal formats:

```text id="tiff34"
RGB16
RGBA16
RGBA16F
RGBA32F
```

Avoid:

```text id="tiff35"
8-bit intermediate conversion
```

until final output.

---

## Stage 7 — Color Space Conversion

Recommended pipeline:

```text id="tiff36"
source color space
 → PCS/XYZ
 → working space
 → display transform
```

Recommended working spaces:

* ProPhoto RGB
* Rec2020
* ACEScg

---

## Stage 8 — Tone Mapping

Required for:

* HDR TIFF
* floating point TIFF
* scientific visualization

Recommended:

* filmic
* ACES
* Reinhard

---

## Stage 9 — GPU Upload

Preferred formats:

```text id="tiff37"
RGBA16F
RGBA32F
```

for:

* HDR
* scientific rendering
* zoom systems

---

# Tile-Based Rendering Strategy

## Recommended for Large TIFFs

Do not decode entire image.

Instead:

```text id="tiff38"
visible viewport
    ↓
identify required tiles
    ↓
decode only visible tiles
    ↓
GPU upload
```

Critical for:

* GIS
* microscopy
* gigapixel images
* texture streaming

---

# Suggested Rust Architecture

## Module Layout

```text id="tiff39"
tiff/
 ├── header
 ├── ifd
 ├── tags
 ├── compression
 ├── strips
 ├── tiles
 ├── metadata
 ├── exif
 ├── icc
 ├── thumbnail
 ├── color_pipeline
 ├── tone_mapping
 ├── webp_export
 └── cache
```

---

# Recommended Initial Strategy

## Phase 1 — Standard TIFF Support

Implement:

* baseline TIFF
* LZW
* Deflate
* RGB8/RGB16
* thumbnail extraction

---

## Phase 2 — Advanced TIFF

Add:

* tiled TIFF
* BigTIFF
* floating point TIFF
* JPEG-in-TIFF
* ICC workflows

---

## Phase 3 — High-End Pipelines

Implement:

* ROI decode
* GPU acceleration
* HDR workflows
* scientific visualization
* multithreaded tile streaming

---

# Performance Characteristics

## Small TIFFs

Easy and fast.

---

## Huge TIFFs

Potentially extremely expensive because of:

* gigantic resolutions
* float channels
* tiled storage
* decompression overhead

Memory usage may exceed:

```text id="tiff40"
multiple gigabytes
```

for scientific images.

---

# BigTIFF Considerations

Critical for:

* VFX
* GIS
* microscopy
* HDR pipelines

Requires:

```text id="tiff41"
64-bit-safe implementations
```

throughout parser architecture.

---

# Scientific Imaging Considerations

TIFF is heavily used in:

* astronomy
* microscopy
* medical imaging
* satellite imagery

Possible characteristics:

* > 4 channels
* float channels
* unusual photometric models
* massive datasets

Avoid assumptions like:

```text id="tiff42"
RGB8 only
```

---

# TIFF and RAW Development

TIFF expertise directly helps with:

* DNG
* CR2
* NEF variants
* ORF
* RW2
* proprietary RAW formats

because many are:

```text id="tiff43"
TIFF-derived containers
```

---

# Uncertain Points

## 1. Nonstandard Vendor Extensions

Many applications create:

* invalid TIFFs
* partially compliant TIFFs
* undocumented custom tags

Robust parsers must tolerate:

* malformed offsets
* invalid tag ordering
* unusual metadata

---

## 2. JPEG-in-TIFF Variants

Different encoders may implement:

* old-style JPEG TIFF
* new-style JPEG TIFF
* incompatible layouts

Implementation complexity is moderate-to-high.

---

## 3. Modern Compression Extensions

Support for:

* WebP
* ZSTD
* JPEG XL
* AVIF

inside TIFF is fragmented.

---

## 4. Floating Point TIFF Variants

HDR/scientific TIFFs may contain:

* NaN
* Infinity
* negative luminance
* unusual transfer functions

---

## 5. Multi-IFD Semantics

Different software interprets:

* thumbnails
* reduced-resolution images
* pyramids

differently.

---

# Other informations

## MIME Types

Common:

```text id="tiff44"
image/tiff
```

BigTIFF may still use:

```text id="tiff45"
image/tiff
```

despite structural differences.

---

# TIFF Variants

Important variants:

* Baseline TIFF
* BigTIFF
* GeoTIFF
* OME-TIFF
* TIFF/EP
* DNG
* pyramidal TIFF

---

# GeoTIFF

Adds:

* GIS metadata
* map projection
* coordinate systems

Used in:

* satellite imagery
* terrain data

---

# OME-TIFF

Scientific microscopy extension.

Supports:

* multidimensional imaging
* time series
* channel stacks

---

# Recommended Internal Pixel Formats

## Standard Processing

Use:

```text id="tiff46"
RGB16
RGBA16
```

## HDR / Scientific

Use:

```text id="tiff47"
RGBA16F
RGBA32F
```

Avoid:

```text id="tiff48"
premature 8-bit conversion
```

---

# Recommended Cache Formats

## Thumbnail cache

```text id="tiff49"
WebP lossy
```

## Editing cache

```text id="tiff50"
16-bit TIFF
```

## HDR cache

```text id="tiff51"
EXR or float TIFF
```

---

# Recommended Development Priorities

## Most Important

### 1. Reliable IFD parser

Foundation of all TIFF handling.

### 2. Compression abstraction layer

Critical for extensibility.

### 3. Tile support

Essential for large images.

### 4. ICC-aware color pipeline

Critical for fidelity.

### 5. Streaming decode architecture

Avoid huge memory usage.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="tiff52"
embedded previews or reduced IFD levels
```

## High-quality rendering

Use:

```text id="tiff53"
16-bit or floating-point color-managed pipelines
```

## Native TIFF decoder implementation

Should be considered:

```text id="tiff54"
medium-to-high complexity
```

because TIFF is:

* extremely flexible
* loosely constrained
* heavily extended by vendors
* compression-pluggable
* capable of many pixel layouts
* widely abused by nonstandard implementations.
