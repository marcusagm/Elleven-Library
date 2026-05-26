# Panasonic RAW Version 2 (`.rw2`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.rw2`
* **Possible Origin**: Proprietary RAW format developed by Panasonic for Lumix digital cameras
* **Category**: RAW / Bayer Sensor Image Container
* **LibRaw Support**: Yes (excellent support across most Panasonic Lumix generations)
* **FFMPEG Support**: No native RAW decoder; indirect support possible through LibRaw or external conversion pipelines
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
    * `exiftool`

The `.rw2` format is the primary RAW format used by:

* Panasonic Lumix mirrorless cameras
* Panasonic compact cameras
* Micro Four Thirds systems
* Leica/Panasonic collaborative camera systems

It replaced earlier Panasonic RAW formats such as:

```text id="rw201"
.raw
```

and introduced:

* better metadata organization
* improved compression
* higher bit-depth support
* larger sensor compatibility

Internally, `.rw2` is:

* TIFF-derived
* proprietary
* Bayer-oriented
* metadata-heavy

It shares conceptual similarities with:

* TIFF/EP
* Olympus ORF
* Leica RAW variants
* Panasonic-derived Leica formats

---

# File structure

## High-Level Container Layout

Typical `.rw2` structure:

```text id="rw202"
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
| Panasonic MakerNotes |
+----------------------+
```

The structure is strongly TIFF-like.

Most parsers can treat `.rw2` as:

```text id="rw203"
a proprietary TIFF-based RAW container
```

with Panasonic-specific tags and packing methods.

---

# TIFF Foundation

## TIFF Magic

Little-endian:

```hex id="rw204"
49 49 2A 00
```

Big-endian:

```hex id="rw205"
4D 4D 00 2A
```

Most `.rw2` files are:

```text id="rw206"
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

```c id="rw207"
struct TIFF_HEADER {
    uint16 endian;
    uint16 magic;
    uint32 first_ifd_offset;
}
```

---

## 2. IFD (Image File Directory)

Contains:

* RAW offsets
* image dimensions
* compression metadata
* preview references
* EXIF references

Typical TIFF-like structure:

```c id="rw208"
struct IFD_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

Panasonic adds proprietary tags for:

* sensor data
* lens metadata
* stabilization
* focus information

---

## 3. RAW Sensor Data

Usually:

* Bayer CFA
* CMOS sensor data
* packed RAW bitstreams

Observed bit depths:

* 10-bit
* 12-bit
* 14-bit

Common CFA:

```text id="rw209"
RGGB
```

Some models may use:

* BGGR
* GBRG

depending on sensor generation.

---

## 4. Embedded JPEG Preview

Most `.rw2` files contain:

* medium/high-resolution JPEG preview
* EXIF thumbnail
* camera-rendered preview

This is the preferred source for:

* thumbnails
* galleries
* instant previews

Advantages:

* extremely fast
* low CPU usage
* preserves Panasonic rendering

Disadvantages:

* already tone-mapped
* clipped highlights
* limited editing latitude

---

## 5. EXIF Metadata

Typical metadata:

* ISO
* aperture
* shutter speed
* focal length
* orientation
* white balance
* GPS (some models)

Rust recommendations:

```toml id="rw210"
kamadak-exif
little_exif
```

---

## 6. Panasonic MakerNotes

Contains proprietary metadata.

Possible contents:

* lens correction profiles
* image stabilization metadata
* focus distance
* face detection data
* lens distortion coefficients
* chromatic aberration correction
* shading correction

Panasonic MakerNotes are relatively extensive.

---

# RAW Sensor Characteristics

## Bayer RAW Pipeline

`.rw2` uses:

```text id="rw211"
single-layer Bayer CFA sensors
```

Requires:

* demosaicing
* white balance
* color transforms

---

# Compression Behavior

## Observed Compression Modes

### 1. Packed RAW Encoding

Very common.

Pixels are packed tightly:

* 10-bit packed
* 12-bit packed
* 14-bit packed

This reduces file size while remaining:

```text id="rw212"
lossless
```

---

## 2. Panasonic Predictive Compression

Some generations appear to use:

* delta encoding
* predictive coding
* sensor-row compression

Exact details are partially proprietary.

LibRaw handles most known variants.

---

# RAW Packing Characteristics

## Packed Bitstream Example

12-bit packing example:

```text id="rw213"
2 pixels = 24 bits = 3 bytes
```

Requires:

* bit unpacking
* endian-aware extraction

Performance-sensitive implementation area.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="rw214"
RW2
 └── parse TIFF IFD
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* low CPU usage
* preserves camera rendering

Ideal for:

* galleries
* Tauri applications
* DAM systems
* file browsers

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="rw215"
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

```text id="rw216"
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

```toml id="rw217"
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

```text id="rw218"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="rw219"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

Embedded JPEG previews are insufficient for:

* highlight reconstruction
* white balance editing
* noise reduction
* full dynamic range extraction

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — TIFF Parsing

Read:

* IFD structures
* RAW offsets
* metadata
* preview references

Recommended crates:

```toml id="rw220"
binrw
nom
tiff
```

Preferred:

```toml id="rw221"
binrw
```

---

## Stage 2 — RAW Extraction

Read:

* packed Bayer data
* compressed blocks
* CFA layout

Challenges:

* packed bitstream decoding
* predictive compression
* sensor-generation differences

---

## Stage 3 — Black Level Correction

Critical.

Without this:

* lifted shadows
* incorrect contrast
* shadow color artifacts

Panasonic sensors often require:

```text id="rw222"
careful black level handling
```

for proper shadow rendering.

---

## Stage 4 — White Balance

Apply:

* camera multipliers
* neutral references
* metadata-based corrections

---

## Stage 5 — Demosaicing

Recommended algorithms:

* AMaZE
* RCD
* DCB

For maximum quality:

```text id="rw223"
AMaZE
```

because Panasonic sensors preserve:

* strong fine detail
* high edge sharpness
* dense texture information

---

## Stage 6 — Lens Correction

Very important for Panasonic systems.

Panasonic MakerNotes frequently contain:

* distortion coefficients
* chromatic aberration data
* vignetting correction

Possible implementation:

* Lensfun
* MakerNote interpretation
* empirical calibration

---

## Stage 7 — Color Space Conversion

Recommended pipeline:

```text id="rw224"
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

## Stage 8 — Tone Mapping

Panasonic rendering often emphasizes:

* high perceived sharpness
* strong local contrast
* vivid colors

Recommended:

* filmic
* ACES
* Reinhard

Avoid:

```text id="rw225"
aggressive sharpening during tone mapping
```

which creates artifacts.

---

## Stage 9 — Noise Reduction

Panasonic sensors may exhibit:

* chroma noise
* shadow noise
* high-ISO texture artifacts

Recommended:

* chroma-first denoise
* edge-preserving luminance denoise

---

## Stage 10 — Sharpening

Panasonic images are naturally sharp.

Use:

* restrained sharpening
* detail-preserving enhancement

Avoid:

```text id="rw226"
oversharpening
```

which exaggerates edge artifacts.

---

# Suggested Rust Architecture

## Module Layout

```text id="rw227"
rw2/
 ├── tiff_parser
 ├── ifd
 ├── metadata
 ├── maker_notes
 ├── jpeg_extract
 ├── raw_unpack
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

## Phase 3 — High-End Fidelity

Implement:

* Panasonic-specific lens correction
* GPU acceleration
* tiled rendering
* ROI decode
* advanced denoise

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

`.rw2` contains:

* proprietary packing
* Panasonic-specific MakerNotes
* generation-dependent metadata
* predictive compression

LibRaw already handles:

* Bayer unpacking
* CFA interpretation
* metadata extraction
* black level correction

Recommended architecture:

```text id="rw228"
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

Moderate workload because of:

* packed Bayer decoding
* demosaicing
* lens corrections
* high-resolution sensors

Generally simpler than:

* X3F
* CR3
* JPEG XL RAW workflows

---

# Panasonic Rendering Characteristics

## Tonal Rendering

Panasonic rendering often emphasizes:

* crisp detail
* vivid colors
* strong local contrast

Requires:

```text id="rw229"
high precision internal processing
```

Avoid:

```text id="rw230"
8-bit intermediate stages
```

until final export.

---

## Lens Correction Dependency

Panasonic workflows heavily depend on:

* software correction
* digital distortion compensation
* chromatic aberration correction

Ignoring these may produce:

* warped geometry
* strong CA
* edge softness

---

# Uncertain Points

## 1. Exact Compression Algorithms

Some Panasonic predictive schemes remain partially undocumented.

Different generations may use:

* different predictors
* different packing layouts

---

## 2. MakerNote Semantics

Many proprietary tags remain undocumented.

Possible contents:

* autofocus metadata
* lens calibration
* stabilization vectors
* scene detection

---

## 3. Sensor Generation Differences

Older CCD and newer CMOS systems differ significantly:

* noise behavior
* black level handling
* dynamic range

---

## 4. Leica Shared Technology

Some Leica cameras use Panasonic-derived RAW pipelines.

This may affect:

* metadata interpretation
* CFA handling
* color science

---

## 5. Exact Panasonic Rendering Pipeline

Panasonic software likely applies:

* proprietary lens correction
* custom sharpening
* tone mapping
* color transforms

Perfect reproduction is difficult.

---

# Other informations

## MIME Type

Commonly observed:

```text id="rw231"
image/x-panasonic-rw2
```

Not formally standardized.

---

# Cameras Using RW2

Examples:

* Panasonic Lumix GH series
* Panasonic Lumix G series
* Panasonic Lumix S series
* Panasonic compact systems
* some Leica/Panasonic collaborative cameras

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="rw232"
RGB16 linear
```

Preferred for HDR workflows:

```text id="rw233"
RGBA16F
```

Avoid:

```text id="rw234"
8-bit intermediate processing
```

until final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="rw235"
WebP lossy
```

## Editing cache

```text id="rw236"
16-bit TIFF
```

## GPU visualization

```text id="rw237"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Embedded JPEG extraction

Highest ROI.

### 2. Packed Bayer decoder

Core technical challenge.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. Lens correction support

Critical for Panasonic image fidelity.

### 5. Accurate demosaicing

Essential for preserving detail quality.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="rw238"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="rw239"
LibRaw + high precision Bayer RAW pipeline
```

## Native `.rw2` decoder implementation

Should be considered:

```text id="rw240"
medium complexity
```

because `.rw2` combines:

* TIFF-derived structures
* packed Bayer RAW streams
* proprietary MakerNotes
* predictive compression
* lens-correction-dependent rendering
* generation-specific metadata behavior.
