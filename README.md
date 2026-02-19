<p align="center">
  <img src="public/branding/logo.svg" alt="Mundam Logo" width="400">
</p>

# Mundam

**Mundam** is a high-performance, local-first image reference manager designed specifically for artists, concept designers, and illustrators. It allows you to organize, tag, and view massive collections of reference images with zero lag, keeping your workflow uninterrupted.

Built with **Tauri v2**, **Rust**, and **SolidJS**, Mundam combines the raw power of native code with the reactivity of modern web interfaces.

### 💖 Support the Project

**Mundam** is a free, open-source project built to empower artists. If this tool helps your creative process and you'd like to support its continuous development and maintenance, consider making a donation. Your support keeps the project alive, independent, and free for everyone!

[![PayPal](https://www.paypalobjects.com/en_US/i/btn/btn_donate_LG.gif)](https://www.paypal.com/donate/?hosted_button_id=86CZGZQXBBQ6C)

---

## 🚀 Key Features

*   **Extreme Performance**:
    *   **Virtualized Masonry Grid**: Handles folders with thousands of images at a silky smooth 60fps.
    *   **Parallel Processing**: Uses multi-threaded CPU acceleration (Rayon) to generate high-quality WebP thumbnails in the background without freezing the UI.
    *   **Smart Caching**: Thumbnails are generated once and persisted. The incremental indexer tracks file changes instantly.

*   **Artist-Centric Design**:
    *   **Distraction-Free UI**: Minimalist dark mode interface that puts your art first.
    *   **Local First**: No cloud uploads, no subscriptions. Your images stay on your disk.
    *   **Real-Time Watcher**: Drop images into your folder, and they appear in the library immediately.

*   **Efficient Indexing**:
    *   **Duplicate Detection**: (Planned) Hash-based tracking to avoid duplicates.
    *   **Metadata Preserved**: Automatically reads modification dates and file specs.

---

## 🛠 Tech Stack

*   **Frontend**: SolidJS, TypeScript, Vite
*   **Backend**: Rust (Tauri v2)
*   **Database**: SQLite (via `sqlx` & `tauri-plugin-sql`)
*   **Styling**: Vanilla CSS (Scoped, Variable-based)
*   **Architecture**:
    *   Custom `thumb://` and `orig://` protocols for secure asset loading.
    *   Upsert-based indexing for robust crash recovery.
    *   Rayon-powered worker threads for heavy image lifting.

---

## 📦 Installation & Development

### Prerequisites
*   **Node.js** (v18+)
*   **Rust** (v1.70+)
*   **macOS / Linux / Windows** (Build tools required)

### Getting Started

1.  **Clone the repository**
    ```bash
    git clone https://github.com/marcusagm/Mundam.git
    cd mundam
    ```

2.  **Install Frontend Dependencies**
    ```bash
    npm install
    ```

3.  **Run in Development Mode**
    This command starts the Vite server and the Tauri Rust backend simultaneously with hot-reload enabled.
    ```bash
    npm run tauri dev
    ```

4.  **Build for Production**
    ```bash
    npm run tauri build
    ```
    The binary will be available in `src-tauri/target/release/bundle/`.

---

## 🧩 Architecture Highlights

This project intentionally diverges from typical Electron/Web apps to prioritize performance:

1.  **Optimistic Updates**: The indexing worker communicates directly with the UI store via granular events, eliminating database polling and ensuring instant visual feedback.
2.  **Blocking Placeholders**: The UI intelligently hides original high-res images until the lightweight thumbnail is ready, saving significant RAM and CPU during rapid scrolling.
3.  **Self-Healing DB**: The internal SQLite database handles schema creation and migrations automatically on startup (`create_if_missing`).

---

## 🗺 Roadmap

### 1. Library & Location Management
*   [x] **Location Management**: Select and monitor local folders.
*   [x] **Real-time Watcher**: Auto-sync new files, renames, and deletions.
*   [ ] **Drag-and-Drop**: Import folders via drag-and-drop.
*   [ ] **Integrity Checks**: Detect and handle broken paths or moved libraries.

### 2. Tag System (Taxonomy)
*   [x] **Hierarchical Tags**: Parent/Child tag structures (Tag Tree).
*   [ ] **Tag Management**: Rename, merge, and move tags; custom colors.
*   [ ] **Assignment**: Bulk tagging, auto-complete suggestions.
*   [ ] **Tag Search**: Quick filtering of the tag list itself.

### 3. Media Visualization
*   [x] **Masonry Layout**: Optimized virtualized grid for variable aspect ratios.
*   [x] **Progressive Loading**: Async thumbnail generation and "lazy" original loading.
*   [x] **Slide/Inspection Mode**: Fullscreen viewer with zoom/pan and navigation.
*   [ ] **File Actions**: "Open in Explorer", "Copy to Clipboard".

### 4. Search & Filtering
*   [x] **Basic Search**: By filename.
*   [x] **Advanced Criteria**: Filter by resolution, file type, dates, or tag logic (AND/OR).
*   [x] **Smart Collections**: Saved searches that auto-update (Smart Folders).

### 5. Metadata & Extras
*   [ ] **EXIF/IPTC**: Auto-read camera data and creation dates.
*   [ ] **Custom Properties**: User-defined fields (Notes, URL source).
*   [ ] **Web Clipper**: Browser extension integration for direct imports.

### 6. Infrastructure & Internals
*   [x] **Parallel Indexing**: Rayon-powered background worker for thumbnails.
*   [x] **Resilient Database**: SQLite Upsert logic for crash recovery.
*   [ ] **Backup System**: Automated database snapshots.
*   [x] **Format Support**: Extensive support for 3D, Fonts, RAW, and Vectors.

---

## 🎨 Supported Formats

Mundam provides extensive support for various media types, categorized by their rendering and thumbnail generation capabilities.

Total registered formats: 138 extensions
*   **Native/Full Support**: 100 (Thumbnail processing + Interactive visualization)
*   **Basic Support**: 40 (Visualization available, thumbnails via format icons or stubs)
*   **Testing Base**: 209 formats monitored for future expansion.

### 🖼️ Images
| Category | Formats | Status | Notes |
| :--- | :--- | :---: | :--- |
| **Standards** | `jpg`, `jpeg`, `jpe`, `jfif`, `webp`, `png`, `tiff`, `gif`, `bmp`, `ico`, `tga` | ✅ | Full support (Thumb + View). |
| **Design** | `psd`, `psb`, `ai`, `afdesign`, `afphoto`, `afpub`, `xmind`, `aseprite`, `kra`, `xcf`, `clip`, `fig`, `sketch`, `mdp`, `sai`, `reb` | ✅ | Full support (Thumb + View). |
| **RAW** | `dng`, `cr2`, `nef`, `nrw`, `rw2`, `raf`, `orf`, `pef`, `erf`, `sr2`, `srf`, `cr3`, `crw`, `arw`, `3fr`, `srw`, `kdc` | ✅ | Full support (Thumb + View). |
| **RAW (Stubs)** | `braw`, `r3d`, `ari`, `gpr`, `iiq`, `x3f`, `dcr`, `fff`, `iiq`, `mos`, `sai2` | 🚧 | Planned support. |
| **Specialized** | `pam`, `pbm`, `pgm`, `pnm`, `ppm`, `cur`, `heic`, `heif`, `avif`, `exr`, `dds` | ✅ | Full support (Thumb + View). |
| **Vectors** | `svg` | ✅ | Full support (Thumb + View). |
| **Publishing** | `indd`, `idml`, `cdr`, `pdf` | 🚧 | Planned support. |
| **Adobe/OS** | `eps` | 👁️ | View usually works via WebView/OS. |

### 🧊 3D Models
| Category | Formats | Status | Notes |
| :--- | :--- | :---: | :--- |
| **Standard 3D** | `glb`, `gltf`, `obj`, `fbx`, `stl`, `dae`, `3ds`, `dxf`, `lws`, `lwo` | 👁️ | View only (Thumbnails pending). |
| **Project** | `blend` | 👁️ | View as image (Internal preview). |
| **USD & CAD** | `usdz`, `usd`, `usda`, `usdc`, `step`, `stp`, `iges`, `igs` | 🚧 | Planned support. |

### 🔡 Fonts
| Formats | Status | Notes |
| :--- | :---: | :--- |
| `ttf`, `otf`, `ttc`, `woff`, `woff2` | ✅ | Full support (Thumb + View). |
| `eof` | 🚧 | Planned support. |

### 🎬 Video & Audio
| Category | Formats | Status | Method |
| :--- | :--- | :---: | :--- |
| **Native Video** | `mp4`, `m4v`, `mov`, `qt` | ✅ | Native browser playback. |
| **Transcoded Video** | `webm`, `wmv`, `asf`, `mkv`, `flv`, `f4v`, `avi`, `divx`, `mxf`, `ts`, `mts`, `vob`, `m2ts`, `3gp`, `3g2`, `wtv`, `rm`, `rmvb`, `ogv` | ✅ | HLS Streaming. |
| **Linear Video** | `swf`, `m2v`, `mpg`, `mpeg`, `mjpeg`, `mjpg`, `hevc`, `h264`, `h265`, `y4m` | ✅ | Linear HLS for legacy/raw formats. |
| **Video Projects** | `aep`, `prproj`, `fcpxml`, `drp` | 🚧 | Planned support (Icons). |
| **Native Audio** | `mp3`, `wav`, `aac`, `m4a`, `m4r`, `flac`, `mp2` | ✅* | Native browser playback. |
| **Transcoded Audio** | `opus`, `oga`, `ogg`, `wma`, `ac3`, `dts`, `wv`, `aifc`, `amr`, `ape`, `spx`, `ra`, `mka`, `aiff`, `aif`, `caf`, `aax`, `mid`, `midi`, `bwf` | ✅* | HLS / Linear HLS Streaming. |

 ---
 
 **Legend:**
 *  ✅ **Full Support**: Thumbnail generation and interactive visualization.
 *  ✅* **Audio Support**: Interactive visualization with format-specific icons as thumbnails.
 *  🖼️ **Thumb Only**: Thumbnail available, but no deep inspection/view.
 *  👁️ **View Only**: Interactive visualization available, but no thumbnail.
 *  🚧 **OS Dependent**: Behavior varies depending on system-level codecs/WebView.
 *  ❌ **No Support**: Currently not supported for preview or view.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
