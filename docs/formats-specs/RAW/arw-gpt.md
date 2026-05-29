# Sony Alpha RAW (`.arw`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.arw`
* **Possible Origin**: Proprietary RAW format developed by Sony for Alpha-series digital cameras
* **Category**: RAW / Bayer Sensor Image Container
* **LibRaw Support**: Yes (extensive support across Sony Alpha generations)
* **FFMPEG Support**: No native advanced RAW pipeline; indirect decoding possible through LibRaw integration or image conversion layers
* **Rust alternative converters**:

  * `libraw-rs`
  * `rawloader`
  * `image`
  * `kamadak-exif`
  * `tiff`
  * External tools:

    * `dcraw`
    * `darktable-cli`
    * `rawtherapee-cli`
    * `Adobe DNG Converter`
    * `ImageMagick`
    * `Capture One`
    * `exiftool`

The `.arw` format is Sony’s primary RAW format used in:

* Sony Alpha mirrorless systems
* Sony DSLR systems
* Sony compact professional cameras
* Sony full-frame and APS-C cameras

The format evolved significantly over time:

* ARW 1.x
* ARW 2.x
* compressed/uncompressed generations
* lossy compressed generations
* modern lossless compressed variants

Internally, `.arw` is:

* TIFF-derived
* metadata-heavy
* Bayer-oriented
* generation-dependent

It shares architectural similarities with:

* TIFF/EP
* DNG
* CR2
* NEF
* RW2

but includes Sony-specific:

* compression schemes
* metadata
* MakerNotes
* sensor calibration pipelines

---

# File structure

## High-Level Container Layout

Typical `.arw` structure:

```text id="arw01"
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
| Sony MakerNotes      |
+----------------------+
```

Most `.arw` files behave as:

```text id="arw02"
proprietary TIFF-derived RAW containers
```

---

# TIFF Foundation

## TIFF Magic

Little-endian:

```hex id="arw03"
49 49 2A 00
```

Big-endian:

```hex id="arw04"
4D 4D 00 2A
```

Most `.arw` files are:

```text id="arw05"
little-endian
```

---

# Main Structural Components

## 1. TIFF Header

Contains:

* endian marker
* TIFF magic
* first IFD offset

Conceptual structure:

```c id="arw06"
struct TIFF_HEADER {
    uint16 endian;
    uint16 magic;
    uint32 first_ifd_offset;
}
```

---

## 2. Image File Directories (IFDs)

Contains:

* RAW offsets
* dimensions
* compression info
* preview references
* metadata references

Typical structure:

```c id="arw07"
struct IFD_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

Sony adds proprietary:

* MakerNote tags
* sensor metadata
* autofocus metadata
* lens correction metadata

---

## 3. RAW Sensor Data

Usually:

* Bayer CFA
* CMOS sensor data
* packed RAW bitstreams

Observed bit depths:

* 12-bit
* 14-bit
* occasionally 16-bit container alignment

Common CFA:

```text id="arw08"
RGGB
```

Some sensors may use:

* different Bayer arrangements
* phase-detection pixel masking

---

## 4. Embedded JPEG Preview

Most `.arw` files contain:

* full-size JPEG preview
* medium preview
* EXIF thumbnail

Preferred source for:

* thumbnails
* galleries
* fast previews

Advantages:

* extremely fast
* low CPU cost
* preserves camera rendering

Disadvantages:

* already tone-mapped
* clipped highlights
* reduced editing latitude

---

## 5. EXIF Metadata

Typical metadata:

* ISO
* aperture
* shutter speed
* focal length
* white balance
* orientation
* GPS
* lens identification

Rust recommendations:

```toml id="arw09"
kamadak-exif
little_exif
```

---

## 6. Sony MakerNotes

Contains extensive proprietary metadata.

Possible contents:

* autofocus points
* stabilization metadata
* lens correction
* face detection
* dynamic range optimizer settings
* picture profile settings
* focus distance
* sensor calibration

Sony MakerNotes are:

```text id="arw10"
large and complex
```

---

# RAW Sensor Characteristics

## Bayer RAW Pipeline

Sony `.arw` uses:

```text id="arw11"
single-layer Bayer CFA sensors
```

Requires:

* demosaicing
* white balance
* black level correction
* color transforms

---

# Compression Behavior

## ARW Compression Generations

One of the most important implementation considerations.

---

## 1. Uncompressed RAW

Available on some cameras.

Advantages:

* simpler decoding
* highest fidelity

Disadvantages:

* huge files

---

## 2. Lossy Compressed RAW

Used heavily in:

* older Alpha systems
* space-saving workflows

Sony’s older lossy compression:

* discards some precision
* uses delta-like encoding
* may introduce artifacts in extreme editing

Implementation complexity:

```text id="arw12"
moderate-to-high
```

---

## 3. Lossless Compressed RAW

Modern Sony cameras support:

* lossless compression
* better fidelity
* variable compression ratios

These formats are more complex.

---

# Packed Bayer Streams

## Common Packing Modes

Typical examples:

### 12-bit packed

```text id="arw13"
2 pixels = 24 bits = 3 bytes
```

### 14-bit packed

Complex bit alignment:

* non-byte-aligned extraction
* endian-sensitive parsing

Critical implementation area.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="arw14"
ARW
 └── parse TIFF IFD
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* low CPU usage
* preserves Sony rendering

Ideal for:

* galleries
* Tauri applications
* DAM systems
* file browsers

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="arw15"
160–512px
```

Often insufficient for modern interfaces.

---

## Tier 3 — Full RAW Decode

Required for:

* editing
* exposure recovery
* zoom rendering
* high-fidelity previews

Pipeline:

```text id="arw16"
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

```toml id="arw17"
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

```text id="arw18"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="arw19"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

Embedded previews are insufficient for:

* highlight recovery
* white balance editing
* HDR workflows
* shadow reconstruction
* full Sony sensor fidelity

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — TIFF Parsing

Read:

* IFD structures
* RAW offsets
* metadata
* compression identifiers
* preview references

Recommended crates:

```toml id="arw20"
binrw
nom
tiff
```

Preferred:

```toml id="arw21"
binrw
```

---

## Stage 2 — Compression Decode

Critical stage.

Must support:

* uncompressed RAW
* lossy Sony RAW
* lossless Sony RAW

Challenges:

* variable packing
* predictive coding
* bitstream alignment

---

## Stage 3 — RAW Extraction

Read:

* packed Bayer streams
* sensor calibration data
* black level metadata

Challenges:

* non-byte-aligned pixels
* masked PDAF pixels
* generation differences

---

## Stage 4 — Black Level Correction

Critical.

Without this:

* lifted shadows
* magenta shadows
* incorrect contrast

Sony sensors often require:

```text id="arw22"
precise black level handling
```

for correct rendering.

---

## Stage 5 — White Balance

Apply:

* camera multipliers
* neutral references
* sensor calibration matrices

---

## Stage 6 — Demosaicing

Recommended algorithms:

* AMaZE
* RCD
* DCB

For maximum quality:

```text id="arw23"
AMaZE
```

because Sony sensors preserve:

* very high detail density
* strong edge sharpness
* fine textures

---

## Stage 7 — PDAF Pixel Handling

Modern Sony sensors may contain:

```text id="arw24"
phase-detection autofocus pixels
```

Potential requirements:

* masking
* interpolation
* artifact correction

Important for:

* avoiding banding
* preventing grid artifacts

---

## Stage 8 — Lens Correction

Sony workflows heavily depend on:

* digital lens correction
* distortion compensation
* chromatic aberration correction

Possible implementation:

* Lensfun
* MakerNote interpretation
* Sony lens databases

---

## Stage 9 — Color Space Conversion

Recommended pipeline:

```text id="arw25"
camera RGB
 → XYZ
 → wide gamut working space
 → display transform
```

Recommended working spaces:

* ProPhoto RGB
* Rec2020
* ACEScg

---

## Stage 10 — Tone Mapping

Sony rendering often emphasizes:

* strong detail
* vivid contrast
* sharp local textures

Recommended:

* filmic
* ACES
* Reinhard

Avoid:

```text id="arw26"
overaggressive local contrast enhancement
```

which may create:

* halos
* harsh textures
* unnatural rendering

---

## Stage 11 — Noise Reduction

Sony sensors may exhibit:

* shadow chroma noise
* high-ISO texture noise
* banding in extreme recovery

Recommended:

* chroma-first denoise
* edge-aware luminance denoise

---

## Stage 12 — Sharpening

Sony images are naturally sharp.

Use:

* restrained sharpening
* detail-preserving enhancement

Avoid:

```text id="arw27"
oversharpening
```

which exaggerates:

* halos
* moiré
* demosaic artifacts

---

# Suggested Rust Architecture

## Module Layout

```text id="arw28"
arw/
 ├── tiff_parser
 ├── ifd
 ├── metadata
 ├── maker_notes
 ├── jpeg_extract
 ├── compression
 ├── raw_unpack
 ├── pdaf_handling
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

* TIFF parsing
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

## Phase 3 — Advanced Sony Support

Implement:

* lossy/lossless Sony compression
* PDAF correction
* lens correction
* GPU acceleration
* ROI decode

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

Sony `.arw` variants differ significantly across generations:

* compression modes
* metadata structure
* sensor layouts
* PDAF handling

LibRaw already handles:

* Bayer unpacking
* Sony compression
* metadata extraction
* black level correction

Recommended architecture:

```text id="arw29"
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
* minimal memory usage

---

## Full RAW Decode

Moderate-to-heavy workload because of:

* packed Bayer decoding
* Sony compression
* demosaicing
* lens corrections
* PDAF correction

Modern high-resolution Sony files can be:

```text id="arw30"
extremely computationally expensive
```

especially:

* 60+ MP sensors
* lossless RAW
* burst workflows

---

# Sony Rendering Characteristics

## Detail Rendering

Sony sensors emphasize:

* very high detail
* strong microcontrast
* aggressive sensor sharpness

Requires:

```text id="arw31"
high precision processing
```

to avoid:

* moiré
* zipper artifacts
* oversharpened rendering

---

## Dynamic Range

Sony sensors are known for:

* exceptional shadow recovery
* high DR performance
* strong low-light behavior

Requires:

```text id="arw32"
16-bit or float internal pipelines
```

Avoid:

```text id="arw33"
8-bit intermediate stages
```

until final export.

---

# Uncertain Points

## 1. Sony Lossy Compression Details

Some older Sony compression schemes remain partially undocumented.

Potential behaviors:

* precision truncation
* delta prediction
* nonlinear quantization

---

## 2. PDAF Metadata Semantics

Different sensor generations may expose:

* different PDAF maps
* undocumented correction metadata

---

## 3. MakerNote Semantics

Many Sony MakerNote tags remain undocumented.

Possible contents:

* autofocus tracking
* stabilization vectors
* AI scene metadata
* dynamic range optimizer behavior

---

## 4. Compression Generation Variability

Different Alpha generations differ significantly:

* ARW versioning
* packing alignment
* lossless support
* metadata organization

---

## 5. Exact Sony Rendering Pipeline

Sony software likely applies:

* proprietary tone curves
* custom color science
* AI-assisted processing
* lens-specific rendering

Perfect reproduction is difficult.

---

# Other informations

## MIME Type

Commonly observed:

```text id="arw34"
image/x-sony-arw
```

Not formally standardized.

---

# Cameras Using ARW

Examples:

* Sony Alpha A7 series
* Sony Alpha A1 series
* Sony Alpha A6000 series
* Sony RX professional compact series
* Sony SLT systems

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="arw35"
RGB16 linear
```

Preferred for HDR workflows:

```text id="arw36"
RGBA16F
```

Avoid:

```text id="arw37"
8-bit intermediate processing
```

until final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="arw38"
WebP lossy
```

## Editing cache

```text id="arw39"
16-bit TIFF
```

## GPU visualization

```text id="arw40"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Reliable Sony compression support

Core technical challenge.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. High precision pipeline

Critical for Sony DR performance.

### 5. PDAF-aware processing

Important for modern sensors.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="arw41"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="arw42"
LibRaw + high precision Bayer RAW pipeline
```

## Native `.arw` decoder implementation

Should be considered:

```text id="arw43"
medium-to-high complexity
```

because `.arw` combines:

* TIFF-derived structures
* multiple compression generations
* packed Bayer streams
* proprietary metadata
* PDAF-aware sensor layouts
* lens-correction-dependent rendering
* generation-specific decoding behavior.
