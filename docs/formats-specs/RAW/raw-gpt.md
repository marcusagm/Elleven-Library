# Leica RAW (`.raw`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.raw`
* **Possible Origin**: Proprietary RAW format used by several Leica digital camera systems, especially older Leica compact, bridge, and medium-format cameras
* **Category**: RAW / Digital Camera Sensor Data Container
* **LibRaw Support**: Partial to good support depending on camera generation
* **FFMPEG Support**: No native Leica RAW decoder; indirect support through LibRaw or external RAW processing tools
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
    * `ImageMagick`
    * `exiftool`

The Leica `.raw` format is problematic from an implementation perspective because:

```text id="lraw01"
".raw" is not globally unique
```

Many manufacturers and software systems use:

```text id="lraw02"
.raw
```

for entirely different formats.

For Leica specifically, `.raw` has historically been used in:

* Leica Digilux systems
* Leica compact cameras
* Leica medium-format systems
* some rebadged Panasonic collaborations

The actual internal structure varies significantly depending on:

* camera generation
* sensor manufacturer
* Leica/Panasonic collaboration period
* whether the file predates DNG adoption

Modern Leica systems mostly standardized on:

```text id="lraw03"
DNG
```

but older systems still use proprietary `.raw`.

---

# File structure

## High-Level Container Layout

Typical Leica `.raw` structure:

```text id="lraw04"
+----------------------+
| File Header          |
+----------------------+
| TIFF-like Directories|
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

Many Leica `.raw` files are:

```text id="lraw05"
TIFF-derived RAW containers
```

similar to:

* Panasonic RAW
* early TIFF/EP variants
* proprietary Bayer containers

However, exact structure differs substantially across models.

---

# Important Format Ambiguity

## Critical Implementation Warning

The extension:

```text id="lraw06"
.raw
```

alone is insufficient to identify the format.

Detection must rely on:

* magic bytes
* EXIF Make/Model
* internal TIFF tags
* metadata structure

Recommended identification pipeline:

```text id="lraw07"
extension check
    ↓
magic byte detection
    ↓
EXIF Make/Model validation
    ↓
decoder selection
```

---

# Main Structural Components

## 1. File Header

Usually contains:

* endian marker
* TIFF-like identifiers
* directory offsets

Common TIFF magic:

Little-endian:

```hex id="lraw08"
49 49 2A 00
```

Big-endian:

```hex id="lraw09"
4D 4D 00 2A
```

Some Leica/Panasonic variants are strongly TIFF-compliant internally.

---

## 2. TIFF-Like Directory Structures

Contains:

* RAW offsets
* image dimensions
* compression metadata
* preview references
* EXIF references

Conceptually similar to TIFF IFDs:

```c id="lraw10"
struct IFD_ENTRY {
    uint16 tag;
    uint16 type;
    uint32 count;
    uint32 value_or_offset;
}
```

---

## 3. RAW Sensor Data

Depending on generation:

* Bayer CFA
* CCD or CMOS
* packed RAW streams
* lossless compressed RAW

Observed bit depths:

* 10-bit
* 12-bit
* 14-bit

Common CFA:

```text id="lraw11"
RGGB
```

Some Panasonic-derived Leica systems may use:

```text id="lraw12"
different CFA ordering
```

such as:

* BGGR
* GBRG

---

## 4. Embedded JPEG Preview

Most Leica RAW files contain:

* embedded JPEG preview
* EXIF thumbnail
* camera-rendered image

This is the preferred source for:

* thumbnails
* galleries
* fast previews

Advantages:

* extremely fast
* preserves Leica rendering
* low CPU usage

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
* white balance
* orientation
* timestamp

Rust recommendations:

```toml id="lraw13"
kamadak-exif
little_exif
```

---

## 6. Leica MakerNotes

Contains proprietary metadata.

Possible contents:

* lens correction
* sensor calibration
* Leica color matrices
* focus metadata
* image stabilization data

Documentation is limited.

Different Leica generations may use:

* different MakerNote structures
* Panasonic-compatible metadata
* proprietary offsets

---

# RAW Sensor Characteristics

## Bayer RAW Pipeline

Most Leica RAW systems use:

```text id="lraw14"
single-layer Bayer CFA sensors
```

Requires:

* demosaicing
* white balance
* color transforms

---

# Compression Behavior

## Observed Compression Modes

### 1. Uncompressed RAW

Common in older Leica systems.

Advantages:

* simpler parsing
* easier reverse engineering

Disadvantages:

* large file sizes

---

### 2. Lossless Compression

Observed in newer variants.

Likely includes:

* predictive coding
* packed Bayer streams
* Panasonic-derived compression

LibRaw abstracts much of this complexity.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (Preferred)

Pipeline:

```text id="lraw15"
RAW
 └── parse TIFF structures
      └── locate JPEG preview
           └── decode JPEG
                └── resize
                     └── encode WebP
```

Advantages:

* extremely fast
* low CPU cost
* preserves Leica rendering

Ideal for:

* galleries
* Tauri applications
* file explorers
* lazy loading

---

## Tier 2 — EXIF Thumbnail

Fallback only.

Usually:

```text id="lraw16"
160–512px
```

Often insufficient for modern UIs.

---

## Tier 3 — Full RAW Decode

Required for:

* editing
* exposure recovery
* zoom rendering
* high-fidelity previews

Pipeline:

```text id="lraw17"
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

```toml id="lraw18"
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

```text id="lraw19"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="lraw20"
Quality: 90–95
High-quality lossy or lossless
```

---

# Strategy for Visualization

## Important Principle

Embedded JPEG previews are insufficient for:

* highlight recovery
* white balance editing
* full dynamic range
* Leica RAW fidelity

Proper visualization requires RAW decoding.

---

# Recommended Visualization Pipeline

## Stage 1 — Format Identification

Critical.

Because:

```text id="lraw21"
.raw is ambiguous
```

Detection should include:

* magic bytes
* EXIF Make
* EXIF Model
* TIFF tag inspection

---

## Stage 2 — TIFF Parsing

Read:

* IFD structures
* RAW offsets
* metadata
* preview locations

Recommended crates:

```toml id="lraw22"
binrw
nom
tiff
```

Preferred:

```toml id="lraw23"
binrw
```

---

## Stage 3 — RAW Extraction

Read:

* Bayer RAW
* packed bit streams
* compressed blocks

Challenges:

* camera-generation variability
* Panasonic-derived formats
* packed pixel decoding

---

## Stage 4 — Black Level Correction

Critical.

Without this:

* lifted shadows
* color shifts
* incorrect tonal response

---

## Stage 5 — White Balance

Apply:

* camera multipliers
* neutral references
* calibration matrices

---

## Stage 6 — Demosaicing

Recommended algorithms:

* AMaZE
* RCD
* DCB

For maximum quality:

```text id="lraw24"
AMaZE or RCD
```

especially for Leica lenses and optics, which preserve:

* fine microcontrast
* subtle edge transitions
* detailed textures

---

## Stage 7 — Lens Correction

Important for:

* geometric correction
* vignetting
* chromatic aberration

Possible implementation:

* Lensfun
* Leica profiles
* empirical calibration

---

## Stage 8 — Color Space Conversion

Recommended pipeline:

```text id="lraw25"
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

## Stage 9 — Tone Mapping

Leica systems often preserve:

* smooth highlight rolloff
* natural contrast
* film-like rendering

Recommended:

* filmic
* ACES
* Reinhard

Avoid:

```text id="lraw26"
aggressive local contrast enhancement
```

which damages Leica rendering aesthetics.

---

## Stage 10 — Noise Reduction

Recommended:

* chroma-first denoise
* preserve luminance texture
* avoid plastic smoothing

---

## Stage 11 — Sharpening

Leica optics emphasize:

* microcontrast
* natural acuity

Use:

* restrained sharpening
* low-radius enhancement

Avoid:

```text id="lraw27"
oversharpening
```

which destroys Leica optical character.

---

# Suggested Rust Architecture

## Module Layout

```text id="lraw28"
leica_raw/
 ├── detector
 ├── tiff_parser
 ├── ifd
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

* Leica RAW detection
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

* Leica-specific color rendering
* lens correction
* GPU acceleration
* tiled rendering
* ROI decoding

---

# LibRaw Integration Strategy

Strongly recommended.

## Why

Leica `.raw` variants may differ significantly by:

* camera generation
* Panasonic partnership generation
* compression scheme
* metadata structure

LibRaw already supports:

* Bayer unpacking
* CFA handling
* metadata extraction
* black level correction

Recommended architecture:

```text id="lraw29"
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

* packed Bayer streams
* demosaicing
* high bit depth
* lens corrections

But generally simpler than:

* X3F
* CR3
* JPEG XL

---

# Leica Rendering Characteristics

## Tonal Rendering

Leica rendering often emphasizes:

* natural contrast
* smooth highlights
* subtle tonal separation

Requires:

```text id="lraw30"
high precision processing
```

Avoid:

```text id="lraw31"
8-bit intermediate stages
```

until final output.

---

## Color Science

Leica rendering typically emphasizes:

* natural colors
* realistic skin tones
* restrained saturation
* film-like aesthetics

Exact reproduction may require:

* proprietary matrices
* empirical profiling
* Adobe/Leica DNG comparisons

---

# Uncertain Points

## 1. Format Ambiguity

`.raw` is not unique to Leica.

Detection must not rely solely on extension.

---

## 2. Compression Variants

Different Leica generations may use:

* Panasonic-derived compression
* proprietary packing
* uncompressed Bayer streams

---

## 3. MakerNote Semantics

Many Leica MakerNote tags remain undocumented.

Possible contents:

* lens calibration
* stabilization metadata
* rendering hints

---

## 4. Panasonic Shared Technology

Some Leica systems share:

* sensors
* RAW pipelines
* metadata conventions

with Panasonic systems.

This may affect:

* CFA interpretation
* compression handling
* color rendering

---

## 5. Exact Leica Rendering Pipeline

Leica software likely applies:

* proprietary tone curves
* lens-specific rendering
* custom highlight reconstruction

Perfect reproduction is difficult.

---

# Other informations

## MIME Type

Commonly observed:

```text id="lraw32"
image/x-leica-raw
```

Not formally standardized.

---

# Cameras Using Leica RAW

Examples:

* Leica Digilux series
* older Leica compact systems
* some Leica/Panasonic collaboration cameras

Modern Leica systems typically use:

```text id="lraw33"
DNG
```

instead of proprietary `.raw`.

---

# Recommended Internal Pixel Formats

## Processing

Use:

```text id="lraw34"
RGB16 linear
```

Preferred for HDR workflows:

```text id="lraw35"
RGBA16F
```

Avoid:

```text id="lraw36"
8-bit intermediate processing
```

until final export.

---

# Recommended Cache Formats

## Thumbnail cache

```text id="lraw37"
WebP lossy
```

## Editing cache

```text id="lraw38"
16-bit TIFF
```

## GPU visualization

```text id="lraw39"
RGBA16F
```

---

# Recommended Development Priorities

## Most Important

### 1. Reliable format detection

Critical because `.raw` is ambiguous.

### 2. Embedded JPEG extraction

Highest ROI.

### 3. LibRaw integration

Avoids major reverse-engineering effort.

### 4. High precision processing

Critical for Leica tonal rendering.

### 5. Accurate demosaicing

Essential for preserving Leica optical character.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="lraw40"
embedded JPEG preview extraction
```

## High-quality rendering

Use:

```text id="lraw41"
LibRaw + high precision Bayer RAW pipeline
```

## Native Leica `.raw` decoder implementation

Should be considered:

```text id="lraw42"
medium-to-high complexity
```

because Leica `.raw` combines:

* ambiguous container identification
* TIFF-derived RAW structures
* multiple camera-generation variants
* proprietary metadata
* Panasonic-derived technology in some systems
* calibration-dependent rendering behavior.
