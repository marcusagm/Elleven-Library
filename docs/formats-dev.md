# Formatos Suportados (Visão de Desenvolvimento)

Este documento mapeia o suporte de formatos do Mundam sob a ótica técnica do backend, detalhando as estruturas de enumeração (`ThumbnailStrategy`, `PreviewStrategy` e `PlaybackStrategy`) definidas no núcleo da aplicação em `src-tauri/src/formats/definitions.rs`.

O objetivo é fornecer clareza imediata sobre como cada arquivo é redirecionado nos _pipelines_ internos de extração e transcode.

---

## 🖼️ Imagens
O motor de imagens usa bibliotecas nativas web ou conversões em _background_ para exibir as mídias em tempo viável para a lista.

### Padrões e Exibição Direta
| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| `jpg`, `jpeg`, `jpe`, `jfif`, `webp`, `png`, `gif`, `bmp`, `ico`, `cur` | `NativeImage` | `BrowserNative` | `None` |
| `tif`, `tiff` | `NativeImage` | `Convert` | `None` |
| `tga` | `NativeExtractor` | `Convert` | `None` |

### Especializados e High Dynamic Range
| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| `dds`, `exr`, `hdr` | `NativeExtractor` | `NativeExtractor` | `None` |
| `pam`, `pbm`, `pgm`, `pnm`, `ppm` | `NativeExtractor` | `Convert` | `None` |
| `avif` | `Ffmpeg` | `Ffmpeg` | `None` |

### Vetoriais e Documentos
| Extensões (Software) | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| Adobe Illustrator (`ai`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Inkscape (`svg`) | `Webview` | `BrowserNative` | `None` |
| Portable Document Format (`pdf`) | `NativeExtractor` | `BrowserNative` | `None` |

---

## ✍️ Projetos Artísticos (Digital Painting e UI)
Os extratores nativos varrem o binário dos projetos para retirar as "composite images" salvas diretamente pelos _softwares_.

### Photo and digital painting
| Software (Extensões) | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| Adobe Photoshop (`psd`, `psb`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Affinity (`afdesign`, `afpub`, `afphoto`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Clip Studio (`clip`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Corel Painter (`rif`, `riff`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Krita (`kra`, `krz`, `kra~`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Rebelle (`reb`) | `NativeExtractor` | `NativeExtractor` | `None` |
| SketchBook Pro (`tif`, `tiff`) | `NativeImage` | `Convert` | `None` |

### PixelArt
| Software (Extensões) | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| Aseprite (`aseprite`, `ase`) | `NativeExtractor` | `NativeExtractor` | `None` |

### Design, ui & ux
| Software (Extensões) | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| Figma (`fig`) | `NativeExtractor` | `NativeExtractor` | `None` |
| Sketch (`sketch`) | `NativeExtractor` | `NativeExtractor` | `None` |

### Others
| Software (Extensões) | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| xMind (`xmind`) | `NativeExtractor` | `NativeExtractor` | `None` |

---

## 📸 RAW (Fotografia)
Delega inteiramente ao uso atrelado à `LibRaw` no backend. Formatos variam imensamente por tipo de sensor da câmera.

| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| `dng`, `cr2`, `nef`, `nrw`, `rw2`, `raf`, `orf`, `pef`, `erf`, `sr2`, `srf`, `cr3`, `crw`, `arw`, `srw`, `mos`, `rwl`, `mrw`, `fff`, `iiq`, `raw`, `x3f`, `mef`, `3fr`, `kdc` | `Raw` | `Raw` | `None` |
> **Nota:** A estratégia `Raw` possui um fallback automático para `NativeExtractor` (varredura binária) caso o decodificador LibRaw falhe ou não suporte o modelo específico.
---

## 🧊 Modelos 3D
Módulos que geram visão espacial via _canvas_ no frontend.

| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| Standard 3D (`glb`, `gltf`, `obj`, `fbx`, `stl`, `dae`, `3ds`, `dxf`, `lws`, `lwo`) | `Model3D` | `None` | `None` |
| Blender (`blend`) | `NativeExtractor` | `NativeExtractor` | `None` |

---

## 🎬 Vídeos e Animações
Sustentado por _pipelines_ do `FFmpeg`.

| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| `mp4`, `m4v`, `mov`, `qt` | `Ffmpeg` | `None` | `Native` |
| `webm`, `mkv`, `mxf`, `wmv`, `asf`, `flv`, `f4v`, `ts`, `mts`, `vob`, `m2ts`, `3gp`, `3g2`, `rm`, `rmvb`, `wtv` | `Ffmpeg` | `None` | `Hls` |
| `swf`, `mpg`, `mpeg`, `m2v`, `mjpeg`, `mjpg`, `h264`, `h265`, `y4m` | `Ffmpeg` | `None` | `LinearHls` |

---

## 🎶 Áudio 
A exibição utiliza o player embutido do navegador se nativo, ou transcode `HLS`.

| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| `mp3`, `mp2`, `wav`, `flac`, `m4a`, `aac`, `m4r` | `Icon` | `None` | `Native` |
| `ogg`, `oga`, `opus`, `wma`, `ac3`, `dts`, `wv`, `aiff`, `aif`, `aifc`, `spx`, `ra`, `mka`, `amr`, `ape`, `caf` | `Icon` | `None` | `AudioHls` |

---

## 🔡 Fontes Tipográficas
Delegação a biblioteca de conversão de Glifos (FontDB/Skia/etc).

| Extensões | `ThumbnailStrategy` | `PreviewStrategy` | `PlaybackStrategy` |
| :--- | :--- | :--- | :--- |
| `ttf`, `otf`, `ttc`, `woff`, `woff2` | `Font` | `None` | `None` |

---

## ⚠️ Limitações e Problemas Identificados (Tracker Técnico)
Um histórico direto do pipeline em falha na engenharia atual. Formatos que, apesar do `definitions.rs` determinar uma estratégia, apresentam quedas ou degradações devido os executores internos falharem ou precisarem ser expandidos.

### Qualidade Reduzida ou Inconsistências
| Formato / Software (Extensões) | Sub-Estratégias Falhas (`Thumb` / `Preview` / `Playback`) | Causa / Bug / Status |
| :--- | :--- | :--- |
| Corel Draw (`cdr`) | `NativeExtractor` / `NativeExtractor` | Faltam parsers robustos para extrair os binários sem as compressões nativas que retornam imagem borrada. |
| Gimp (`xcf`) | `NativeExtractor` / `NativeExtractor` | Limitado nas camadas mapeadas (ocultas e com máscaras vazam visualmente nas _composições_). |
| Medibang / FireAlpaca (`mdp`) | `NativeExtractor` / `NativeExtractor` | Qualidade e fidelidade comprometidos no byte buffer. |
| Paint Tool SAI (`sai`) | `NativeExtractor` / `NativeExtractor` | Inconsistências na re-projeção e tamanhos minúsculos limitados da engine legada. |
| Áudio/MIDI (`aax`, `mid`, `midi`, `bwf`) | `Icon` / `None` / `AudioHls` | Erros técnicos de transcode (Falta Soundfont) ou codificação protegida barrando o container de HLS de ser lido/convertido. |
| Vídeo Ogg (`ogv`) | `Ffmpeg` / `None` / `Hls` | Transcode do OGG theora falha nas conversões ou perde referência da imagem no frame extraído. |
| Vídeo HEVC (`hevc`) | `Ffmpeg` / `None` / `LinearHls` | Formato não envelopado causa instabilidade no decodificador M3U8 local. |
| Apple Fotos (`heic`, `heif`) | `Ffmpeg` / `Ffmpeg` | O FFmpeg engasga intermitentemente em decodificar o HEVC puro embutido como imagem para stream Canvas de alta contagem de megapixels. |
| Penpot v1 / v2 (`penpot`) | `NativeExtractor` / `NativeExtractor` | A Versão 1 lê perfeitamente, mas a versão 2 não possui _Preview object_ em seus containers e necessita de rendering full (o Extractor atual falha limpo). |

### Sem Suporte Vigente (Acesso Quebrado / Bypass)
| Formato / Software (Extensões) | Sub-Estratégias Falhas | Causa / Bug / Status |
| :--- | :--- | :--- |
| Paint Tool SAI 2 (`sai2`) | `NativeExtractor` / `NativeExtractor` | Mudança severa na infra do arquivo. Não existe mais ponte de extração direta de imagens. |
| Módulos RAW Falhos (`dcr`, `mdc`) | `None` / `None` | Arquivos possuem preview criptografado/color-spaced desconhecido (DCR) ou sequer embutem um Jpeg/Tiff genérico (MDC). Resultam em Arquivo Vazio. |
| PostScript Clássico (`eps`, `ps`) | `NativeExtractor` / `NativeExtractor` | Extrações abortam (não possuímos ponte PDFium/Ghostscript configurada 100% à prova de falhas neste estágio de suporte web nativo). |
| Fonte EOT (`eot`) | `Icon` / `None` / `None` | Biblioteca de extração de glifos não lida com essa estrutura; desativado ("Stubbed"). |
