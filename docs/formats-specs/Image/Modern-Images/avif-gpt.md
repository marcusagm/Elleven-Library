# AV1 Image File Format (`.avif`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.avif`
* **Possible Origin**: Developed by the Alliance for Open Media (AOMedia) as an image container and profile using the AV1 codec inside the ISO Base Media File Format (ISOBMFF)
* **Category**: Modern Compressed Raster Image Container / HDR-Capable Image Format
* **LibRaw Support**: No
* **FFMPEG Support**: Yes (broad support through libavif/libaom/dav1d integrations depending on build configuration)
* **Rust alternative converters**:

  * `libavif-rs`
  * `ravif`
  * `image`
  * `rgb`
  * `dav1d`
  * `avif-decode`
  * `fast_image_resize`
  * `webp`
  * External tools:

    * `ffmpeg`
    * `ImageMagick`
    * `libavif`
    * `dav1d`
    * `aomenc`
    * `avifenc`
    * `avifdec`
    * `vips`

AVIF is a modern image format based on:

```text id="avif01"
AV1 intra-frame image compression
```

It is designed to provide:

* extremely high compression efficiency
* HDR support
* wide gamut support
* alpha support
* film grain
* animation
* modern metadata support

Compared to older formats:

* JPEG
* PNG
* WebP

AVIF usually achieves:

```text id="avif02"
smaller file sizes at similar visual quality
```

especially at:

* medium bitrates
* HDR workflows
* photographic content

AVIF is:

* container-based
* codec-based
* heavily metadata-driven
* color-management-oriented

Internally it combines:

```text id="avif03"
ISOBMFF container + AV1 bitstream
```

similar conceptually to:

* HEIF
* HEIC
* MP4

---

# File structure

## High-Level Container Layout

Typical AVIF structure:

```text id="avif04"
+----------------------+
| ftyp box             |
+----------------------+
| meta box             |
+----------------------+
| item properties      |
+----------------------+
| AV1 image item       |
+----------------------+
| metadata             |
+----------------------+
| auxiliary images     |
+----------------------+
```

AVIF uses:

```text id="avif05"
ISO Base Media File Format (ISOBMFF)
```

which is the same container family as:

* MP4
* HEIF
* HEIC

---

# Container Foundation

## ISOBMFF Box Architecture

AVIF is structured as:

```text id="avif06"
nested typed boxes
```

Each box:

```c id="avif07"
struct BOX {
    uint32 size;
    char type[4];
    uint8 payload[];
}
```

Large boxes may use:

```text id="avif08"
64-bit extended sizes
```

---

# File Type Box (`ftyp`)

Identifies format compatibility.

Common brands:

| Brand  | Meaning         |
| ------ | --------------- |
| `avif` | AVIF image      |
| `avis` | AVIF sequence   |
| `mif1` | HEIF-compatible |
| `miaf` | MIAF profile    |

Example:

```text id="avif09"
ftypavif
```

---

# Meta Box

Contains:

* image item declarations
* property associations
* color metadata
* auxiliary image references

Critical container structure.

---

# Item-Based Architecture

Unlike JPEG:

```text id="avif10"
AVIF is item-oriented
```

Images are stored as:

* primary items
* auxiliary items
* derived items

Possible auxiliary items:

* alpha planes
* depth maps
* thumbnails
* gain maps

---

# AV1 Bitstream

The actual image payload is:

```text id="avif11"
AV1 intra-frame compressed data
```

AVIF typically uses:

* intra-only AV1 frames
* no inter-frame prediction for still images

Though animated AVIF:

```text id="avif12"
supports temporal prediction
```

---

# Compression Characteristics

## AV1 Features Used by AVIF

Possible features:

* transform coding
* intra prediction
* directional prediction
* palette coding
* film grain synthesis
* chroma subsampling
* loop restoration
* CDEF filtering

Compression efficiency is:

```text id="avif13"
extremely high
```

but computational cost is also high.

---

# Color Depth Support

AVIF supports:

* 8-bit
* 10-bit
* 12-bit

Internally:

* YUV
* RGB-derived transforms
* HDR transfer functions

---

# HDR Support

One of AVIF’s major strengths.

Supports:

* HDR10
* HLG
* PQ
* wide gamut
* Rec.2020
* BT.2100

Possible metadata:

* mastering display
* content light levels
* ICC profiles
* NCLX profiles

---

# Alpha Channel Support

AVIF supports:

```text id="avif14"
full alpha channels
```

Alpha may be:

* embedded
* auxiliary image item
* separately compressed

Usually:

```text id="avif15"
lossless or near-lossless
```

---

# Chroma Subsampling

Possible formats:

| Format | Meaning                      |
| ------ | ---------------------------- |
| 4:4:4  | Full chroma                  |
| 4:2:2  | Horizontal subsampling       |
| 4:2:0  | Full video-style subsampling |

For:

* UI
* screenshots
* icons
* text

Prefer:

```text id="avif16"
4:4:4
```

For photography:

```text id="avif17"
4:2:0 or 4:2:2
```

may be acceptable.

---

# Film Grain Synthesis

AV1 supports:

```text id="avif18"
procedural film grain reconstruction
```

instead of storing explicit noise.

Advantages:

* reduced bitrate
* cinematic appearance

Challenges:

* deterministic reproduction
* GPU compatibility
* consistent rendering

---

# Animation Support

Animated AVIF:

```text id="avif19"
AVIF sequences
```

can behave similarly to:

* animated WebP
* GIF
* APNG

but with:

* much higher efficiency
* significantly higher decode complexity

---

# Metadata Support

AVIF supports:

* EXIF
* XMP
* ICC profiles
* NCLX color metadata

Metadata is usually stored in:

```text id="avif20"
ISOBMFF metadata boxes
```

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded Thumbnail Extraction

Some AVIF files may contain:

* reduced images
* thumbnail items
* auxiliary previews

Pipeline:

```text id="avif21"
AVIF
 └── parse item graph
      └── locate thumbnail item
           └── decode AV1 image
                └── encode WebP
```

Advantages:

* faster
* lower CPU usage

---

## Tier 2 — Full Decode + Resize

Most common practical strategy.

Pipeline:

```text id="avif22"
AVIF decode
 → YUV reconstruction
 → RGB conversion
 → resize
 → WebP encode
```

---

# Recommended Rust Thumbnail Pipeline

## Suggested Crates

```toml id="avif23"
ravif
image
rgb
fast_image_resize
webp
rayon
```

For decoding:

```toml id="avif24"
libavif-rs
dav1d
```

---

## Ideal WebP Settings

### Gallery thumbnails

```text id="avif25"
Quality: 70–85
Lossy WebP
```

### High-quality previews

```text id="avif26"
Quality: 90–100
Lossless or near-lossless
```

---

# Strategy for Visualization

## Important Principle

AVIF rendering quality depends heavily on:

* bit depth
* HDR metadata
* chroma format
* transfer functions
* tone mapping
* AV1 decoder quality

AVIF is:

```text id="avif27"
not just a simple bitmap format
```

---

# Recommended Visualization Pipeline

## Stage 1 — ISOBMFF Parsing

Read:

* box hierarchy
* item tables
* image properties
* metadata references

Recommended crates:

```toml id="avif28"
mp4parse
binrw
nom
```

---

## Stage 2 — AV1 Bitstream Extraction

Locate:

* primary image item
* alpha image
* auxiliary images

Extract:

```text id="avif29"
AV1 elementary bitstream
```

---

## Stage 3 — AV1 Decode

Critical stage.

Possible decoders:

* dav1d
* libaom
* rav1e decode layers
* hardware AV1 decoders

Recommended:

```text id="avif30"
dav1d
```

because:

* extremely fast
* highly optimized
* production-grade

---

## Stage 4 — Chroma Reconstruction

Convert:

* YUV420
* YUV422
* YUV444

into:

```text id="avif31"
linear RGB
```

Critical for:

* avoiding chroma artifacts
* preserving edges
* UI rendering

---

## Stage 5 — Bit Depth Handling

AVIF frequently uses:

* 10-bit
* 12-bit

Recommended internal formats:

```text id="avif32"
RGB16
RGBA16
RGBA16F
```

Avoid:

```text id="avif33"
8-bit intermediate conversion
```

until final output.

---

## Stage 6 — HDR Interpretation

Critical.

Possible transfer functions:

* PQ
* HLG
* sRGB
* gamma curves

Possible color primaries:

* Rec709
* Rec2020
* P3

Improper handling causes:

* washed-out images
* clipped highlights
* incorrect saturation

---

## Stage 7 — Tone Mapping

Required for:

* SDR displays
* WebP export
* thumbnails

Recommended:

* ACES
* Reinhard
* filmic

Avoid:

```text id="avif34"
naive clipping
```

---

## Stage 8 — Alpha Reconstruction

If alpha stored separately:

```text id="avif35"
decode alpha item
 → combine with RGB
```

Preserve:

* premultiplication semantics
* edge fidelity

---

## Stage 9 — Film Grain Reconstruction

Optional.

Some AVIF files depend on:

```text id="avif36"
procedural grain synthesis
```

Disabling it may alter intended appearance.

---

## Stage 10 — GPU Upload

Preferred formats:

```text id="avif37"
RGBA16F
RGBA8
```

HDR workflows:

```text id="avif38"
RGBA16F preferred
```

---

# Recommended Rust Architecture

## Module Layout

```text id="avif39"
avif/
 ├── isobmff
 ├── boxes
 ├── item_graph
 ├── av1_extract
 ├── av1_decode
 ├── chroma
 ├── hdr
 ├── tone_mapping
 ├── alpha
 ├── metadata
 ├── thumbnail
 ├── webp_export
 └── cache
```

---

# Recommended Initial Strategy

## Phase 1 — Practical Support

Implement:

* AVIF decode
* RGB conversion
* thumbnail generation
* WebP export

Prefer:

```text id="avif40"
libavif or dav1d integration
```

instead of native decoder development.

---

## Phase 2 — Advanced Rendering

Add:

* HDR support
* ICC handling
* alpha fidelity
* animated AVIF

---

## Phase 3 — High-End Pipeline

Implement:

* GPU decode
* ROI rendering
* film grain synthesis
* advanced tone mapping
* HDR display pipelines

---

# Performance Characteristics

## Decode Complexity

AVIF decode is:

```text id="avif41"
computationally expensive
```

compared to:

* JPEG
* PNG
* WebP

Especially:

* 10-bit
* 12-bit
* HDR
* animated AVIF

---

## Memory Characteristics

AVIF compressed size is small, but decode surfaces may be large.

Example:

```text id="avif42"
4K 12-bit RGBA16F
```

can consume substantial memory.

---

# Hardware Decode Support

Modern hardware increasingly supports:

* AV1 video decode
* partial AVIF acceleration

Support varies by:

* GPU generation
* operating system
* drivers

---

# AVIF vs WebP

## AVIF Advantages

Usually:

* smaller files
* better HDR support
* better compression efficiency

---

## AVIF Disadvantages

Usually:

* slower encode
* slower decode
* higher implementation complexity

---

# AVIF vs JPEG XL

AVIF strengths:

* ecosystem momentum
* AV1 ecosystem reuse

JPEG XL strengths:

* faster decode
* better lossless
* progressive workflows
* photographic fidelity

---

# HDR Workflow Considerations

AVIF is extremely suitable for:

* HDR galleries
* wide gamut imaging
* future-proof pipelines

Strong recommendation:

```text id="avif43"
maintain high precision internally
```

Use:

```text id="avif44"
16-bit or floating-point pipelines
```

---

# Recommended Internal Pixel Formats

## Standard Rendering

Use:

```text id="avif45"
RGBA16
```

## HDR Rendering

Use:

```text id="avif46"
RGBA16F
RGBA32F
```

Avoid:

```text id="avif47"
8-bit SDR intermediate conversion
```

---

# Recommended Cache Formats

## Thumbnail cache

```text id="avif48"
WebP lossy
```

## HDR cache

```text id="avif49"
AVIF or EXR
```

## Editing cache

```text id="avif50"
16-bit TIFF
```

---

# Uncertain Points

## 1. AV1 Decoder Variability

Different decoders may produce:

* slightly different grain synthesis
* chroma reconstruction differences
* edge filtering differences

---

## 2. HDR Metadata Interpretation

Applications vary in:

* tone mapping
* PQ handling
* HLG rendering
* display transforms

Perfect consistency is difficult.

---

## 3. Auxiliary Item Semantics

Some AVIF files may include:

* depth maps
* gain maps
* thumbnails
* alpha references

with ecosystem inconsistencies.

---

## 4. Film Grain Reproducibility

Film grain synthesis is:

```text id="avif51"
algorithmically reconstructed
```

Small implementation differences may alter appearance.

---

## 5. Animated AVIF Timing

Animated AVIF support differs across:

* browsers
* libraries
* playback engines

Timing semantics may vary.

---

# Other informations

## MIME Types

Common:

```text id="avif52"
image/avif
```

Animated AVIF:

```text id="avif53"
image/avif-sequence
```

may appear.

---

# Typical Use Cases

AVIF is increasingly used for:

* web delivery
* HDR imaging
* mobile applications
* modern galleries
* image CDNs
* bandwidth-sensitive systems

---

# Recommended Development Priorities

## Most Important

### 1. Reliable AV1 decode integration

Core dependency.

### 2. Correct HDR handling

Critical for fidelity.

### 3. Proper chroma reconstruction

Important for edge quality.

### 4. High precision pipeline

Essential for HDR workflows.

### 5. Efficient thumbnail generation

Important for UX responsiveness.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text id="avif54"
full AV1 decode + high quality resize
```

because embedded thumbnails are less common than in RAW formats.

## High-quality rendering

Use:

```text id="avif55"
high precision HDR-aware AV1 pipelines
```

## Native `.avif` decoder implementation

Should be considered:

```text id="avif56"
high complexity
```

because AVIF combines:

* ISOBMFF container semantics
* AV1 intra decoding
* HDR workflows
* high bit depth handling
* chroma subsampling
* auxiliary image graphs
* modern color management
* optional film grain synthesis
* animated sequencing support.
