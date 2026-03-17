# Sprint 10.10: Paridade de Audio e Video Extractors

**Status da sprint:** Verificação necessária
**Data e hora de inicio da sprint:** -
**Data e hora da conclusão da sprint:** -

## Objetivo

Verificar e garantir paridade completa entre os extractors de áudio e vídeo da V1 e V2, com foco em extensões legadas, metadados técnicos e geração de waveform.

## Estado Atual

### Audio — V2 (`audio_format.rs`, 13,299 bytes)
- ✅ Extensões modernas: `mp3`, `flac`, `wav`, `aac`, `ogg`, `opus`, `m4a`
- ✅ Waveform via `get_audio_waveform_data` (IPC command)
- ⚠️ Extensões legadas da V1 precisam de verificação

### Video — V2 (`video_format.rs`, 14,176 bytes)
- ✅ Extensões modernas: `mp4`, `mkv`, `webm`, `mov`, `avi`
- ✅ HLS streaming via `HlsManager`
- ⚠️ Extensões legadas e codecs específicos precisam de verificação

### Extensões V1 que precisam ser verificadas

**Áudio legado V1:**
```
dts, ac3, aiff, aif, flac, ape, wv, mpc, oga, spx, m4b
```

**Vídeo legado V1:**
```
f4v, mjpeg, asf, wmv, m2ts, mts, ts, 3gp, 3g2, divx, xvid, vob
```

## Tarefas

### 1. Auditar audio_format.rs — Extensões V1 Faltantes

**Status:** Pendente

**Extensões identificadas na sprint 9.1 como problemáticas:**
- `.dts` — Digital Theater Systems audio
- `.aif` — Apple AIFF (variante sem 'f')
- `.ac3` — Dolby Digital audio

**Verificar se estão em `supported_extensions()` do `AudioFormatProvider`.**

Se faltantes:
```rust
fn supported_extensions(&self) -> Vec<&'static str> {
    vec![
        // Existentes
        "mp3", "flac", "wav", "aac", "ogg", "opus", "m4a", "wma",
        // Legados V1 a adicionar
        "dts", "ac3", "aiff", "aif", "ape", "wv", "mpc", "oga", "spx", "m4b",
    ]
}
```

### 2. Auditar video_format.rs — Extensões V1 Faltantes

**Status:** Pendente

**Extensões identificadas na sprint 9.1 como problemáticas:**
- `.f4v` — Flash Video (legado)
- `.mjpeg` — Motion JPEG
- `.asf` — Advanced Systems Format (Microsoft)

**Verificar e adicionar:**
```rust
fn supported_extensions(&self) -> Vec<&'static str> {
    vec![
        // Existentes
        "mp4", "mkv", "webm", "mov", "avi", "wmv", "m4v",
        // Legados V1 a adicionar
        "f4v", "flv", "mjpeg", "mjpg", "asf", "m2ts", "mts",
        "3gp", "3g2", "divx", "xvid", "vob", "ogv",
    ]
}
```

### 3. Verificar FormatRegistry Master — todos os formatos registrados

**Status:** Pendente

O `FormatRegistry` resolve extensões para providers. Verificar o arquivo master de registro:

```bash
# Localizar onde os providers são registrados
grep -n "register\|FormatRegistry::new" src-tauri/src/core/formats/mod.rs
grep -rn "push\|register" src-tauri/src/core/formats/
```

**Verificar se todos os 23 `FormatProviders` estão sendo registrados em `lib.rs` ou similar.**

### 4. Validar Metadados Técnicos de Áudio

**Status:** Pendente

Para áudio, os metadados técnicos devem incluir:
- `duration` (em segundos)
- `bitrate` (em kbps)
- `sample_rate` (em Hz)
- `channels` (mono/stereo/5.1)
- `codec` (mp3, flac, aac, etc.)

**Verificar se `AudioFormatProvider.MetadataCapability.extract_technical()` retorna todos esses campos usando FFprobe ou lofty-rs.**

### 5. Validar Metadados Técnicos de Vídeo

**Status:** Pendente

Para vídeo, os metadados técnicos devem incluir:
- `duration` (em segundos)
- `width`, `height` (resolução)
- `frame_rate` (em fps)
- `video_codec` (h264, hevc, av1, etc.)
- `audio_codec`
- `bitrate`

**Verificar se esses campos estão sendo extraídos e retornados pelo `VideoFormatProvider`.**

### 6. Waveform — Verificar Extensões Suportadas

**Status:** Pendente

O comando IPC `get_audio_waveform_data` chama `feature::media::waveform::extract_audio_waveform()`.

Verificar se `.dts`, `.ac3`, `.aif` são suportados pela extração de waveform (provavelmente via FFmpeg, então devem funcionar automaticamente).

## Arquivos a Modificar

- `src-tauri/src/processing/media/audio_format.rs` — adicionar extensões legadas
- `src-tauri/src/processing/media/video_format.rs` — adicionar extensões legadas
- `src-tauri/src/core/formats/mod.rs` ou `lib.rs` — verificar registro completo

## Critérios de Aceitação

- [ ] `.dts`, `.aif`, `.ac3` carregam o inspector de **áudio** (não imagem)
- [ ] `.f4v`, `.mjpeg`, `.asf` carregam o inspector de **vídeo** (não imagem)
- [ ] Playback de `.dts` via HLS (se FFmpeg suportar) ou mensagem de codec incompatível
- [ ] Waveform gerado para formatos de áudio legados suportados pelo FFmpeg
- [ ] Inspector de áudio mostra duration, bitrate, sample rate, channels
- [ ] Inspector de vídeo mostra duration, resolution, codec, frame rate

## Referência V1

- `mundam-main/src-tauri/src/formats/definitions.rs` — lista completa de extensões V1
- `mundam-main/src-tauri/src/thumbnails/extractors/mod.rs` — mapeamento extension → extractor
