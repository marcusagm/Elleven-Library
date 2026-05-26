# Sigma / Foveon RAW (`.x3f`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.x3f`
* **Possible Origin**: Proprietary RAW image format developed by Sigma Corporation for cameras using the Foveon X3 sensor architecture.
* **Category**: RAW / Digital Negative / Sensor Dump Container
* **LibRaw Support**: Yes (partial-to-good support depending on camera generation)
* **FFMPEG Support**: Partial indirect support through `libraw` integration or external conversion pipeline; native decoding support is limited/nonexistent
* **Rust alternative converters**:

  * `libraw-rs` (Rust bindings for LibRaw)
  * External process invocation:

    * `dcraw`
    * `rawtherapee-cli`
    * `darktable-cli`
    * `x3f_extract`
    * `x3f-tools`
    * `ImageMagick` (via delegates)
  * Native Rust image pipeline:

    * `image`
    * `fast_image_resize`
    * `ravif`
    * `webp`
    * `zune-image`
    * `kamadak-exif`

The `.x3f` format is primarily associated with Sigma DSLR and compact cameras using the Foveon X3 layered sensor architecture. Unlike Bayer CFA sensors, Foveon sensors capture RGB information at each spatial location through stacked photodiodes. ([Vertopal][1])

This distinction heavily affects:

* RAW decoding
* Demosaicing strategy
* Thumbnail extraction
* Color science
* Noise handling
* Conversion fidelity

The format is proprietary and only partially documented publicly. Sigma historically encrypted or obfuscated portions of calibration and processing metadata. ([kronometric.org][2])

---

# File structure

## High-Level Container Layout

The `.x3f` format is a chunk-based binary container.

Typical structure:

```text
+----------------------+
| File Header          |
+----------------------+
| Directory Structure  |
+----------------------+
| Image Sections       |
|  - RAW sensor data   |
|  - JPEG preview      |
|  - Thumbnail         |
+----------------------+
| Metadata Sections    |
|  - EXIF              |
|  - Camera settings   |
|  - Calibration data  |
+----------------------+
| CAMF encrypted block |
+----------------------+
```

---

## Magic Header

Typical files begin with:

```text
FOVb
```

This identifies:

* Foveon container
* Binary structured format

Observed signatures:

```hex
46 4F 56 62
```

Meaning:

```text
"FOVb"
```

---

## Endianness

* Usually little-endian
* Some substructures may use independent endian interpretation

Recommendation:

```rust
byteorder = "1"
```

---

## Core Structural Concepts

The X3F format behaves similarly to:

* TIFF-like chunk organization
* RIFF-like section traversal

But it is NOT TIFF-based.

Instead, it contains:

* Typed sections
* Directory entries
* Offset tables
* Independent binary payloads

---

# Main Internal Sections

## 1. File Header

Contains:

* Magic
* Format version
* Directory offsets
* Section count

Probable structure:

```c
struct X3F_HEADER {
    char magic[4];        // "FOVb"
    uint32 version;
    uint32 directory_offset;
}
```

---

## 2. Directory Table

Acts as central lookup table.

Contains entries for:

* RAW image blocks
* JPEG previews
* Thumbnails
* Metadata
* CAMF blocks

Typical fields:

* type id
* offset
* length

---

## 3. Image Sections

X3F commonly contains multiple embedded image representations.

Typical payloads:

* Full RAW sensor data
* Medium preview
* JPEG preview
* Thumbnail preview

This is important because thumbnail extraction usually does NOT require RAW decoding.

---

## 4. RAW Sensor Block

This is the most difficult section.

Characteristics:

* Proprietary encoding
* Foveon layered RGB capture
* Often compressed or obfuscated
* Generation-specific layouts

Unlike Bayer RAW:

* There is no CFA interpolation
* Pixels already contain stacked RGB samples

LibRaw internally exposes these files differently:

* `raw_image == NULL`
* `color3_image != NULL`

This indicates:

* Full RGB-per-pixel RAW representation instead of Bayer mosaic. ([libraw.org][3])

---

## 5. JPEG Preview Block

Most X3F files include:

* Embedded JPEG preview
* Usually camera-rendered
* Often high quality

This is the best source for:

* Fast thumbnail extraction
* Gallery previews
* File browsing

Advantages:

* Extremely fast
* No RAW processing required
* Preserves Sigma camera color science

Disadvantages:

* Already tone-mapped
* Not linear RAW
* Lower editing latitude

---

## 6. Thumbnail Block

Usually contains:

* Small JPEG
* Sometimes RGB bitmap
* Used for camera browsing

Typical sizes:

* 160px
* 320px
* 640px

---

## 7. EXIF Metadata

Contains:

* ISO
* Exposure
* Lens
* Timestamp
* White balance
* Camera model

Often compatible with:

* EXIF parsers
* TIFF metadata readers

Rust recommendations:

```toml
kamadak-exif
little_exif
```

---

## 8. CAMF Section

One of the most important proprietary blocks.

Mentioned in reverse-engineering documentation. ([kronometric.org][2])

Contains:

* Camera calibration
* Color matrices
* Noise parameters
* Lens data
* Processing hints

Often partially encrypted or obfuscated.

Observed identifiers:

```text
CAMF
SECc
FCEb
```

This section is critical for:

* Accurate color reproduction
* Proper RAW conversion
* Sigma Photo Pro equivalence

---

# Foveon Sensor Characteristics

This is fundamental to correct implementation.

## Bayer Sensors

Typical RAW:

```text
1 pixel = 1 color sample
```

Need demosaicing.

---

## Foveon Sensors

Foveon:

```text
1 pixel = R + G + B stacked samples
```

Implications:

* No demosaicing
* Higher color fidelity
* Different noise profile
* Different sharpening behavior

Challenges:

* Higher chroma noise
* Nonlinear response
* Complex color transforms

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Embedded JPEG Extraction (FASTEST)

Preferred method.

Pipeline:

```text
X3F
 └── locate JPEG preview
      └── decode JPEG
           └── resize
                └── encode WebP
```

Advantages:

* Extremely fast
* Stable
* Minimal memory
* Best for file explorers

Recommended for:

* Gallery view
* Tauri apps
* File managers
* Lazy loading

---

## Tier 2 — Embedded Thumbnail

Fallback only.

Advantages:

* Very fast

Disadvantages:

* Often low resolution
* Poor quality

---

## Tier 3 — Full RAW Decode

Use only when:

* Generating high-quality previews
* Zoom visualization
* Editing pipeline
* Export conversion

Pipeline:

```text
RAW decode
 → color transform
 → white balance
 → gamma
 → tone map
 → resize
 → WebP
```

This is computationally expensive.

---

## Recommended Rust Thumbnail Pipeline

### Architecture

```text
x3f parser
   ↓
extract jpeg preview
   ↓
jpeg decoder
   ↓
resize
   ↓
webp encoder
```

### Suggested crates

```toml
jpeg-decoder
image
fast_image_resize
webp
rayon
```

---

## Ideal Thumbnail Strategy

### Small gallery thumbnails

Source:

* Embedded JPEG preview

Output:

* Lossy WebP
* Quality 75–85

---

### Detail previews

Source:

* Full RAW decode

Output:

* High quality WebP
* Quality 90–95

---

# Strategy for Visualization

## Important Principle

The embedded JPEG preview is NOT sufficient for high-fidelity visualization.

To achieve:

* true Foveon rendering
* accurate color
* shadow recovery
* exposure adjustment

You must decode RAW data.

---

# Recommended Visualization Pipeline

## Stage 1 — Parse Container

Read:

* section table
* metadata
* image blocks

---

## Stage 2 — Decode RAW Sensor Data

Options:

* LibRaw
* dcraw
* reverse-engineered native decoder

Recommended initial approach:

```text
LibRaw wrapper
```

Because Sigma/Foveon decoding is highly nontrivial.

---

## Stage 3 — Apply Calibration

Critical:

* white balance
* color matrices
* black level
* channel scaling

Without this:

* severe color shifts occur

---

## Stage 4 — Linear RGB Reconstruction

Foveon RAW is usually:

* linear
* wide dynamic range

Need:

* linear RGB normalization

---

## Stage 5 — Tone Mapping

Required for display.

Suggested:

* filmic tone mapping
* Reinhard
* ACES

---

## Stage 6 — Color Space Conversion

Recommended output:

```text
linear RGB
→ ProPhoto / Rec2020 internal
→ sRGB display transform
```

---

## Stage 7 — Display Cache

Recommended:

* cache WebP previews
* cache mipmaps
* progressive rendering

---

# Suggested Rust Architecture

## Core Modules

```text
x3f/
 ├── parser
 ├── sections
 ├── metadata
 ├── jpeg_extract
 ├── raw_decode
 ├── color_pipeline
 ├── thumbnail
 ├── webp_export
 └── cache
```

---

## Parsing Layer

Use:

```rust
nom
binrw
zerocopy
```

Recommended:

```rust
binrw
```

---

## RAW Processing Layer

Recommended initial strategy:

```text
Rust frontend
    ↓
LibRaw FFI
    ↓
RGB16 linear output
```

Reason:

* native Foveon decode is extremely difficult

---

## WebP Export

Recommended:

* 10-bit or 8-bit tone-mapped output
* optional lossless WebP

Suggested:

```toml
libwebp-sys
webp
```

---

# Conversion Fidelity Considerations

## Sigma Photo Pro

Historically considered:

* reference implementation
* best Foveon rendering

But:

* slow
* proprietary
* Windows/macOS focused

Community commonly converts:

```text
X3F → TIFF/DNG
```

before editing. ([Reddit][4])

---

## DNG Conversion

Possible via:

* x3f_tools
* x3f_extract
* dcraw derivatives

Caveat:

* Some Sigma-specific information may be lost
* Color rendering changes
* Noise behavior changes

---

# Uncertain Points

## 1. Full RAW Compression Algorithm

Still not completely documented publicly.

Likely:

* generation-dependent
* partially obfuscated

---

## 2. CAMF Encryption

Known to exist.

Not fully documented. ([kronometric.org][2])

---

## 3. Exact Color Science

Sigma Photo Pro uses:

* proprietary transforms
* proprietary denoise
* proprietary tone curves

Exact reproduction is difficult.

---

## 4. Merrill vs Quattro Differences

Different sensor generations:

* SD9 / SD10
* Merrill
* Quattro

may use:

* different RAW layouts
* different calibration
* different channel structures

Need separate testing.

---

## 5. Bit Depth Variations

Observed:

* 10-bit
* 12-bit
* 14-bit variants

Not fully standardized across generations.

---

# Other informations

## MIME Type

```text
image/x-sigma-x3f
```

([Vertopal][1])

---

## Cameras Using X3F

Examples:

* Sigma SD9
* Sigma SD10
* Sigma SD14
* Sigma DP1
* Sigma DP2
* Sigma Merrill series
* Sigma Quattro series

---

## Performance Characteristics

Foveon RAW processing is significantly heavier than Bayer RAW because:

* full RGB data per pixel
* larger data footprint
* complex color transforms
* higher chroma noise processing

---

## Recommended Development Strategy

## Phase 1

Implement:

* container parser
* JPEG preview extraction
* EXIF extraction
* WebP thumbnail export

This yields:

* immediate practical utility
* fast browsing support

---

## Phase 2

Integrate:

* LibRaw decoding
* RAW visualization
* 16-bit pipeline

---

## Phase 3

Implement:

* native Foveon decode
* custom color science
* GPU acceleration

---

## Recommended Test Corpus

You should build a corpus containing:

* multiple Sigma generations
* ISO variations
* daylight/tungsten/night scenes
* overexposed/underexposed samples

Because Foveon rendering varies heavily by:

* sensor generation
* ISO
* lighting conditions

---

## Recommended Internal Pixel Format

For processing:

```text
RGB16 linear
```

Avoid:

```text
u8 processing
```

until final display/export stage.

---

## Recommended Cache Formats

### For thumbnails

```text
WebP lossy
```

### For editing cache

```text
16-bit TIFF
```

### For GPU visualization

```text
RGBA16F
```

---

## Most Important Practical Insight

For production-grade software:

### Thumbnail generation

Use:

```text
embedded JPEG preview
```

### High-quality visualization

Use:

```text
LibRaw + custom color pipeline
```

### Native X3F RAW decoding

Should be considered:

```text
advanced / research-grade implementation
```

due to incomplete public documentation and proprietary Sigma processing behavior. ([kronometric.org][2])

[1]: https://www.vertopal.com/en/format/x3f?utm_source=chatgpt.com "X3F File Type and Format Guide - Vertopal"
[2]: https://www.kronometric.org/phot/sensor/Foveon/X3F%20SIGMA%20Raw%20format%20documentation%20project.htm?utm_source=chatgpt.com "X3F SIGMA Raw format documentation project"
[3]: https://www.libraw.org/node/2225?utm_source=chatgpt.com "Sigma X3F CFA images | LibRaw"
[4]: https://www.reddit.com/r/infraredphotography/comments/1owmmex/red_palm_tree_on_foveon/?utm_source=chatgpt.com "Red palm tree on foveon"
