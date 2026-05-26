# Phase One IQ RAW (`.iiq`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.iiq`
* **Possible Origin**: Proprietary RAW format developed by Phase One for Phase One digital backs and medium-format camera systems
* **Category**: RAW / Medium-Format Sensor Data Container
* **LibRaw Support**: Yes (good support for many IQ and P-series backs)
* **FFMPEG Support**: No native RAW decoder; indirect support possible through LibRaw-based or external conversion pipelines
* **Rust alternative converters**:

  * `libraw-rs`
  * `rawloader`
  * `image`
  * `kamadak-exif`
  * External tools:

    * `Capture One`
    * `dcraw`
    * `darktable-cli`
    * `rawtherapee-cli`
    * `ImageMagick`
    * `exiftool`

The `.iiq` format is the primary RAW container used by:

* Phase One digital backs
* Phase One XF camera systems
* some Mamiya/Phase One integrated systems

The format was engineered for:

* extremely high image fidelity
* medium-format workflows
* tethered studio operation
* ultra-high dynamic range
* high precision color rendering

Compared to consumer RAW formats:

* `.iiq` files are extremely large
* metadata is more calibration-heavy
* tonal precision is prioritized over storage efficiency

Internally, `.iiq` is:

* proprietary
* partially TIFF-inspired
* heavily metadata-driven
* calibration-centric

Unlike DNG:

```text id="iiq01"
IIQ is not openly standardized
```

and significant portions remain undocumented.

---

# File structure

## High-Level Container Layout

Typical `.iiq` structure:

```text id="iiq02"
+----------------------+
| File Header          |
+----------------------+
| Directory Structures |
+----------------------+
| RAW Sensor Data      |
+----------------------+
| Embedded JPEG Preview|
+----------------------+
| Metadata Blocks      |
+----------------------+
| Calibration Data     |
+----------------------+
| MakerNotes           |
+----------------------+
```

Internally, the format behaves somewhat similarly to:

```text id="iiq03"
TIFF/EP-derived RAW systems
```

but with extensive proprietary extensions.

---

# Container Characteristics

The `.iiq` format contains:

* offset tables
* metadata sections
* RAW image blocks
* calibration payloads

Different camera generations may use:

* different compression
* different metadata layouts
* different RAW packing methods

---

# Main Structural Components

## 1. File Header

Contains:

* file identifiers
* offsets
* versioning
* metadata references

Conceptual structure:

```c id="iiq04"
struct IIQ_HEADER {
    uint32 magic;
    uint32 version;
    uint64 directory_offset;
}
```

Exact structure varies by camera generation.

---

## 2. Directory Structures

Used to locate:

* RAW blocks
* previews
* metadata
* calibration data

Conceptually similar to:

```text id="iiq05"
TIFF IFD systems
```

but not fully TIFF-compliant.

---

## 3. RAW Sensor Data

Usually:

* Bayer CFA
* very high bit depth
* massive medium-format resolutions

Observed bit depths:

* 14-bit
* 16-bit

Common CFA:

```text id="iiq06"
RGGB
```

Sensor characteristics:

* extremely high dynamic range
* smooth tonal transitions
* excellent shadow recovery

File sizes commonly exceed:

```text id="iiq07"
100–1000 MB
```

depending on sensor generation.

---

## 4. Embedded JPEG Preview

Most `.iiq` files contain:

* medium/high resolution JPEG preview
* Capture One-compatible rendering
* camera-generated previews

This is the preferred source for:

* thumbnails
* galleries
* quick previews

Advantages:

* extremely fast
* visually attractive
* low CPU usage

Disadvantages:

* already tone-mapped
* clipped highlights
* limited editing latitude

---

## 5. Metadata Blocks

Contains:

* EXIF
* lens information
* tethering metadata
* camera configuration
* sensor calibration

Rust recommendations:

```toml id="iiq08"
kamadak-exif
little_exif
```

---

## 6. Calibration Data

One of the most important components.

Likely contains:

* color matrices
* black level calibration
* lens shading correction
* sensor nonuniformity correction
* dead pixel maps

Critical for:

* accurate rendering
* smooth gradients
* color precision

---

## 7. MakerNotes

Phase One-specific metadata.

Possible contents:

* Capture One integration data
* tethering workflow metadata
* focus information
* lens corrections
* proprietary rendering hints

Documentation is limited.

---

# RAW Sensor Characteristics

## Medium Format Bayer Pipeline

Unlike Foveon:

```text id="iiq09"
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

Present in older systems.

Advantages:

* simpler decoding
* maximum fidelity

Disadvantages:

* huge files

---

### 2. Lossless Compression

Common in newer IQ-series systems.

Likely uses:

* predictive coding
* packed RAW encoding
* delta compression

Compression behavior is partially proprietary.

LibRaw abstracts much of this complexity.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="iiq10"
IIQ
 └── parse directory structures
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* preserves Capture One/Phase One rendering
* low CPU usage

Ideal for:

* DAM systems
* Tauri apps
* galleries
* lazy loading

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="iiq11"
160–512px
```

Often insufficient for modern interfaces.

---

## Tier 3 — Full RAW Decode

Required for:

* editing
* zoom rendering
* exposure recovery
* maximum detail extraction

Pipeline:

```text id="iiq12"
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

```toml id="iiq13"
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

```text id="iiq14"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="iiq15"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

The embedded JPEG preview is insufficient for:

* full dynamic range extraction
* highlight recovery
* white balance adjustment
* medium-format fidelity

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — Container Parsing

Read:

* directory structures
* metadata blocks
* RAW offsets
* preview references

Recommended crates:

```toml id="iiq16"
binrw
nom
```

Preferred:

```toml id="iiq17"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* Bayer RAW
* packed bit streams
* compressed blocks

Challenges:

* proprietary packing
* huge memory footprint
* camera-generation variability

---

## Stage 3 — Black Level Correction

Critical.

Without this:

* incorrect shadows
* tonal artifacts
* color shifts

---

## Stage 4 — White Balance

Apply:

* camera multipliers
* calibration matrices
* neutral references

---

## Stage 5 — Demosaicing

Recommended algorithms:

* RCD
* AMaZE
* DCB

For maximum quality:

```text id="iiq18"
AMaZE or RCD
```

because Phase One sensors preserve:

* extremely fine detail
* subtle textures
* smooth gradients

---

## Stage 6 — Lens Correction

Important for:

* Schneider Kreuznach lenses
* Rodenstock systems
* technical camera workflows

Possible corrections:

* distortion
* chromatic aberration
* lens shading

Possible implementation:

* Lensfun integration
* Capture One profile emulation
* empirical calibration

---

## Stage 7 — Color Space Conversion

Recommended pipeline:

```text id="iiq19"
camera RGB
 → XYZ
 → wide gamut working space
 → display transform
```

Recommended internal spaces:

* ProPhoto RGB
* ACEScg
* Rec2020

---

## Stage 8 — Tone Mapping

Medium-format sensors preserve:

* exceptional highlights
* deep shadows
* smooth transitions

Recommended:

* ACES
* filmic
* Reinhard

Avoid:

```text id="iiq20"
aggressive contrast curves
```

which destroy medium-format rendering characteristics.

---

## Stage 9 — Noise Reduction

Usually lighter than consumer cameras.

Recommended:

* preserve luminance detail
* chroma-first denoise

---

## Stage 10 — Sharpening

Use:

* restrained sharpening
* low-strength/high-radius methods

Avoid:

```text id="iiq21"
aggressive microcontrast sharpening
```

which damages natural medium-format rendering.

---

# Suggested Rust Architecture

## Module Layout

```text id="iiq22"
iiq/
 ├── parser
 ├── directories
 ├── metadata
 ├── maker_notes
 ├── calibration
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

* directory parsing
* embedded JPEG extraction
* EXIF parsing
* WebP export

This gives:

* immediate usability
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

* Capture One-like rendering
* advanced calibration handling
* GPU acceleration
* tiled rendering
* ROI decoding

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

`.iiq` contains:

* proprietary compression
* proprietary metadata
* calibration-heavy workflows
* generation variability

LibRaw already supports:

* Bayer unpacking
* CFA handling
* black level correction
* metadata parsing

Recommended architecture:

```text id="iiq23"
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

* huge resolutions
* high bit depth
* expensive demosaicing
* large memory footprint

But typically less algorithmically exotic than:

* X3F
* CR3
* compressed RAF

---

# Medium Format Rendering Characteristics

## Dynamic Range

Phase One systems preserve:

* excellent highlight rolloff
* deep shadow information
* subtle tonal gradients

Requires:

```text id="iiq24"
high precision processing pipelines
```

Avoid:

```text id="iiq25"
8-bit intermediate stages
```

until final output.

---

## Color Science

Phase One rendering emphasizes:

* realistic color
* smooth skin tones
* high tonal precision
* studio neutrality

Exact replication may require:

* proprietary Capture One transforms
* hidden calibration data
* empirical profiling

---

# Uncertain Points

## 1. Exact Compression Algorithms

Different generations likely use:

* different predictive coders
* different packing schemes

Many details remain undocumented.

---

## 2. Calibration Semantics

Some calibration structures remain proprietary.

Possible contents:

* defect correction maps
* sensor calibration grids
* spectral correction

---

## 3. MakerNote Semantics

Many proprietary tags remain undocumented.

Possible contents:

* tethering state
* workflow integration
* rendering hints

---

## 4. Capture One Rendering Pipeline

Exact rendering likely involves:

* proprietary highlight reconstruction
* custom tone curves
* advanced color transforms

Perfect replication is difficult.

---

## 5. Tile/Strip Variability

Some systems may vary between:

* strip storage
* tile storage
* packed block layouts

Needs empirical testing.

---

# Other informations

## MIME Type

Commonly observed:

```text id="iiq26"
image/x-phaseone-iiq
```

Not formally standardized.

---

# Cameras Using IIQ

Examples:

* Phase One IQ series
* Phase One P series
* Phase One XF systems
* Mamiya/Phase One integrated systems

Typically integrated with:

```text id="iiq27"
Capture One
```

workflow ecosystems.

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="iiq28"
RGB16 linear
```

Preferred for HDR workflows:

```text id="iiq29"
RGBA16F
```

Avoid:

```text id="iiq30"
8-bit intermediate processing
```

until final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="iiq31"
WebP lossy
```

## Editing cache

```text id="iiq32"
16-bit TIFF
```

## GPU visualization

```text id="iiq33"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Reliable parser

Foundation for all functionality.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. High precision processing

Essential for medium-format fidelity.

### 5. Calibration-aware rendering

Critical for professional output quality.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="iiq34"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="iiq35"
LibRaw + high precision medium-format RAW pipeline
```

## Native `.iiq` decoder implementation

Should be considered:

```text id="iiq36"
high complexity
```

because `.iiq` combines:

* proprietary metadata
* proprietary compression
* calibration-heavy workflows
* ultra-high-resolution medium-format Bayer RAW
* Capture One-integrated rendering behavior
* very large memory and processing requirements.
