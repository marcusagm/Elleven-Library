# Technical Analysis: XMind (.xmind) Format

## 1. Format Overview

*   **Extension:** `.xmind`
*   **Software:** XMind 8, XMind (Zen).
*   **Category:** Mind Mapping / Document.
*   **Versions:**
    *   **Modern (XMind 2020+):** ZIP-based container with JSON content.
    *   **Classic (XMind 8):** ZIP-based container with XML content.
*   **Signature:** `PK\x03\x04` (ZIP).

---

## 2. Structure (Modern)

XMind files are ZIP archives using a structure inspired by OpenDocument.

| Path | Description |
| :--- | :--- |
| `manifest.json` | Manifest of files in the archive. |
| `content.json` | Main mind map structure (Modern). |
| `content.xml` | Main mind map structure (Classic). |
| `Thumbnails/` | **Critical:** Contains the preview image. |
| `resources/` | Embedded images, attachments. |
| `metadata.json` | Map metadata (title, author). |

---

## 3. Thumbnail Extraction Strategy

The thumbnail is a standard image file inside the ZIP.

*   **Primary Path:** `Thumbnails/thumbnail.png`.
*   **Fallback Paths:**
    *   `metadata/thumbnail.png`
    *   `Thumbnails/thumbnail.jpg`
*   **Strategy:** Simply open the ZIP and extract the file in the `Thumbnails/` directory.

---

## 4. Implementation Strategy

### 4.1. Fast Extraction
1.  Verify the `PK` header.
2.  List files in the ZIP.
3.  Extract `Thumbnails/thumbnail.png`.
4.  If not found, fall back to the first image found in the `resources/` directory if the map consists mostly of one image.

---

## 5. Uncertainties
*   **Legacy Formats:** Very old XMind versions (pre-2008) used a different binary format, but these are rare today.
*   **Password Protection:** If the XMind file is password protected, the ZIP entries are encrypted using standard ZIP encryption or AES, preventing direct thumbnail extraction without a key.
