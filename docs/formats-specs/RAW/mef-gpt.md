# Mamiya RAW (`.mef`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.mef`
* **Possible Origin**: Proprietary RAW format developed for Mamiya digital camera systems and some Phase One/Mamiya integrated medium-format workflows
* **Category**: RAW / Medium-Format Sensor Data Container
* **LibRaw Support**: Yes (supported for multiple Mamiya camera models)
* **FFMPEG Support**: No native RAW decoder; indirect support through LibRaw or external RAW conversion tools
* **Rust alternative converters**:

  * `libraw-rs`
  * `rawloader`
  * `image`
  * `kamadak-exif`
  * External tools:

    * `dcraw`
    * `darktable-cli`
    * `rawtherapee-cli`
    * `Capture One`
    * `ImageMagick`
    * `exiftool`

The `.mef` format is associated primarily with:

* Mamiya ZD systems
* Mamiya medium-format digital cameras
* some Mamiya/Phase One integrated ecosystems

The format was designed for:

* medium-format image fidelity
* high bit-depth RAW workflows
* studio-oriented photography
* high dynamic range preservation

Compared to smaller-sensor consumer RAW formats:

* `.mef` files are significantly larger
* tonal precision is prioritized
* metadata often contains calibration-oriented information

Internally, `.mef` behaves similarly to:

```text id="mef01"
TIFF/EP-derived RAW containers
```

with proprietary extensions.

---

# File structure

## High-Level Container Layout

Typical `.mef` structure:

```text id="mef02"
+----------------------+
| TIFF Header          |
+----------------------+
| IFD Directory Tree   |
+----------------------+
| RAW Sensor Data      |
+----------------------+
| Embedded JPEG Preview|
+----------------------+
| EXIF Metadata        |
+----------------------+
| MakerNotes           |
+----------------------+
```

The structure is strongly TIFF-like.

Most implementations can treat `.mef` as:

```text id="mef03"
a proprietary TIFF-based RAW format
```

---

# TIFF Foundation

## TIFF Magic

Little-endian:

```hex id="mef04"
49 49 2A 00
```

Big-endian:

```hex id="mef05"
4D 4D 00 2A
```

Most observed `.mef` files are:

```text id="mef06"
little-endian
```

---

# Main Structural Components

## 1. TIFF Header

Contains:

* endian marker
* TIFF magic
* offset to first IFD

Conceptual structure:

```c id="mef07"
struct TIFF_HEADER {
    uint16 endian;
    uint16 magic;
    uint32 first_ifd_offset;
}
```

---

## 2. IFD (Image File Directory)

The core organizational structure.

Contains:

* RAW offsets
* image dimensions
* compression information
* preview references
* metadata references

Typical structure:

```c id="mef08"
struct IFD_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

---

## 3. RAW Sensor Data

Usually:

* Bayer CFA
* medium-format CCD sensor data
* high bit depth

Observed bit depths:

* 12-bit
* 14-bit
* sometimes padded to 16-bit containers

Common CFA:

```text id="mef09"
RGGB
```

Sensor characteristics:

* smooth tonal transitions
* strong color depth
* large dynamic range

---

## 4. Embedded JPEG Preview

Most `.mef` files contain:

* medium/high-resolution JPEG preview
* camera-rendered image
* embedded thumbnail

This is the preferred source for:

* thumbnails
* galleries
* fast previews

Advantages:

* very fast extraction
* low CPU usage
* preserves camera rendering

Disadvantages:

* already tone-mapped
* clipped highlights
* reduced RAW editing latitude

---

## 5. EXIF Metadata

Typical metadata:

* ISO
* aperture
* shutter speed
* focal length
* orientation
* timestamp
* white balance

Rust recommendations:

```toml id="mef10"
kamadak-exif
little_exif
```

---

## 6. MakerNotes

Contains proprietary camera metadata.

Possible contents:

* sensor calibration
* lens metadata
* color matrices
* black level values
* workflow metadata

Documentation is limited.

---

# RAW Sensor Characteristics

## Bayer RAW Pipeline

Unlike Foveon:

```text id="mef11"
1 sensor pixel = 1 color component
```

Requires:

* demosaicing
* white balance
* color conversion

---

# Compression Behavior

## Observed Compression Modes

### 1. Uncompressed RAW

Common in older medium-format systems.

Advantages:

* easier parsing
* lower decode complexity

Disadvantages:

* very large files

---

### 2. Lossless Compression

Observed in some variants.

Likely:

* predictive encoding
* packed Bayer streams
* delta compression

Usually abstracted by LibRaw.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="mef12"
MEF
 └── parse TIFF IFD
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* low CPU cost
* preserves camera rendering

Ideal for:

* galleries
* Tauri applications
* lazy loading
* DAM systems

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="mef13"
160–512px
```

Often insufficient for modern interfaces.

---

## Tier 3 — Full RAW Decode

Required for:

* editing
* exposure recovery
* high-fidelity previews
* zoom rendering

Pipeline:

```text id="mef14"
RAW decode
 → unpack Bayer
 → black level correction
 → white balance
 → demosaic
 → color transform
 → tone mapping
 → resize
 → WebP
```

---

# Recommended Rust Thumbnail Pipeline

## Suggested Crates

```toml id="mef15"
image
jpeg-decoder
fast_image_resize
webp
rayon
kamadak-exif
```

---

## Ideal WebP Settings

### Gallery thumbnails

```text id="mef16"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="mef17"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

Embedded JPEG previews are insufficient for:

* highlight recovery
* full dynamic range
* white balance adjustment
* maximum medium-format fidelity

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — TIFF Parsing

Read:

* IFD tables
* RAW offsets
* metadata
* preview locations

Recommended crates:

```toml id="mef18"
binrw
nom
tiff
```

Preferred:

```toml id="mef19"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* Bayer RAW
* packed bit streams
* compression blocks

Challenges:

* packed pixel decoding
* large memory usage
* variant-specific layouts

---

## Stage 3 — Black Level Correction

Critical.

Without this:

* lifted blacks
* shadow artifacts
* incorrect tonal response

---

## Stage 4 — White Balance

Apply:

* camera multipliers
* calibration matrices
* neutral references

---

## Stage 5 — Demosaicing

Recommended algorithms:

* AMaZE
* RCD
* DCB

For maximum quality:

```text id="mef20"
AMaZE or RCD
```

because medium-format imagery preserves:

* subtle gradients
* fine textures
* smooth transitions

---

## Stage 6 — Color Space Conversion

Recommended pipeline:

```text id="mef21"
camera RGB
 → XYZ
 → wide gamut working space
 → display transform
```

Recommended internal spaces:

* ProPhoto RGB
* Rec2020
* ACEScg

---

## Stage 7 — Tone Mapping

Medium-format sensors preserve:

* smooth highlights
* wide dynamic range
* subtle tonal rolloff

Recommended:

* filmic
* ACES
* Reinhard

Avoid:

```text id="mef22"
aggressive contrast curves
```

which destroy medium-format rendering characteristics.

---

## Stage 8 — Noise Reduction

Usually lighter than consumer sensors.

Recommended:

* chroma-first denoise
* preserve luminance texture

---

## Stage 9 — Sharpening

Use:

* restrained sharpening
* low-strength/high-radius approaches

Avoid:

```text id="mef23"
aggressive edge sharpening
```

which damages natural medium-format rendering.

---

# Suggested Rust Architecture

## Module Layout

```text id="mef24"
mef/
 ├── tiff_parser
 ├── ifd
 ├── metadata
 ├── maker_notes
 ├── jpeg_extract
 ├── raw_extract
 ├── demosaic
 ├── color_pipeline
 ├── tone_mapping
 ├── thumbnail
 ├── webp_export
 └── cache
```

---

# Recommended Initial Strategy

## Phase 1 — Fast Practical Support

Implement:

* TIFF parser
* embedded JPEG extraction
* EXIF parsing
* WebP export

This gives:

* immediate usability
* fast previews
* low implementation complexity

---

## Phase 2 — RAW Rendering

Add:

* Bayer unpacking
* demosaicing
* white balance
* tone mapping
* color transforms

---

## Phase 3 — High-End Fidelity

Implement:

* camera-specific rendering
* calibration handling
* GPU acceleration
* tiled rendering
* ROI decode

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

`.mef` variants may differ by:

* sensor generation
* packing methods
* metadata layouts
* compression schemes

LibRaw already handles:

* Bayer unpacking
* CFA interpretation
* metadata extraction
* black level correction

Recommended architecture:

```text id="mef25"
Rust frontend
    ↓
LibRaw FFI
    ↓
16-bit linear RGB
```

---

# Performance Characteristics

## Embedded JPEG Path

Very fast:

* ideal for galleries
* low memory usage

---

## Full RAW Decode

Heavy workload because of:

* medium-format resolutions
* high bit depth
* demosaicing cost
* large memory bandwidth

But generally simpler than:

* X3F
* CR3
* heavily compressed RAF

---

# Medium Format Rendering Characteristics

## Dynamic Range

Medium-format systems preserve:

* smooth highlights
* deep shadows
* subtle gradients

Requires:

```text id="mef26"
high precision internal processing
```

Avoid:

```text id="mef27"
8-bit intermediate pipelines
```

until final export.

---

## Color Science

Mamiya rendering emphasizes:

* natural tones
* smooth gradients
* studio-oriented neutrality

Exact replication may require:

* empirical profiling
* proprietary matrices
* Capture One compatibility testing

---

# Uncertain Points

## 1. Compression Variants

Different generations may use:

* different predictive coders
* different Bayer packing layouts

---

## 2. MakerNote Semantics

Many proprietary tags remain undocumented.

Possible contents:

* calibration data
* lens corrections
* rendering hints

---

## 3. Exact Sensor Calibration Behavior

Some rendering characteristics may depend on:

* hidden calibration metadata
* proprietary color transforms

---

## 4. Tile/Strip Variability

Some `.mef` files may vary between:

* strip-based layouts
* tile-based storage

Requires empirical testing.

---

## 5. Camera Generation Differences

Older CCD systems and newer integrated systems may differ significantly:

* noise behavior
* black level handling
* metadata structure

---

# Other informations

## MIME Type

Commonly observed:

```text id="mef28"
image/x-mamiya-mef
```

Not formally standardized.

---

# Cameras Using MEF

Examples:

* Mamiya ZD
* Mamiya medium-format systems
* some Mamiya/Phase One integrated workflows

Often interoperable with:

```text id="mef29"
Capture One
```

and other professional RAW workflows.

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="mef30"
RGB16 linear
```

Preferred for HDR workflows:

```text id="mef31"
RGBA16F
```

Avoid:

```text id="mef32"
8-bit intermediate processing
```

until final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="mef33"
WebP lossy
```

## Editing cache

```text id="mef34"
16-bit TIFF
```

## GPU visualization

```text id="mef35"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Reliable TIFF parser

Foundation for all processing.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. High precision pipeline

Critical for medium-format fidelity.

### 5. Accurate demosaicing

Essential for preserving fine detail.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="mef36"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="mef37"
LibRaw + high precision RAW pipeline
```

## Native `.mef` decoder implementation

Should be considered:

```text id="mef38"
medium complexity
```

because `.mef` combines:

* TIFF-derived RAW structures
* medium-format Bayer RAW
* proprietary MakerNotes
* possible compression variants
* calibration-dependent rendering behavior.
