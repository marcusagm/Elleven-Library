# Sprint 10.10: Paridade de Audio e Video Extractors

**Status da sprint:** ✅ Concluída
**Data e hora de inicio da sprint:** 2026-03-24T23:20:00-03:00
**Data e hora da conclusão da sprint:** 2026-03-25T08:14:00-03:00

## Objetivo

Verificar e garantir paridade completa entre os extractors de áudio e vídeo da V1 e V2, com foco em extensões legadas, metadados técnicos e geração de waveform.

## Estado Atual

### Audio — V2 (`audio_format.rs`, 13,299 bytes)
- ✅ Extensões modernas: `mp3`, `flac`, `wav`, `aac`, `ogg`, `opus`, `m4a`
- ✅ Waveform via `get_audio_waveform_data` (IPC command)
- ✅ Extensões legadas da V1 verificadas — todas já presentes + `m4b`, `mpc` adicionadas

### Video — V2 (`video_format.rs`, 14,176 bytes)
- ✅ Extensões modernas: `mp4`, `mkv`, `webm`, `mov`, `avi`
- ✅ HLS streaming via `HlsManager`
- ✅ Extensões legadas e codecs específicos verificados — todas já presentes

### Playback — Pipeline de Mídia
- ✅ Campo `path` adicionado ao `AssetSummaryDto` — corrige playback de todos os formatos
- ✅ Waveform extraction com `spawn_blocking` — corrige timeout de extração
- ✅ Todos os formatos de áudio e vídeo reproduzem corretamente (exceto MIDI e AIFF HLS)

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

**Status:** ✅ Concluída — Todas as extensões legadas já presentes na V2. Adicionadas `m4b` e `mpc` como extras.

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

**Status:** ✅ Concluída — Todas as extensões legadas já presentes na V2. `xvid` não existia na V1 (`definitions.rs`) e não foi adicionada (formato obsoleto sem suporte FFmpeg nativo como extensão).

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

**Status:** ✅ Concluída — 22+ providers registrados em `build_format_registry()`. AudioFormatProvider e VideoFormatProvider incluídos.

O `FormatRegistry` resolve extensões para providers. Verificar o arquivo master de registro:

```bash
# Localizar onde os providers são registrados
grep -n "register\|FormatRegistry::new" src-tauri/src/core/formats/mod.rs
grep -rn "push\|register" src-tauri/src/core/formats/
```

**Verificar se todos os 23 `FormatProviders` estão sendo registrados em `lib.rs` ou similar.**

### 4. Validar Metadados Técnicos de Áudio

**Status:** ✅ Concluída — Campo `bitrate_kbps` adicionado ao `extract_technical()` via FFprobe `format.bit_rate`.

Para áudio, os metadados técnicos devem incluir:
- `duration` (em segundos)
- `bitrate` (em kbps)
- `sample_rate` (em Hz)
- `channels` (mono/stereo/5.1)
- `codec` (mp3, flac, aac, etc.)

**Verificar se `AudioFormatProvider.MetadataCapability.extract_technical()` retorna todos esses campos usando FFprobe ou lofty-rs.**

### 5. Validar Metadados Técnicos de Vídeo

**Status:** ✅ Concluída — Campos `frame_rate_fps` e `bitrate_kbps` adicionados. Helper `parse_frame_rate_fraction()` criado para parsear frações FFprobe (ex: "30000/1001").

Para vídeo, os metadados técnicos devem incluir:
- `duration` (em segundos)
- `width`, `height` (resolução)
- `frame_rate` (em fps)
- `video_codec` (h264, hevc, av1, etc.)
- `audio_codec`
- `bitrate`

**Verificar se esses campos estão sendo extraídos e retornados pelo `VideoFormatProvider`.**

### 6. Waveform — Verificar Extensões Suportadas

**Status:** ✅ Concluída — Waveform usa FFmpeg que suporta automaticamente todos os formatos de áudio registrados, incluindo `.dts`, `.ac3`, `.aif`. Nenhuma mudança necessária.

O comando IPC `get_audio_waveform_data` chama `feature::media::waveform::extract_audio_waveform()`.

Verificar se `.dts`, `.ac3`, `.aif` são suportados pela extração de waveform (provavelmente via FFmpeg, então devem funcionar automaticamente).

### 7. Corrigir Pipeline de Playback de Mídia

**Status:** ✅ Concluída — Campo `path` adicionado ao `AssetSummaryDto`, corrigindo playback de todos os formatos de áudio e vídeo.

O `AssetSummaryDto` (DTO lightweight retornado pelo `PaginatedAssetsDto`) não incluía o campo `path: PathBuf`. O frontend `AssetItem` espera `path: string` para construir URLs de streaming. Resultado: todos os vídeos mostravam tela preta e todos os áudios ficavam com loader infinito.

**Correção:** Adicionado `path` ao `AssetSummaryDto`, `AssetSummaryDb`, queries SQL (`list_paginated` e `search_assets`), e mapeamento `From<AssetSummaryDb>`.

### 8. Corrigir Timeout de Extração de Waveform

**Status:** ✅ Concluída — FFmpeg agora executa em `spawn_blocking`, evitando bloqueio do runtime async.

A função `extract_audio_waveform()` executava o subprocesso FFmpeg diretamente na thread do runtime async do tokio via `run_command_with_timeout()` (bloqueante). Na V1, o IPC command era síncrono e rodava numa thread dedicada. Na V2, o command é async mas o FFmpeg bloqueava a thread, causando timeout no frontend (15s) antes do backend completar (30s).

**Correção:** Envolveu a execução FFmpeg em `tokio::task::spawn_blocking` e aumentou o timeout do frontend de 15s para 35s.

### 9. Limpeza de Logs de Diagnóstico

**Status:** ✅ Concluída — Todos os `console.log` de debug removidos dos hooks e player.

Logs `[DEBUG:VideoSource]`, `[DEBUG:AudioSource]` e `[DEBUG:HlsPlayer]` foram adicionados temporariamente para diagnosticar o problema de playback. Após confirmação da correção, todos foram removidos.

## Arquivos a Modificar

- `src-tauri/src/processing/media/audio_format.rs` — adicionar extensões legadas
- `src-tauri/src/processing/media/video_format.rs` — adicionar extensões legadas
- `src-tauri/src/core/formats/mod.rs` ou `lib.rs` — verificar registro completo

## Critérios de Aceitação

- [x] `.dts`, `.aif`, `.ac3` carregam o inspector de **áudio** (não imagem)
- [x] `.f4v`, `.mjpeg`, `.asf` carregam o inspector de **vídeo** (não imagem)
- [x] Playback de `.dts` via HLS (se FFmpeg suportar) ou mensagem de codec incompatível
- [x] Waveform gerado para formatos de áudio legados suportados pelo FFmpeg
- [x] Inspector de áudio mostra duration, bitrate, sample rate, channels
- [x] Inspector de vídeo mostra duration, resolution, codec, frame rate
- [x] Playback de áudio e vídeo funcional para todos os formatos (exceto MIDI)
- [x] Waveform renderiza sem timeout para formatos suportados pelo FFmpeg
- [x] Logs de diagnóstico removidos do código de produção

## Referência V1

- `mundam-main/src-tauri/src/formats/definitions.rs` — lista completa de extensões V1
- `mundam-main/src-tauri/src/thumbnails/extractors/mod.rs` — mapeamento extension → extractor

## Detalhes da Implementação

### Resultado da Auditoria de Extensões

A V2 já possuía paridade quase total com a V1 em extensões de áudio e vídeo. As extensões listadas como "problemáticas" na sprint 9.1 (`dts`, `ac3`, `aif`, `f4v`, `mjpeg`, `asf`) já estavam registradas tanto em `AUDIO_EXTENSIONS`/`VIDEO_EXTENSIONS` quanto nos respectivos `supported_formats()` com `SupportedFormat` entries completas.

### Extensões Adicionadas (além da paridade V1)

| Extensão | Formato | Justificativa |
|----------|---------|---------------|
| `m4b` | MPEG-4 Audiobook | Container M4A com capítulos. Formato legítimo não presente na V1. Adicionado ao grupo "MPEG-4 Audio" com `PlaybackStrategy::Native`. |
| `mpc` | Musepack Audio | Codec lossless/lossy legado. Novo `SupportedFormat` "Musepack Audio" com `PlaybackStrategy::AudioHls`. |

### Extensão `xvid` — Não adicionada

A extensão `xvid` listada no sprint não existe na V1 (`definitions.rs`). Arquivos XviD utilizam containers `.avi` ou `.divx`, ambos já suportados. A extensão `.xvid` como tal não é padrão e não possui suporte nativo como extensão de arquivo.

### Campos de Metadados Adicionados

| Campo | Arquivo | Fonte FFprobe | Conversão |
|-------|---------|---------------|------------|
| `bitrate_kbps` | `audio_format.rs` | `format.bit_rate` (string, bps) | Dividido por 1000, arredondado |
| `bitrate_kbps` | `video_format.rs` | `format.bit_rate` (string, bps) | Dividido por 1000, arredondado |
| `frame_rate_fps` | `video_format.rs` | `stream.r_frame_rate` (fração, ex: "30000/1001") | Parseado via `parse_frame_rate_fraction()`, arredondado a 2 casas |

### Função Auxiliar Criada

`parse_frame_rate_fraction(fraction_string: &str) -> Option<f64>` — Parseia frações de frame rate do FFprobe, com fallback para parse direto de número. Trata edge cases como denominador zero.

### Correção do Campo `path` no `AssetSummaryDto`

O `AssetSummaryDto` é o DTO lightweight usado para listagens paginadas no grid. Na V2, o campo `path: PathBuf` foi omitido por design (performance), mas o frontend precisa dele para construir URLs de streaming via `getVideoUrl()` e `getAudioUrl()` em `stream-utils.ts`.

**Caminho de dados corrigido:**
```
SQL (a.path) → AssetSummaryDb.path → AssetSummaryDto.path → JSON (path) → AssetItem.path → stream-utils
```

### Correção do Waveform Timeout

Na V1, o IPC `get_audio_waveform_data` era um comando síncrono que rodava automaticamente em uma thread dedicada pelo Tauri. Na V2, o IPC é `async` mas chamava `run_command_with_timeout()` (bloqueante) diretamente na thread do tokio, impedindo o runtime de processar outras tarefas.

**Solução:** Envolver a execução FFmpeg em `tokio::task::spawn_blocking()`:
```rust
let result = tokio::task::spawn_blocking(move || {
    // FFmpeg blocking execution here
}).await??;
```

### Erros Conhecidos e Esperados

| Formato | Erro | Motivo |
|---------|------|--------|
| `.mid` (MIDI) | `Invalid data found when processing input` | FFmpeg não processa MIDI para waveform/playback — formato simbólico, não PCM |
| `.aiff` (AIFF) | `frame size not set` (HLS segments) | FFmpeg falha ao transcodar PCM big-endian para MPEGTS em segmentos HLS. Playback nativo funciona. |

### Verificação

| Teste | Resultado |
|-------|---------:|
| `cargo build` | ✅ 0 erros (3 warnings pré-existentes não relacionados) |
| `cargo test --lib core::formats` | ✅ 1/1 teste passou (`test_registry_resolution`) |
| Playback de áudio (todos os formatos) | ✅ Funcional (exceto MIDI) |
| Playback de vídeo (todos os formatos) | ✅ Funcional |
| Waveform rendering | ✅ Funcional (sem timeout) |
| Logs de diagnóstico | ✅ Todos removidos |

## Arquivos Modificados

- `src-tauri/src/processing/media/audio_format.rs` — `m4b`/`mpc` em `AUDIO_EXTENSIONS`, `m4b` no `SupportedFormat` "MPEG-4 Audio", novo `SupportedFormat` "Musepack Audio", campo `bitrate_kbps` em `extract_technical()`
- `src-tauri/src/processing/media/video_format.rs` — nova função `parse_frame_rate_fraction()`, campos `frame_rate_fps` e `bitrate_kbps` em `extract_technical()`
- `src-tauri/src/core/models/asset.rs` — adicionado campo `path: PathBuf` ao `AssetSummaryDto`
- `src-tauri/src/infra/database/models.rs` — adicionado campo `path: String` ao `AssetSummaryDb`, mapeamento `path` no `From<AssetSummaryDb> for AssetSummaryDto`
- `src-tauri/src/infra/database/queries.rs` — adicionado `a.path as path` nas queries `list_paginated` e `search_assets`
- `src-tauri/src/feature/media/waveform.rs` — FFmpeg envolvido em `tokio::task::spawn_blocking`, mensagem de erro inclui path do arquivo
- `src/core/hooks/useVideoSource.ts` — removidos logs `[DEBUG:VideoSource]`
- `src/core/hooks/useAudioSource.ts` — removidos logs `[DEBUG:AudioSource]`
- `src/lib/createHlsPlayer.ts` — removidos logs `[DEBUG:HlsPlayer]`
- `src/components/ui/AudioPlayer/useAudioPlayer.ts` — timeout de waveform aumentado de 15s para 35s
