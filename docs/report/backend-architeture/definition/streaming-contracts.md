# 📡 Streaming & Media URL Contracts

This document defines the definitive contracts for media URLs, streaming protocols, authentication, and caching for the Mundam application. It establishes the single source of truth for communication between the frontend media players and the backend serving infrastructure.

---

## 1. Protocol Definitions

To prevent fragmentation between `audio://`, `video://`, and other legacy protocols, the application standardizes on two main delivery mechanisms based on the playback strategy provided by the `FormatRegistry`.

### 1.1 Custom Tauri Protocol (`asset://`)

Used strictly for **native file access** when the host OS and the webview inherently support the media format without transcoding (e.g., `.mp4`, `.mov`, `.mp3`, images).

* **Format**: `asset://localhost/{assetId}`
* **Capabilities**: Supports HTTP Range requests for seeking natively within the webview.
* **When to use**: 
  * `playbackStrategy === 'native'`
* **Authentication**: Implicitly authenticated since the protocol is intercepted locally by the Tauri backend process.

### 1.2 HTTP HLS Protocol (`http://127.0.0.1:9876`)

Used for all **transcoded media**, enabling adaptive bitrate and on-the-fly streaming for unsupported formats (e.g., `.mkv`, `.avi`, `.flac`, `.swf`).

The HTTP server runs locally (usually on port `9876`) and serves HLS (`.m3u8` playlists and `.ts` segments).

#### Standard HLS (VOD)
* **Format**: `http://127.0.0.1:9876/playlist/{assetId}/playlist.m3u8?quality={quality}`
* **When to use**: 
  * `playbackStrategy === 'hls'`
  * `playbackStrategy === 'audioHls'`
  * `playbackStrategy === 'transcode'`
  * `playbackStrategy === 'audioTranscode'`

#### Linear HLS (Live/Async)
* **Format**: `http://127.0.0.1:9876/hls-live/{assetId}/index.m3u8?quality={quality}&mode={mode}&token={token}`
* **When to use**: 
  * `playbackStrategy === 'linearHls'`
  * `playbackStrategy === 'audioLinearHls'`
  * Formats requiring immediate stream-in-progress delivery (e.g., complex demuxing where chunking requires sequential generation).

---

## 2. Authentication & Security

While the `asset://` protocol is intrinsically secure within the app IPC, the HTTP HLS server is exposed to `localhost`. To prevent unauthorized local network access (or local system access), an authentication token is used.

1. **Session Token**: On boot, the backend generates a random `streaming_token`.
2. **Retrieval**: The frontend requests this token via the Tauri command `get_streaming_token`.
3. **Usage**: The token is appended to sensitive streaming URLs (especially Live/Linear endpoints) as `&token={token}`.
4. **Validation**: The Axum HTTP server middleware verifies the token before serving any `.m3u8` or `.ts` file.

---

## 3. Caching Strategy

The HLS pipeline generates numerous transport stream (`.ts`) segments.

* **Location**: Segments are stored in the OS-specific application cache directory (`app_cache_dir/streaming/`).
* **TTL (Time To Live)**: Streaming cache is volatile. Segments have a defined maximum age (defaulting to 24-72 hours depending on settings).
* **Cleanup Strategy**: 
  * A background worker (HLS Cleanup Worker) periodically sweeps the directory.
  * The frontend can manually invoke `cleanup_cache(maxAgeDays)` or `clear_cache()` via IPC.
* **Integrity**: Missing segments trigger a partial re-transcode request from the player automatically, as HLS is resilient to missing chunks.

---

## 4. Frontend Integration (`stream-utils.ts`)

The frontend abstracts URL resolution using `stream-utils.ts`. The UI should **never** manually construct these URLs.

```typescript
import { getVideoUrl, getAudioUrl } from '@/lib/stream-utils';

// Resolves to asset://localhost/12345 or http://127.0.0.1:9876/playlist/12345/playlist.m3u8
const url = getVideoUrl('12345', '/path/to/file.mkv', 'standard', probeResult);
```

### Supported Qualities
- `preview`: Lowest resolution/bitrate for fast skimming.
- `standard`: Balanced bitrate for normal playback.
- `high`: Maximum visual/audio fidelity.
