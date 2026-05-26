# GoPro RAW / GoPro Photo RAW (`.gpr`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.gpr`
* **Possible Origin**: Proprietary RAW format developed by GoPro for HERO-series action cameras
* **Category**: RAW / Digital Negative / TIFF-DNG-derived Sensor Container
* **LibRaw Support**: Yes (good support for supported GoPro HERO models)
* **FFMPEG Support**: No native RAW decoder; indirect processing possible through DNG-compatible pipelines and external RAW tools
* **Rust alternative converters**:

  * `libraw-rs`
  * `rawloader`
  * `image`
  * `kamadak-exif`
  * External tools:

    * `dcraw`
    * `darktable-cli`
    * `rawtherapee-cli`
    * `Adobe DNG Converter`
    * `exiftool`
    * `ImageMagick`

The `.gpr` format is a GoPro-specific RAW image format introduced primarily in:

* GoPro HERO5 Black
* HERO6
* HERO7
* HERO8
* HERO9
* HERO10
* HERO11
* HERO12

Internally, `.gpr` behaves very similarly to:

```text id="2h9u7m"
Adobe DNG / TIFF-EP
```

In practice, many `.gpr` files are:

* DNG-compatible
* TIFF-based
* using standard RAW metadata structures
* augmented with GoPro-specific MakerNotes and calibration fields

This makes `.gpr` substantially easier to support than:

* X3F
* CR3
* RAF
* proprietary compressed RAW systems

---

# File structure

## High-Level Container Layout

Typical `.gpr` structure:

```text id="1x8y5u"
+----------------------+
| TIFF Header          |
+----------------------+
| IFD Directory Tree   |
+----------------------+
| EXIF Metadata        |
+----------------------+
| RAW Sensor Data      |
+----------------------+
| Embedded JPEG Preview|
+----------------------+
| DNG Metadata         |
+----------------------+
| GoPro MakerNotes     |
+----------------------+
```

The format is effectively:

```text id="jlwmk1"
TIFF/EP + DNG extensions + GoPro metadata
```

---

# TIFF Foundation

## TIFF Magic

Little-endian:

```hex id="jlwmk2"
49 49 2A 00
```

Big-endian:

```hex id="jlwmk3"
4D 4D 00 2A
```

Most observed GPR files are:

```text id="jlwmk4"
little-endian
```

---

# DNG Compatibility

GPR files often expose:

* DNGVersion
* CFA pattern
* ColorMatrix
* BlackLevel
* WhiteLevel
* CalibrationIlluminant

This means:

* many DNG parsers partially work automatically
* Adobe Camera Raw compatibility exists
* LibRaw support is relatively strong

---

# Main Structural Components

## 1. TIFF Header

Contains:

* endian marker
* TIFF magic
* offset to first IFD

Typical structure:

```c id="jlwmk5"
struct TIFF_HEADER {
    uint16 endian;
    uint16 magic;
    uint32 first_ifd_offset;
}
```

---

## 2. IFD (Image File Directory)

Central organizational structure.

Contains:

* RAW image offsets
* image dimensions
* compression flags
* EXIF references
* thumbnail references
* JPEG preview offsets

Typical entry:

```c id="jlwmk6"
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
* packed linear RAW
* lossless compressed or lightly compressed

Common bit depths:

* 10-bit
* 12-bit

Likely sensor vendors:

* Sony
* OmniVision
* custom GoPro ISP integrations

Common CFA patterns:

```text id="jlwmk7"
RGGB
BGGR
```

---

## 4. Embedded JPEG Preview

Most `.gpr` files contain:

* medium or high-resolution JPEG preview
* ISP-rendered image
* GoPro color science already applied

This is the preferred source for:

* thumbnail extraction
* fast browsing
* gallery rendering

Advantages:

* extremely fast
* low CPU usage
* visually attractive

Disadvantages:

* already tone-mapped
* clipped highlights
* reduced dynamic range

---

## 5. EXIF Metadata

Typical fields:

* ISO
* shutter speed
* aperture (fixed on many GoPro models)
* focal length
* timestamp
* GPS
* orientation
* white balance

Rust recommendations:

```toml id="jlwmk8"
kamadak-exif
little_exif
```

---

## 6. DNG Metadata Blocks

Common tags:

* BlackLevel
* WhiteLevel
* CFARepeatPatternDim
* CFAPattern
* ColorMatrix1
* AsShotNeutral

These are critical for:

* accurate color rendering
* RAW reconstruction
* proper dynamic range handling

---

## 7. GoPro MakerNotes

Likely contains:

* stabilization metadata
* lens correction hints
* gyro synchronization
* ISP tuning
* noise reduction parameters

Some metadata may overlap with:

* MP4 gyro telemetry systems
* GoPro GPMF ecosystem

Documentation is limited.

---

# RAW Sensor Characteristics

## Bayer CFA Pipeline

Unlike Foveon:

```text id="jlwmk9"
1 sensor pixel = 1 color sample
```

Requires:

* demosaicing
* white balance
* color transform

---

# Compression Behavior

## Observed Compression Modes

### 1. Uncompressed RAW

Some early models.

Advantages:

* easy parsing
* fast decoding

Disadvantages:

* large files

---

### 2. Lossless Compression

More common on newer HERO generations.

Likely:

* TIFF-compatible predictor compression
* DNG lossless JPEG variants
* packed RAW encoding

LibRaw handles most supported variants transparently.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="jlwmka"
GPR
 └── parse TIFF IFD
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* minimal CPU cost
* preserves GoPro ISP rendering

Best for:

* file explorers
* galleries
* Tauri applications
* lazy loading

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="jlwmkb"
160px–512px
```

May be insufficient for modern UI previews.

---

## Tier 3 — Full RAW Decode

Use for:

* high-quality previews
* editing
* zoom rendering
* export pipelines

Pipeline:

```text id="jlwmkc"
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

```toml id="jlwmkd"
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

```text id="jlwmke"
Quality: 70–85
Lossy
```

### Large previews

```text id="开奖直播kf"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

The embedded JPEG preview is insufficient for:

* exposure recovery
* highlight reconstruction
* white balance editing
* maximum detail extraction

Proper visualization requires RAW processing.

---

# Recommended Visualization Pipeline

## Stage 1 — TIFF/DNG Parsing

Read:

* IFD tables
* metadata
* RAW offsets
* compression flags

Recommended crates:

```toml id="开奖直播kg"
binrw
nom
tiff
```

Preferred:

```toml id="开奖直播kh"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* Bayer RAW
* packed pixels
* strip/tile layouts

Challenges:

* packed bit decoding
* compression support
* endian correctness

---

## Stage 3 — Black Level Correction

Essential.

Without this:

* lifted shadows
* incorrect dynamic range
* color artifacts

---

## Stage 4 — White Balance

Apply:

* AsShotNeutral
* camera multipliers
* DNG calibration matrices

---

## Stage 5 — Demosaicing

Recommended algorithms:

* AHD
* DCB
* RCD
* AMaZE

For highest quality:

```text id="开奖直播ki"
AMaZE or RCD
```

---

## Stage 6 — Lens Correction

Important for GoPro.

GoPro lenses exhibit:

* strong barrel distortion
* chromatic aberration
* edge stretching

Correction may require:

* empirical lens profiles
* MakerNote interpretation
* OpenCV correction models

---

## Stage 7 — Color Space Conversion

Recommended pipeline:

```text id="开奖直播kj"
camera RGB
 → XYZ
 → wide gamut internal
 → sRGB display output
```

Recommended working spaces:

* ProPhoto RGB
* Rec2020

---

## Stage 8 — Tone Mapping

Recommended:

* ACES
* filmic
* Reinhard

GoPro sensors often:

* compress highlights aggressively
* apply strong ISP contrast in JPEG previews

RAW decoding allows substantially better highlight recovery.

---

## Stage 9 — Noise Reduction

Action camera sensors are physically small.

Common issues:

* chroma noise
* low-light noise
* temporal noise artifacts

Recommended:

* chroma-first denoise
* edge-preserving luminance denoise

---

# Suggested Rust Architecture

## Module Layout

```text id="开奖直播kk"
gpr/
 ├── tiff_parser
 ├── ifd
 ├── metadata
 ├── dng_tags
 ├── maker_notes
 ├── jpeg_extract
 ├── raw_extract
 ├── demosaic
 ├── lens_correction
 ├── color_pipeline
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
* WebP thumbnail generation

This provides:

* immediate utility
* fast previews
* robust compatibility

---

## Phase 2 — RAW Rendering

Add:

* Bayer unpacking
* demosaicing
* DNG calibration support
* white balance
* tone mapping

---

## Phase 3 — Advanced Fidelity

Implement:

* lens correction
* GoPro-specific color science
* GPU acceleration
* HDR merging
* stabilization metadata support

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

GPR behaves similarly to DNG but may include:

* packed RAW variations
* GoPro-specific metadata
* camera-specific calibration

LibRaw already supports:

* Bayer unpacking
* DNG matrices
* black level correction
* CFA interpretation

Recommended architecture:

```text id="开奖直播kl"
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

* suitable for real-time galleries
* low memory usage

---

## Full RAW Decode

Moderate CPU cost:

* demosaicing
* lens correction
* denoise
* tone mapping

Still generally lighter than:

* CR3
* compressed RAF
* X3F

---

# Uncertain Points

## 1. GoPro MakerNote Semantics

Many fields remain undocumented.

Possible contents:

* stabilization metadata
* gyro synchronization
* ISP tuning
* internal lens correction parameters

---

## 2. Compression Variants

Different HERO generations may use:

* different packed RAW encodings
* different predictor compression

---

## 3. Sensor Vendor Variability

Different GoPro models likely use:

* different sensors
* different CFA behavior
* different black level calibration

---

## 4. Lens Profile Accuracy

Official correction models are not fully public.

Empirical calibration may be required.

---

## 5. Noise Reduction Metadata

Some ISP tuning parameters may exist internally but remain undocumented.

---

# Other informations

## MIME Type

Commonly observed:

```text id="开奖直播km"
image/x-gopro-gpr
```

Not formally standardized.

---

# Cameras Using GPR

Examples:

* GoPro HERO5 Black
* GoPro HERO6 Black
* GoPro HERO7 Black
* GoPro HERO8 Black
* GoPro HERO9 Black
* GoPro HERO10 Black
* GoPro HERO11 Black
* GoPro HERO12 Black

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="开奖直播kn"
RGB16 linear
```

Avoid:

```text id="开奖直播ko"
u8 processing
```

until final display/export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="开奖直播kp"
WebP lossy
```

## Editing cache

```text id="开奖直播kq"
16-bit TIFF
```

## GPU visualization

```text id="开奖直播kr"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest practical ROI.

### 2. Reliable TIFF/DNG parser

Foundation for all processing.

### 3. LibRaw integration

Avoids large reverse-engineering effort.

### 4. Correct DNG calibration handling

Essential for color fidelity.

### 5. Lens correction support

Critical for GoPro imagery quality.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="开奖直播ks"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="开奖直播kt"
LibRaw + DNG-calibrated RAW pipeline
```

## Native RAW implementation

Should be considered:

```text id="开奖直播ku"
medium complexity
```

because `.gpr` is structurally close to DNG/TIFF formats and therefore significantly easier to support than heavily proprietary RAW ecosystems such as:

* X3F
* CR3
* RAF compressed RAW
* modern Sony compressed ARW variants.
