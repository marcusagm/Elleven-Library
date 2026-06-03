# DirectDraw Surface (`.dds`) File Format Technical Specification

## Format Overview

* **Extension Name**:

  * `.dds`

* **Possible Origin**:

  * Developed by Microsoft as part of the DirectX texture pipeline.
  * Introduced with DirectDraw and later expanded for Direct3D texture storage.

* **Category**:

  * GPU Texture Container Format
  * Raster Image Format
  * Compressed Texture Container
  * Mipmap Container
  * Cubemap Container
  * Volume Texture Container

* **LibRaw Support**:

  * No

* **FFMPEG Support**:

  * Partial
  * Can decode many DDS variants through texture/image decoders.
  * Support depends heavily on the specific texture compression format used.

* **Rust alternative converters**:

  * `image_dds`
  * `ddsfile`
  * `image`
  * `intel_tex`
  * `bcdec-rs`
  * `basis-universal`
  * `wgpu`
  * `ash`
  * `glium`
  * External tools:

    * `texconv`
    * `compressonator`
    * `nvtt`
    * `DirectXTex`
    * `ImageMagick`
    * `RenderDoc`
    * `Noesis`
    * `ffmpeg`

DDS is one of the most important texture formats in computer graphics.

Unlike conventional image formats:

```text
JPEG
PNG
WebP
AVIF
```

DDS was designed primarily for:

* GPU upload
* real-time rendering
* game engines
* texture streaming
* mipmapping
* compressed texture storage

DDS commonly stores:

* diffuse textures
* normal maps
* roughness maps
* metallic maps
* cubemaps
* environment maps
* skyboxes
* volume textures
* texture arrays

DDS is fundamentally:

```text
GPU-oriented texture storage
```

rather than a general-purpose image format.

---

# File structure

## High-Level Container Layout

Classic DDS layout:

```text
+----------------------+
| DDS Magic            |
+----------------------+
| DDS_HEADER           |
+----------------------+
| DDS_HEADER_DXT10?    |
+----------------------+
| Texture Data         |
+----------------------+
| Mipmaps              |
+----------------------+
```

DDS is intentionally simple.

Most complexity resides in:

```text
pixel format interpretation
```

rather than container structure.

---

# Magic Header

Every DDS file begins with:

```hex
44 44 53 20
```

ASCII:

```text
DDS
```

Including trailing space:

```text
"DDS "
```

---

# DDS_HEADER

Immediately follows magic.

Structure:

```c
struct DDS_HEADER {
    uint32 size;
    uint32 flags;
    uint32 height;
    uint32 width;
    uint32 pitchOrLinearSize;
    uint32 depth;
    uint32 mipMapCount;
    uint32 reserved1[11];
    DDS_PIXELFORMAT ddspf;
    uint32 caps;
    uint32 caps2;
    uint32 caps3;
    uint32 caps4;
    uint32 reserved2;
};
```

Size:

```text
124 bytes
```

Always verify.

---

# DDS_PIXELFORMAT

Core structure:

```c
struct DDS_PIXELFORMAT {
    uint32 size;
    uint32 flags;
    uint32 fourCC;
    uint32 RGBBitCount;
    uint32 RBitMask;
    uint32 GBitMask;
    uint32 BBitMask;
    uint32 ABitMask;
};
```

This field determines:

* pixel layout
* compression
* texture interpretation

---

# DX10 Extended Header

Introduced for:

* DirectX 10
* DirectX 11
* modern GPU formats

Detected when:

```text
fourCC == "DX10"
```

Additional structure:

```c
struct DDS_HEADER_DXT10 {
    DXGI_FORMAT dxgiFormat;
    uint32 resourceDimension;
    uint32 miscFlag;
    uint32 arraySize;
    uint32 miscFlags2;
};
```

Size:

```text
20 bytes
```

---

# Texture Types

DDS may contain:

## 2D Textures

Most common.

```text
width × height
```

---

## Mipmapped Textures

Multiple resolutions:

```text
1024x1024
512x512
256x256
...
1x1
```

Stored sequentially.

---

## Cubemaps

Six faces:

```text
+X
-X
+Y
-Y
+Z
-Z
```

Used for:

* skyboxes
* reflections
* environment probes

---

## Volume Textures

Three-dimensional textures:

```text
width × height × depth
```

Common in:

* medical imaging
* voxel rendering
* fog systems

---

## Texture Arrays

DX10+ feature.

Contains:

```text
N independent textures
```

sharing format and dimensions.

---

# Pixel Format Categories

DDS supports an enormous number of texture formats.

---

# Uncompressed Formats

## RGB24

```text
R8 G8 B8
```

Rare.

---

## RGBA32

```text
R8 G8 B8 A8
```

Very common.

---

## BGRA32

```text
B8 G8 R8 A8
```

Extremely common.

---

## RGB565

```text
16-bit
```

Layout:

```text
RRRRRGGGGGGBBBBB
```

---

## ARGB1555

```text
1-bit alpha
```

---

## ARGB4444

```text
4 bits per channel
```

---

# Block Compression Formats

Most important DDS feature.

---

## BC1 / DXT1

FourCC:

```text
DXT1
```

Characteristics:

* 4×4 blocks
* 8 bytes per block
* RGB
* optional 1-bit alpha

Compression ratio:

```text
6:1
```

Approximate.

---

## BC2 / DXT3

FourCC:

```text
DXT3
```

Characteristics:

* explicit alpha
* 16 bytes per block

---

## BC3 / DXT5

FourCC:

```text
DXT5
```

Characteristics:

* interpolated alpha
* most common legacy DDS

---

## BC4

Single-channel.

Used for:

* heightmaps
* masks

---

## BC5

Two-channel.

Common for:

* tangent-space normal maps

Industry standard.

---

## BC6H

HDR texture compression.

Supports:

* floating point HDR

Used in:

* environment maps
* IBL pipelines

---

## BC7

Modern high-quality compression.

Characteristics:

* excellent quality
* alpha support
* modern game engines

Widely preferred today.

---

# DXGI Formats

DX10 DDS supports hundreds of formats.

Examples:

```text
R8_UNORM
RG8_UNORM
RGBA8_UNORM
RGBA8_SRGB
RGBA16_FLOAT
RGBA32_FLOAT
BC1_UNORM
BC3_UNORM
BC5_UNORM
BC6H_UF16
BC7_UNORM
```

DXGI is effectively:

```text
modern DDS format identification
```

---

# Mipmap Layout

Mip levels stored sequentially:

```text
Level 0
Level 1
Level 2
...
Level N
```

Each level size depends on:

* format
* dimensions
* compression

---

# Cubemap Layout

Face order:

```text
+X
-X
+Y
-Y
+Z
-Z
```

Each face may contain:

```text
all mip levels
```

before next face.

Implementation must verify actual ordering.

---

# Strategy for Thumbnail Generation

## Recommended Architecture

### Tier 1 — Highest Resolution Mipmap

Use:

```text
Mip Level 0
```

Pipeline:

```text
DDS
 → Decode texture
 → Convert to RGBA
 → Resize
 → Encode WebP
```

Best quality.

---

### Tier 2 — Use Existing Mipmap

For huge textures:

```text
8192×8192
16384×16384
```

Choose closest mip level.

Pipeline:

```text
DDS
 → Select mip
 → Decode
 → Encode WebP
```

Much faster.

---

# Recommended Rust Thumbnail Pipeline

## Suggested Crates

```toml
ddsfile
image_dds
image
fast_image_resize
webp
rayon
```

For BC decompression:

```toml
intel_tex
bcdec-rs
```

---

# Strategy for Visualization

## Important Principle

DDS is:

```text
texture-centric
```

rather than image-centric.

Visualization requires:

1. Container parsing
2. Texture format detection
3. Texture decompression
4. Color-space interpretation

---

# Recommended Visualization Pipeline

## Stage 1 — Header Parsing

Read:

* DDS magic
* DDS_HEADER
* DX10 header

Validate:

```text
sizes
flags
offsets
```

---

## Stage 2 — Format Detection

Determine:

* Uncompressed
* BC1
* BC3
* BC5
* BC7
* Float
* Integer

Most important stage.

---

## Stage 3 — Texture Decode

Dispatch decoder:

```text
BC1 decoder
BC3 decoder
BC5 decoder
BC6H decoder
BC7 decoder
```

or:

```text
raw pixel decoder
```

---

## Stage 4 — Normal Map Detection

Many DDS files are:

```text
not color images
```

Examples:

* normal maps
* masks
* roughness maps

Normal maps should:

```text
not receive color correction
```

---

## Stage 5 — Color Space Detection

Critical.

Possible spaces:

```text
Linear
sRGB
HDR
```

Incorrect handling causes:

* washed-out textures
* dark textures
* incorrect gamma

---

## Stage 6 — HDR Processing

BC6H formats may contain:

```text
HDR floating point data
```

Recommended pipeline:

```text
Float decode
 → Tone mapping
 → Display transform
```

---

## Stage 7 — Alpha Reconstruction

For:

```text
DXT3
DXT5
BC7
RGBA
```

Preserve:

```text
straight alpha
```

when exporting.

---

## Stage 8 — GPU Upload

Preferred formats:

```text
RGBA8
RGBA16F
RGBA32F
```

depending on source.

---

# BC Compression Details

## BC1

Each block:

```text
4×4 pixels
8 bytes
```

Stores:

* two endpoints
* interpolation indices

---

## BC3

Each block:

```text
16 bytes
```

Contains:

* alpha block
* BC1 color block

---

## BC5

Contains:

```text
Red block
Green block
```

Often reconstructs:

```text
Blue normal component
```

using:

z=\sqrt{1-x^2-y^2}

---

## BC6H

Most complex DDS compression.

Supports:

* HDR
* half-float reconstruction
* signed/unsigned modes

---

## BC7

High-quality block compression.

Contains:

* multiple encoding modes
* partitions
* endpoint interpolation

Most difficult mainstream DDS format.

---

# Suggested Rust Architecture

## Module Layout

```text
dds/
 ├── header
 ├── dx10
 ├── formats
 ├── bc1
 ├── bc3
 ├── bc4
 ├── bc5
 ├── bc6h
 ├── bc7
 ├── mipmaps
 ├── cubemaps
 ├── arrays
 ├── color
 ├── thumbnail
 ├── webp_export
 └── cache
```

---

# Recommended Initial Strategy

## Phase 1

Implement:

* RGBA8
* BGRA8
* DXT1
* DXT3
* DXT5

Covers majority of legacy DDS files.

---

## Phase 2

Add:

* BC4
* BC5
* Cubemaps
* Mipmaps

Covers most game textures.

---

## Phase 3

Add:

* BC6H
* BC7
* DX10 arrays
* HDR workflows

Required for modern engines.

---

# Performance Characteristics

## Decode Speed

Uncompressed DDS:

```text
extremely fast
```

BC formats:

```text
moderately fast
```

BC6H and BC7:

```text
computationally expensive
```

---

## Memory Usage

Compressed DDS:

```text
very compact
```

Decoded textures:

```text
large
```

especially:

```text
8K
16K
HDR
```

textures.

---

# Recommended Internal Pixel Formats

## Standard Visualization

Use:

```text
RGBA8
```

---

## HDR Visualization

Use:

```text
RGBA16F
```

Preferred for:

* BC6H
* float DDS

---

## Editing Pipeline

Use:

```text
RGBA16
RGBA16F
```

to avoid precision loss.

---

# Recommended Cache Formats

## Thumbnail Cache

```text
WebP lossy
```

---

## Intermediate Cache

```text
PNG
WebP lossless
```

---

## HDR Cache

```text
EXR
AVIF HDR
```

---

# Uncertain Points

## 1. Legacy DDS Variants

Older DDS writers occasionally:

* misuse flags
* omit fields
* store invalid mip counts

Robust parsing required.

---

## 2. Cubemap Ordering

Some tools historically generated:

* incorrect face order
* invalid caps flags

Validation recommended.

---

## 3. BC6H Decoder Compatibility

BC6H implementations sometimes differ in:

* rounding
* endpoint reconstruction

Small visual differences are expected.

---

## 4. BC7 Mode Handling

BC7 contains numerous encoding modes.

Malformed DDS files may exploit edge cases.

---

## 5. Color Space Metadata

Many DDS files lack explicit:

```text
sRGB
Linear
HDR
```

metadata.

Heuristics may be necessary.

---

# Other informations

## MIME Types

Commonly observed:

```text
image/vnd-ms.dds
```

Also:

```text
image/x-dds
```

---

# Typical Use Cases

DDS is widely used in:

* video games
* DirectX engines
* Unreal Engine asset pipelines
* Unity asset pipelines
* CAD visualization
* scientific visualization
* environment maps
* GPU texture streaming

---

# DDS vs PNG

DDS advantages:

* GPU-ready
* mipmaps
* compressed textures
* cubemaps

DDS disadvantages:

* poor browser support
* format complexity
* texture-specific semantics

---

# DDS vs KTX2

KTX2 increasingly replaces DDS in cross-platform graphics because it provides:

* Vulkan-native workflows
* Basis Universal compression
* better portability

DDS remains dominant in:

```text
DirectX ecosystems
```

---

# Recommended Development Priorities

## Most Important

### 1. BC1 / DXT1 support

Highest compatibility impact.

### 2. BC3 / DXT5 support

Extremely common.

### 3. Mipmap selection

Critical for thumbnail generation.

### 4. BC5 normal map support

Common in games.

### 5. BC7 support

Required for modern DDS assets.

---

# Most Important Practical Insight

For production-grade implementations:

## Thumbnail generation

Use:

```text
nearest existing mipmap level
```

when available, instead of decoding the full-resolution texture.

## High-quality rendering

Use:

```text
BC-aware decompression
+ proper color-space handling
```

## Native `.dds` decoder implementation

Should be considered:

```text
medium-to-high complexity
```

because DDS itself is simple, but the texture formats it contains are not. The real challenge lies in supporting:

* BC1–BC7 compression families
* HDR formats
* cubemaps
* texture arrays
* mipmap chains
* DX10/DXGI extensions
* linear vs sRGB workflows
* normal map semantics

rather than the DDS container structure itself.
