# Hasselblad RAW (`.3fr`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.3fr`
* **Possible Origin**: Proprietary RAW image format developed by Hasselblad for medium-format digital camera backs and integrated medium-format camera systems
* **Category**: RAW / Digital Camera Sensor Data Container
* **LibRaw Support**: Yes (good support for most supported Hasselblad generations)
* **FFMPEG Support**: No native RAW decoder; indirect support possible through `libraw`, `dcraw`, or external conversion pipelines
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

The `.3fr` format is associated primarily with:

* Hasselblad H-series digital backs
* early Hasselblad integrated digital systems
* medium-format CCD and CMOS workflows

The format predates Hasselblad's later:

```text id="3fr01"
FFF
```

and partially overlaps with TIFF/EP-style RAW architectures.

The design priorities of `.3fr` emphasize:

* maximum tonal fidelity
* medium-format dynamic range
* studio workflow quality
* color precision

Unlike compressed consumer RAW formats:

* `.3fr` frequently prioritizes fidelity over storage efficiency
* files are typically very large
* preview structures are relatively simple

---

# File structure

## High-Level Container Layout

The `.3fr` format is largely TIFF-derived.

Typical structure:

```text id="3fr02"
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
| Hasselblad MakerNotes|
+----------------------+
```

Internally, `.3fr` behaves similarly to:

```text id="3fr03"
TIFF/EP RAW containers
```

with Hasselblad-specific extensions.

---

# TIFF Foundation

## TIFF Magic

Little-endian:

```hex id="3fr04"
49 49 2A 00
```

Big-endian:

```hex id="3fr05"
4D 4D 00 2A
```

Most observed `.3fr` files are:

```text id="3fr06"
little-endian
```

---

# Main Structural Components

## 1. TIFF Header

Contains:

* endian marker
* TIFF magic
* offset to first IFD

Typical structure:

```c id="3fr07"
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

* RAW image offsets
* strip/tile locations
* compression metadata
* preview references
* EXIF references
* MakerNotes

Typical entry:

```c id="3fr08"
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
* medium-format CCD or CMOS
* very high bit depth
* extremely large resolution

Observed bit depths:

* 12-bit
* 14-bit
* 16-bit linear

Common CFA patterns:

```text id="3fr09"
RGGB
```

Sensor characteristics:

* high dynamic range
* low noise at base ISO
* very high color fidelity

---

## 4. Embedded JPEG Preview

Most `.3fr` files contain:

* medium-resolution JPEG
* high-quality preview image
* camera-rendered tone mapping

This is the preferred source for:

* thumbnail extraction
* gallery browsing
* fast preview generation

Advantages:

* very fast
* low CPU usage
* preserves Hasselblad color rendering

Disadvantages:

* already tone-mapped
* clipped highlights
* reduced editing latitude

---

## 5. EXIF Metadata

Typical fields:

* ISO
* shutter speed
* aperture
* focal length
* timestamp
* orientation
* white balance

Rust recommendations:

```toml id="3fr10"
kamadak-exif
little_exif
```

---

## 6. Hasselblad MakerNotes

Contains likely:

* lens correction parameters
* sensor calibration
* color matrices
* black level calibration
* Phocus workflow metadata

Documentation is limited.

Different camera generations may use:

* different MakerNote structures
* different offsets
* proprietary calibration blocks

---

# RAW Sensor Characteristics

## Medium Format Bayer Pipeline

Unlike Foveon:

```text id="3fr11"
1 sensor pixel = 1 color component
```

Requires:

* demosaicing
* white balance
* color transforms

---

# Compression Behavior

## Observed Compression Modes

### 1. Uncompressed RAW

Common in older medium-format systems.

Advantages:

* easier parsing
* lower decode complexity

Disadvantages:

* enormous file sizes

---

### 2. Lossless Compression

Observed in newer systems.

Likely:

* predictive compression
* TIFF-compatible packing
* proprietary delta encoding

Usually supported by LibRaw.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="3fr12"
3FR
 └── parse TIFF IFD
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* preserves Hasselblad rendering
* low CPU cost

Ideal for:

* file explorers
* Tauri applications
* gallery systems
* lazy loading

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="3fr13"
160–512px
```

Often insufficient for modern UI previews.

---

## Tier 3 — Full RAW Decode

Required for:

* zoom rendering
* editing
* exposure recovery
* high-fidelity previews

Pipeline:

```text id="3fr14"
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

```toml id="3fr15"
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

```text id="3fr16"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="3fr17"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

The embedded JPEG preview is insufficient for:

* highlight recovery
* white balance editing
* maximum dynamic range
* medium-format detail extraction

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — TIFF Parsing

Read:

* IFD tables
* metadata
* RAW offsets
* strip/tile organization

Recommended crates:

```toml id="3fr18"
binrw
nom
tiff
```

Preferred:

```toml id="3fr19"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* Bayer RAW
* packed pixel data
* compression blocks

Challenges:

* packed bit decoding
* strip alignment
* large memory requirements

Medium-format files can exceed:

```text id="3fr20"
100–400 MB
```

per image.

---

## Stage 3 — Black Level Correction

Critical.

Without this:

* incorrect shadows
* color shifts
* poor tonal accuracy

---

## Stage 4 — White Balance

Apply:

* camera multipliers
* calibration matrices
* neutral references

---

## Stage 5 — Demosaicing

Recommended algorithms:

* DCB
* RCD
* AMaZE

For maximum quality:

```text id="3fr21"
AMaZE or RCD
```

because medium-format sensors contain:

* extremely high detail density
* subtle tonal gradients

---

## Stage 6 — Color Space Conversion

Recommended pipeline:

```text id="3fr22"
camera RGB
 → XYZ
 → wide gamut working space
 → display output
```

Recommended internal spaces:

* ProPhoto RGB
* Rec2020

---

## Stage 7 — Tone Mapping

Medium-format RAW files contain:

* very high dynamic range
* smooth highlight rolloff

Recommended:

* filmic
* ACES
* Reinhard

---

## Stage 8 — Noise Reduction

Usually lighter than small-sensor systems.

Recommended:

* preserve luminance detail
* chroma-first denoise

---

## Stage 9 — Sharpening

Medium-format imagery benefits from:

* restrained sharpening
* high-radius low-strength approaches

Avoid:

```text id="3fr23"
aggressive edge sharpening
```

which destroys natural medium-format rendering.

---

# Suggested Rust Architecture

## Module Layout

```text id="3fr24"
3fr/
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

* immediate practical value
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

* Hasselblad-specific color science
* lens correction
* GPU acceleration
* tiled rendering
* partial decode

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

`.3fr` variants may differ by:

* sensor generation
* compression
* MakerNotes
* packing layout

LibRaw already supports:

* Bayer unpacking
* black level handling
* metadata interpretation
* CFA decoding

Recommended architecture:

```text id="3fr25"
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
* suitable for real-time browsing

---

## Full RAW Decode

Heavy workload because of:

* huge resolutions
* high bit depth
* expensive demosaicing
* large memory bandwidth

But generally simpler than:

* CR3
* X3F
* compressed RAF

---

# Medium Format Rendering Considerations

## Dynamic Range

Medium-format sensors frequently preserve:

* exceptional shadow detail
* smooth tonal transitions

This requires:

```text id="3fr26"
high precision internal processing
```

Avoid:

```text id="3fr27"
8-bit intermediate pipelines
```

---

## Color Science

Hasselblad rendering emphasizes:

* neutral tones
* smooth skin rendering
* natural highlight transitions

Exact reproduction may require:

* reverse-engineered matrices
* ICC profiling
* Phocus matching

---

# Uncertain Points

## 1. Compression Variants

Different generations may use:

* different predictive compression
* proprietary packing schemes

---

## 2. MakerNote Semantics

Many Hasselblad MakerNote fields remain undocumented.

Possible contents:

* lens correction
* sensor calibration
* Phocus workflow metadata

---

## 3. Camera Generation Differences

Older CCD systems and newer CMOS systems may differ significantly:

* bit depth
* black level behavior
* color matrices
* noise profiles

---

## 4. Exact Phocus Rendering

Hasselblad Phocus likely applies:

* proprietary tone curves
* custom highlight reconstruction
* camera-specific color science

Exact replication is difficult.

---

## 5. Tiling/Strip Variability

Some `.3fr` files may vary between:

* strip-based RAW storage
* tiled RAW layouts

Requires empirical validation.

---

# Other informations

## MIME Type

Commonly observed:

```text id="3fr28"
image/x-hasselblad-3fr
```

Not formally standardized.

---

# Cameras Using 3FR

Examples:

* Hasselblad H3D
* Hasselblad H4D
* Hasselblad H5D
* Hasselblad CF-series digital backs

Some systems transitioned later toward:

```text id="3fr29"
FFF
```

containers.

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="3fr30"
RGB16 linear
```

Preferred for HDR workflows:

```text id="3fr31"
RGBA16F
```

Avoid:

```text id="3fr32"
8-bit processing
```

until final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="3fr33"
WebP lossy
```

## Editing cache

```text id="3fr34"
16-bit TIFF
```

## GPU visualization

```text id="3fr35"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Reliable TIFF parser

Foundation for all functionality.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. High precision processing

Essential for medium-format fidelity.

### 5. Correct demosaicing pipeline

Critical for preserving detail.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="3fr36"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="3fr37"
LibRaw + high precision RAW pipeline
```

## Native RAW implementation

Should be considered:

```text id="3fr38"
medium-to-high complexity
```

because `.3fr` combines:

* TIFF-derived RAW structures
* large medium-format sensor data
* proprietary MakerNotes
* possible compression variants
* high precision tonal workflows.
