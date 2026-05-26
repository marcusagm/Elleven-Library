# Kodak RAW / Kodak Digital Camera (`.kdc`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.kdc`
* **Possible Origin**: Proprietary RAW format used primarily by Kodak digital cameras and Kodak OEM imaging systems
* **Category**: RAW / Digital Camera Sensor Data Container
* **LibRaw Support**: Yes (good support for most known Kodak DSLR/compact variants)
* **FFMPEG Support**: No native decoder; indirect support possible through `libraw`, `dcraw`, or image conversion pipelines
* **Rust alternative converters**:

  * `libraw-rs`
  * `rawloader`
  * `image`
  * `kamadak-exif`
  * External integrations:

    * `dcraw`
    * `darktable-cli`
    * `rawtherapee-cli`
    * `ImageMagick`
    * `ufraw`
    * `exiftool`

The `.kdc` extension refers to multiple Kodak RAW implementations developed across different generations of Kodak digital cameras.

This is important because:

* `.kdc` is NOT a single unified specification
* Internal layouts vary significantly by camera generation
* Some variants are TIFF-derived
* Others are partially proprietary

Observed camera families:

* Kodak DC series
* Kodak DCS professional DSLR series
* Kodak EasyShare RAW-capable devices
* Kodak OEM industrial systems

The format is strongly associated with:

* CCD sensor pipelines
* early DSLR RAW ecosystems
* TIFF-based metadata organization

Unlike modern RAW formats:

* compression is usually simple or nonexistent
* metadata structures are less standardized
* previews are inconsistently embedded

---

# File structure

## High-Level Container Layout

Most `.kdc` variants are TIFF-like containers.

Typical structure:

```text id="1p4j4s"
+----------------------+
| TIFF Header          |
+----------------------+
| IFD Directory        |
+----------------------+
| EXIF Metadata        |
+----------------------+
| RAW Sensor Data      |
+----------------------+
| Embedded JPEG        |
+----------------------+
| MakerNotes           |
+----------------------+
```

Some variants are effectively:

```text id="p9z5ja"
TIFF/EP derivatives
```

while others use Kodak-specific MakerNote extensions.

---

# TIFF Foundation

Most KDC variants use:

## TIFF Magic

Little-endian:

```hex id="j9y1uk"
49 49 2A 00
```

Big-endian:

```hex id="70jjsh"
4D 4D 00 2A
```

Meaning:

```text id="zqcfui"
II*  or  MM*
```

This makes KDC structurally closer to:

* CR2
* NEF
* DNG
* ORF

than to:

* X3F
* RAF
* proprietary binary containers

---

# Main Structural Components

## 1. TIFF Header

Contains:

* endian marker
* TIFF magic
* offset to first IFD

Typical structure:

```c id="0ukb9t"
struct TIFF_HEADER {
    uint16 endian;
    uint16 magic;
    uint32 ifd_offset;
}
```

---

## 2. IFD (Image File Directory)

The core organizational structure.

Contains entries for:

* image dimensions
* compression
* RAW strip offsets
* preview offsets
* metadata
* MakerNotes

Typical TIFF entry:

```c id="0p5d0e"
struct IFD_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

---

## 3. RAW Sensor Data

Usually stored as:

* strips
* tiles
* contiguous blocks

Compression:

* often none
* sometimes simple lossless compression

Common bit depths:

* 10-bit
* 12-bit
* occasionally 14-bit

Sensor types:

* CCD
* Bayer CFA

Unlike Foveon:

```text id="6m4n7e"
1 pixel = 1 color sample
```

requiring demosaicing.

---

## 4. Embedded JPEG Preview

Many KDC files contain:

* medium-resolution JPEG
* camera-rendered preview
* EXIF thumbnail

This is critical for:

* thumbnail extraction
* fast browsing
* UI previews

Preview quality varies greatly by camera generation.

---

## 5. EXIF Metadata

Typically standard TIFF EXIF.

Contains:

* ISO
* shutter speed
* aperture
* timestamp
* focal length
* white balance
* orientation

Rust parsing recommendations:

```toml id="jlwm5n"
kamadak-exif
little_exif
```

---

## 6. Kodak MakerNotes

Most Kodak-specific behavior is here.

Contains:

* sensor calibration
* color matrices
* white balance coefficients
* camera serial data
* firmware identifiers

This section is poorly documented publicly.

Different camera models use:

* different MakerNote schemas
* different offsets
* different encoding conventions

---

# RAW Sensor Characteristics

## Kodak CCD Pipeline

Most KDC cameras use:

* CCD sensors
* Bayer color filter arrays

Common patterns:

```text id="qv0k2z"
RGGB
BGGR
GRBG
GBRG
```

This means RAW decoding requires:

* black level correction
* white balance
* demosaicing
* color transform

---

# Compression Behavior

## Observed Compression Modes

### 1. Uncompressed RAW

Most common in older Kodak cameras.

Advantages:

* easy parsing
* fast decoding

Disadvantages:

* large files

---

### 2. Lossless Compression

Observed in some DSLR/professional variants.

Usually:

* delta encoding
* predictive coding
* TIFF-compatible compression variants

Rarely documented officially.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="vobpy7"
KDC
 └── parse TIFF IFD
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* minimal CPU usage
* preserves camera rendering

Best for:

* galleries
* Tauri previews
* file explorers
* lazy loading

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Typical size:

```text id="ydmrb5"
160x120
320x240
```

Usually too small for quality previews.

---

## Tier 3 — Full RAW Decode

Use for:

* high-quality previews
* zooming
* editing
* archival conversion

Pipeline:

```text id="ntm6wy"
RAW decode
 → black level correction
 → white balance
 → demosaic
 → color transform
 → gamma
 → tone mapping
 → resize
 → WebP
```

---

# Recommended Rust Thumbnail Pipeline

## Suggested Crates

```toml id="m6kt2t"
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

```text id="91e9rp"
Quality: 70–85
Method: lossy
```

### Large previews

```text id="v98rlx"
Quality: 90–95
Method: high-quality lossy or lossless
```

---

# Strategy for Visualization

## High-Fidelity Rendering Pipeline

The embedded JPEG preview is insufficient for:

* exposure recovery
* highlight reconstruction
* WB adjustment
* maximum detail extraction

Proper rendering requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — TIFF Parsing

Read:

* IFD tables
* metadata
* RAW offsets
* preview offsets

Recommended Rust crates:

```toml id="l34lnn"
binrw
nom
tiff
```

Preferred:

```toml id="u2jz9v"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* Bayer data
* bit-packed pixels
* strip/tile organization

Challenges:

* variable bit packing
* endian handling
* camera-specific layouts

---

## Stage 3 — Black Level Correction

Critical for CCD sensors.

Without this:

* lifted blacks
* incorrect dynamic range
* color artifacts

---

## Stage 4 — White Balance

Apply:

* camera multipliers
* daylight calibration
* user WB adjustments

---

## Stage 5 — Demosaicing

Required because:

```text id="jlwm0m"
1 sensor pixel = 1 color component
```

Recommended algorithms:

* Bilinear
* VNG
* AHD
* DCB
* RCD

For best quality:

```text id="u96a7y"
RCD or DCB
```

---

## Stage 6 — Color Space Conversion

Recommended pipeline:

```text id="3wv7jz"
camera RGB
 → XYZ
 → ProPhoto / Rec2020 internal
 → sRGB output
```

---

## Stage 7 — Tone Mapping

Recommended:

* filmic
* Reinhard
* ACES

CCD sensors frequently exhibit:

* strong highlight rolloff
* distinctive color response

Tone mapping strongly affects visual fidelity.

---

## Stage 8 — Noise Reduction

Older Kodak CCD sensors may produce:

* luminance noise
* hot pixels
* chroma noise

Recommended:

* chroma-only denoise initially
* preserve luminance detail

---

# Suggested Rust Architecture

## Module Layout

```text id="a3uv7k"
kdc/
 ├── tiff_parser
 ├── ifd
 ├── metadata
 ├── maker_notes
 ├── jpeg_extract
 ├── raw_extract
 ├── demosaic
 ├── color_pipeline
 ├── thumbnail
 ├── webp_export
 └── cache
```

---

# Recommended Initial Strategy

## Phase 1 — Practical Implementation

Implement:

* TIFF parser
* JPEG preview extraction
* EXIF parsing
* WebP export

This gives:

* immediate usability
* fast previews
* stable implementation

---

## Phase 2 — RAW Decoding

Add:

* Bayer unpacking
* demosaicing
* white balance
* color transforms

---

## Phase 3 — Advanced Fidelity

Implement:

* Kodak-specific color science
* advanced denoise
* highlight recovery
* GPU acceleration

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

Kodak RAW variants are inconsistent across models.

LibRaw already handles:

* bit unpacking
* metadata interpretation
* sensor calibration
* CFA layouts

Recommended architecture:

```text id="jlwm9y"
Rust frontend
    ↓
LibRaw FFI
    ↓
16-bit RGB output
```

---

# Performance Considerations

## Embedded JPEG Path

Very fast:

* suitable for real-time browsing
* low memory usage

---

## Full RAW Decode

More expensive because of:

* demosaicing
* CCD noise reduction
* color transforms

Still generally lighter than:

* X3F
* compressed CR3
* modern Sony RAW

---

# Uncertain Points

## 1. Multiple KDC Variants

`.kdc` is not fully standardized.

Different Kodak cameras use:

* different MakerNotes
* different compression
* different RAW packing

---

## 2. Compression Algorithms

Some professional Kodak DSLRs use:

* undocumented predictive compression
* proprietary strip layouts

Not fully documented publicly.

---

## 3. Color Science

Kodak historically used:

* proprietary color matrices
* unique CCD tuning

Exact reproduction may require:

* reverse engineering
* ICC profiling
* empirical calibration

---

## 4. MakerNote Semantics

Many MakerNote fields remain:

* partially undocumented
* camera-specific

---

## 5. Bit Packing Differences

Observed:

* packed 10-bit
* packed 12-bit
* padded storage
* strip alignment variations

Need empirical testing.

---

# Other informations

## MIME Type

Commonly observed:

```text id="jlwm3m"
image/x-kodak-kdc
```

Not universally standardized.

---

# Cameras Using KDC

Examples:

* Kodak DCS series
* Kodak DC series
* Kodak EasyShare RAW-capable devices

Some Kodak DSLR systems were Nikon/Canon hybrids with Kodak sensors.

This may affect:

* metadata compatibility
* CFA interpretation
* color science

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="jlwm6u"
RGB16 linear
```

Avoid:

```text id="jlwm7e"
u8 processing
```

before final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="jlwm8r"
WebP lossy
```

## Editing cache

```text id="jlwm2w"
16-bit TIFF
```

## GPU visualization

```text id="jlwm1t"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Reliable TIFF/IFD parser

Foundation for everything.

### 3. LibRaw integration

Avoids massive reverse engineering effort.

### 4. Correct Bayer handling

Essential for fidelity.

---

# Most Important Practical Insight

For production-grade software:

## Thumbnail generation

Use:

```text id="jlwm4q"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="jlwm0r"
LibRaw-based RAW decode pipeline
```

## Native Kodak RAW implementation

Should be considered:

```text id="jlwm5a"
medium-to-high complexity
```

because Kodak variants differ substantially between camera generations and documentation quality is inconsistent.
