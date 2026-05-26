# Hasselblad Flexible File Format (`.fff`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.fff`
* **Possible Origin**: Proprietary RAW container developed by Hasselblad for medium-format digital camera systems and digital backs
* **Category**: RAW / Medium-Format Sensor Data Container
* **LibRaw Support**: Yes (good support for many Hasselblad systems)
* **FFMPEG Support**: No native RAW decoder; indirect support possible through external RAW processing pipelines
* **Rust alternative converters**:

  * `libraw-rs`
  * `rawloader`
  * `image`
  * `kamadak-exif`
  * External tools:

    * `dcraw`
    * `darktable-cli`
    * `rawtherapee-cli`
    * `Phocus`
    * `ImageMagick`
    * `exiftool`

The `.fff` format stands for:

```text id="fff01"
Flexible File Format
```

It is primarily associated with:

* Hasselblad H-series systems
* Hasselblad CFV digital backs
* medium-format CCD and CMOS workflows

The format was designed for:

* extremely high image fidelity
* studio workflows
* high bit-depth preservation
* medium-format color precision

Compared to `.3fr`, `.fff` is:

* generally more advanced
* more metadata-rich
* more tightly integrated with Hasselblad Phocus workflows

Internally, `.fff` is partially TIFF-derived but includes substantial proprietary structures and MakerNote extensions.

---

# File structure

## High-Level Container Layout

Typical `.fff` structure:

```text id="fff02"
+----------------------+
| Header               |
+----------------------+
| TIFF-like Directories|
+----------------------+
| RAW Sensor Data      |
+----------------------+
| Embedded JPEG Preview|
+----------------------+
| Metadata Blocks      |
+----------------------+
| MakerNotes           |
+----------------------+
| Calibration Data     |
+----------------------+
```

The format behaves similarly to:

```text id="fff03"
TIFF/EP hybrid RAW containers
```

but with Hasselblad-specific organizational logic.

---

# Container Characteristics

Unlike fully standardized TIFF:

* `.fff` may contain proprietary tags
* offsets may not follow standard TIFF assumptions
* metadata blocks can be camera-generation dependent

However:

* many parsers treat it as TIFF-like internally
* LibRaw successfully abstracts most complexity

---

# Main Structural Components

## 1. File Header

Contains:

* format identifier
* version information
* directory offsets
* metadata references

Observed behavior suggests:

* little-endian storage is common
* TIFF-like organization

Possible conceptual structure:

```c id="fff04"
struct FFF_HEADER {
    uint32 magic;
    uint32 version;
    uint32 directory_offset;
}
```

Exact layouts vary between camera generations.

---

## 2. TIFF-Like Directory Structure

Contains:

* RAW image offsets
* image dimensions
* compression metadata
* preview image offsets
* metadata references

Conceptually similar to TIFF IFDs:

```c id="fff05"
struct FFF_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

But proprietary tags are heavily used.

---

## 3. RAW Sensor Data

Usually:

* Bayer CFA
* medium-format CCD or CMOS
* high bit depth
* very large image dimensions

Observed bit depths:

* 12-bit
* 14-bit
* 16-bit

Common CFA:

```text id="fff06"
RGGB
```

Sensor characteristics:

* exceptional dynamic range
* smooth tonal gradients
* very high color precision

---

## 4. Embedded JPEG Preview

Most `.fff` files contain:

* high-quality JPEG preview
* medium or large preview image
* camera-rendered tone mapping

This is the preferred source for:

* thumbnails
* gallery browsing
* instant previews

Advantages:

* very fast
* preserves Hasselblad rendering
* low CPU usage

Disadvantages:

* baked tone curves
* clipped highlights
* reduced RAW editing latitude

---

## 5. Metadata Blocks

Contains:

* EXIF
* camera configuration
* lens data
* white balance
* calibration parameters

Rust recommendations:

```toml id="fff07"
kamadak-exif
little_exif
```

---

## 6. Hasselblad MakerNotes

Critical proprietary section.

Likely contains:

* Phocus calibration
* lens corrections
* sensor calibration
* color matrices
* black level data
* tethering workflow metadata

Documentation is limited.

Different Hasselblad generations likely use:

* incompatible metadata layouts
* different tag semantics

---

## 7. Calibration Data

One of the most important differences from consumer RAW formats.

Medium-format systems frequently include:

* detailed sensor calibration
* lens shading correction
* precision color transforms

These are critical for:

* accurate rendering
* smooth gradients
* professional color fidelity

---

# RAW Sensor Characteristics

## Medium Format Bayer Pipeline

Unlike Foveon:

```text id="fff08"
1 sensor pixel = 1 color sample
```

Requires:

* demosaicing
* white balance
* color transforms

---

# Compression Behavior

## Observed Compression Modes

### 1. Uncompressed RAW

Common in older systems.

Advantages:

* simpler decoding
* maximum fidelity

Disadvantages:

* huge file sizes

---

### 2. Lossless Compression

Observed in newer systems.

Likely:

* predictive compression
* packed RAW encoding
* delta compression

Usually supported transparently through LibRaw.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="fff09"
FFF
 └── parse metadata/directories
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* preserves Hasselblad rendering
* minimal CPU cost

Ideal for:

* galleries
* Tauri applications
* DAM systems
* lazy loading

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="fff10"
160–512px
```

Often insufficient for modern UIs.

---

## Tier 3 — Full RAW Decode

Required for:

* zoom rendering
* editing
* exposure recovery
* high-fidelity previews

Pipeline:

```text id="fff11"
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

```toml id="fff12"
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

```text id="fff13"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="fff14"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

The embedded JPEG preview is insufficient for:

* highlight recovery
* white balance adjustments
* full dynamic range extraction
* medium-format detail preservation

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — Container Parsing

Read:

* TIFF-like structures
* RAW offsets
* metadata blocks
* preview references

Recommended crates:

```toml id="fff15"
binrw
nom
```

Preferred:

```toml id="fff16"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* Bayer RAW
* packed bit streams
* compression blocks

Challenges:

* very large files
* strip/tile layouts
* proprietary metadata dependencies

Files may exceed:

```text id="fff17"
100–500 MB
```

depending on sensor generation.

---

## Stage 3 — Black Level Correction

Critical.

Without this:

* lifted blacks
* incorrect tonal response
* shadow artifacts

---

## Stage 4 — White Balance

Apply:

* camera multipliers
* neutral references
* calibration matrices

---

## Stage 5 — Demosaicing

Recommended algorithms:

* AMaZE
* RCD
* DCB

For maximum quality:

```text id="fff18"
AMaZE or RCD
```

because medium-format imagery benefits heavily from:

* edge preservation
* smooth gradients
* subtle texture recovery

---

## Stage 6 — Lens Correction

Important for:

* medium-format lenses
* edge shading
* chromatic aberration
* geometric correction

Possible implementation paths:

* Phocus profile emulation
* Lensfun integration
* empirical calibration

---

## Stage 7 — Color Space Conversion

Recommended pipeline:

```text id="fff19"
camera RGB
 → XYZ
 → ProPhoto RGB / Rec2020
 → display transform
```

Recommended internal working spaces:

* ProPhoto RGB
* ACEScg
* Rec2020

---

## Stage 8 — Tone Mapping

Medium-format sensors frequently preserve:

* massive highlight detail
* extremely smooth tonal rolloff

Recommended:

* filmic
* ACES
* Reinhard

Avoid:

```text id="fff20"
aggressive contrast curves
```

which destroy medium-format tonal rendering.

---

## Stage 9 — Noise Reduction

Usually lighter than consumer cameras.

Recommended:

* chroma-first denoise
* preserve microcontrast

---

## Stage 10 — Sharpening

Use:

* low-strength
* high-radius sharpening

Avoid:

```text id="fff21"
oversharpening
```

which destroys the characteristic medium-format appearance.

---

# Suggested Rust Architecture

## Module Layout

```text id="fff22"
fff/
 ├── parser
 ├── directories
 ├── metadata
 ├── maker_notes
 ├── jpeg_extract
 ├── raw_extract
 ├── demosaic
 ├── lens_correction
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

* directory parser
* embedded JPEG extraction
* EXIF parsing
* WebP export

This gives:

* immediate practical utility
* stable previews
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

* Hasselblad-specific rendering
* Phocus-like color science
* GPU acceleration
* tiled rendering
* region-of-interest decode

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

`.fff` contains:

* proprietary metadata
* possible compression variants
* camera-generation differences

LibRaw already handles:

* Bayer unpacking
* CFA interpretation
* black level correction
* metadata parsing

Recommended architecture:

```text id="fff23"
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

* suitable for real-time browsing
* ideal for galleries

---

## Full RAW Decode

Heavy workload because of:

* enormous resolutions
* high bit depth
* complex demosaicing
* large memory usage

But usually simpler than:

* X3F
* CR3
* compressed RAF

---

# Medium Format Rendering Characteristics

## Dynamic Range

Hasselblad systems preserve:

* smooth highlights
* deep shadows
* natural gradients

Requires:

```text id="fff24"
high precision internal pipelines
```

Avoid:

```text id="fff25"
8-bit intermediate processing
```

until final export.

---

## Color Science

Hasselblad rendering emphasizes:

* natural tones
* neutral colors
* subtle transitions
* skin fidelity

Exact reproduction likely depends on:

* proprietary Phocus transforms
* hidden calibration data
* camera-specific matrices

---

# Uncertain Points

## 1. Exact Container Specification

`.fff` is only partially documented publicly.

Some structures remain proprietary.

---

## 2. Compression Variants

Different generations may use:

* different predictive encoders
* different packed RAW schemes

---

## 3. MakerNote Semantics

Many proprietary tags remain undocumented.

Possible contents:

* lens profiles
* calibration matrices
* tethering metadata
* Phocus workflow hints

---

## 4. Phocus Rendering Pipeline

Exact Hasselblad rendering likely involves:

* proprietary highlight reconstruction
* proprietary color transforms
* custom tone curves

Perfect replication is difficult.

---

## 5. Tile/Strip Organization

Some files may vary between:

* strip-based layouts
* tile-based layouts

Needs empirical validation.

---

# Other informations

## MIME Type

Commonly observed:

```text id="fff26"
image/x-hasselblad-fff
```

Not formally standardized.

---

# Cameras Using FFF

Examples:

* Hasselblad H-series
* Hasselblad CFV backs
* newer Hasselblad medium-format systems

Often integrated with:

```text id="fff27"
Phocus
```

workflow ecosystems.

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="fff28"
RGB16 linear
```

Preferred for HDR:

```text id="fff29"
RGBA16F
```

Avoid:

```text id="fff30"
8-bit pipelines
```

until final display/export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="fff31"
WebP lossy
```

## Editing cache

```text id="fff32"
16-bit TIFF
```

## GPU visualization

```text id="fff33"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Reliable parser

Foundation for all processing.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. High precision processing

Essential for medium-format fidelity.

### 5. Accurate color pipeline

Critical for Hasselblad rendering quality.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="fff34"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="fff35"
LibRaw + high precision medium-format RAW pipeline
```

## Native `.fff` decoder implementation

Should be considered:

```text id="fff36"
medium-to-high complexity
```

because `.fff` combines:

* TIFF-like structures
* proprietary metadata
* medium-format Bayer RAW
* calibration-heavy workflows
* possible compression variants
* Phocus-integrated rendering behavior.
