# Recursos

- [x] Filtro de por itens com tags não está funcionando
- [x] Restaurar o menu de contexto da pasta raiz.
- [ ] Melhorar a atualização de dados de arquivos para evitar flicker e interface.
- [ ] Corrigir flicker e interface ao atualizar a thumbnail dos arquivos.
- [ ] Encontrar forma de melhorar a experiência ao exibir thumbnails, mostrando ciones para arquivos quebrados, ou que não possuem geradores de thumbnail.


# Códigos

## Core

- [ ] Melhorar a forma de resgitro de formatos em /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/registry.rs e mod.rs
- [ ] Testar todos os comandas e encontar uma forma melhor de organização /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/ledger/command.rs
- [ ] Melhorar a organização dos arquivos e pastas em /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/ledger/models/*
- [ ] Verificar o funcionamento do core/repository e sua interação com o ledgerer
- [ ] Verificar o funcionamento do core/settings e sua interação com o ledgerer
- [ ] Analisar a pasta core/workflows e verificar se existe algo que podemos retirar ou organizar melhor.

## Infra

- [ ] Analisar /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/infra/database/ para dividir o arquivo e organizar melhor. Muitos arquivos com muitas linhas de código.

## Processing

- [ ] Organização dos formatos e extratores

## Outros arquivos para verificar

- [ ] /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/lib.rs
- [ ] /Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/lifecycle.rs

# Pendências de diferenças entre v1 e v2

- [ ] Verificar operações atômicas do banco de dados durante a movimentação de pastas em larga escala.
- [ ] Verificar implementação da lógica de "adoção" de pastas antigas do V1 no scanner V2.

## Suporte a formatos de arquivos

| Extensão   | V1   | V2   | V1 Notes                                          | V2 Notes                                                    |
| :--------- | :--- | :--- | :------------------------------------------------ | :---------------------------------------------------------- |
| `jpg`      | 🟢    | 🟢    | Estável.                                          | Estável. Suporte nativo via `ImageFormatProvider`.          |
| `jpeg`     | 🟢    | 🟢    | Estável.                                          | Estável. Suporte nativo via `ImageFormatProvider`.          |
| `jpe`      | 🟢    | 🟢    | Estável.                                          | Estável. Suporte nativo via `ImageFormatProvider`.          |
| `jfif`     | 🟢    | 🟢    | Estável.                                          | Estável. Suporte nativo via `ImageFormatProvider`.          |
| `webp`     | 🟢    | 🟢    | Suporte completo.                                 | Nativo via `ModernImageFormatProvider`.                     |
| `png`      | 🟢    | 🟢    | Estável.                                          | Estável. Suporte nativo via `ImageFormatProvider`.          |
| `tiff`     | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `tif`      | 🟢    | 🟢    | Alias estável.                                    | Suporte nativo via `ImageFormatProvider`.                   |
| `gif`      | 🟢    | 🟢    | Estável, incluindo frames de animação.            | Suporte a frames de animação via FFmpeg/ImageUtils.         |
| `bmp`      | 🟢    | 🟢    | Estável.                                          | Estável. Suporte nativo via `ImageFormatProvider`.          |
| `ico`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `IconFormatProvider`.                  |
| `cur`      | 🟢    | 🔴    | Estável.                                          | **Não registrado na V2.**                                   |
| `tga`      | 🟢    | 🟢    | Suporte legado estável.                           | Nativo via `ImageFormatProvider`.                           |
| `svg`      | 🟢    | 🟢    | Nativo.                                           | Renderização via `resvg` e `tiny-skia`.                     |
| `pdf`      | 🟢    | 🟢    | Nativo (Preview no navegador).                    | Preview via `BrowserNative` e Icon Stub.                    |
| `eps`      | 🔴    | 🟢    | Extração aborta (falta ponte Ghostscript estável) | Suporte nativo via `AiFormatProvider`.                      |
| `ps`       | 🔴    | 🟢    | Extração aborta (falta ponte Ghostscript estável) | Suporte nativo via `AiFormatProvider`.                      |
| `psd`      | 🟢    | 🟢    | Estável via `psd` crate.                          | Estável. Metadados e thumbnails nativos.                    |
| `psb`      | 🟢    | 🟢    | Estável via `psd` crate.                          | Estável. Metadados e thumbnails nativos.                    |
| `ai`       | 🟢    | 🟢    | Estável para arquivos baseados em PDF.            | Suporte completo (PDF e PostScript) via `AiFormatProvider`. |
| `afdesign` | 🟢    | 🟠    | Apenas thumbnail via assinatura PNG.              | Thumbnail via assinatura PNG (Sem metadados).               |
| `afphoto`  | 🟢    | 🟠    | Apenas thumbnail via assinatura PNG.              | Thumbnail via assinatura PNG (Sem metadados).               |
| `afpub`    | 🟢    | 🟠    | Apenas thumbnail via assinatura PNG.              | Thumbnail via assinatura PNG (Sem metadados).               |
| `xmind`    | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `XMindFormatProvider`.                   |
| `aseprite` | 🟢    | 🟢    | Estável.                                          | Metadados técnicos e semânticos completos.                  |
| `ase`      | 🟢    | 🟢    | Alias estável.                                    | Metadados técnicos e semânticos completos.                  |
| `kra`      | 🟢    | 🟢    | Estável via parsing de ZIP.                       | Nativo via `ProjectZipFormatProvider`.                      |
| `krz`      | 🟢    | 🟢    | Alias (Krita Compressed).                         | Nativo via `ProjectZipFormatProvider`.                      |
| `kra~`     | 🟢    | 🟢    | Alias (Krita Backup).                             | Nativo via `ProjectZipFormatProvider`.                      |
| `xcf`      | 🟢    | 🟢    | Problemas com modos de camada e máscaras          | Extração de metadados e preview de alta resolução.          |
| `clip`     | 🟢    | 🟢    | Estável.                                          | Estável. Parsing direto do banco SQLite.                    |
| `fig`      | 🟢    | 🟢    | Estável via parsing de ZIP.                       | Suporte nativo via `ProjectZipFormatProvider`.              |
| `sketch`   | 🟢    | 🟢    | Estável via parsing de ZIP.                       | Nativo via `ProjectZipFormatProvider`.                      |
| `mdp`      | 🟢    | 🟠    | Baixa qualidade de miniatura e preview            | Thumbnail extraído, mas extração de dimensões pendente.     |
| `sai`      | 🟢    | 🟢    | Baixa qualidade de miniatura e preview            | Parsing binário nativo para metadados e preview.            |
| `sai2`     | 🔴    | 🟢    | Sem thumbnail e sem preview (mudança na infra)    | Decodificação DPCM proprietária (Sprint 10.4).              |
| `reb`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `ProjectZipFormatProvider`.            |
| `cdr`      | 🟢    | 🟢    | Baixa qualidade de miniatura e preview            | Preview de alta resolução via parsing binário.              |
| `rif`      | 🟢    | 🟢    | Suporte Corel Painter.                            | Suporte via `BinaryDesignFormatProvider`.                   |
| `riff`     | 🟢    | 🟢    | Suporte Corel Painter.                            | Suporte via `BinaryDesignFormatProvider`.                   |
| `cr2`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `cr3`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `crw`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `nef`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `nrw`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `arw`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `srf`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `sr2`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `dng`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `raf`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `orf`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `rw2`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `pef`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `erf`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `3fr`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `fff`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `dcr`      | 🔴    | 🔴    | Formatos antigos sem jpeg embutido                | Removido da registry V2 por obsolescência.                  |
| `kdc`      | 🔴    | 🔴    | Formatos antigos sem jpeg embutido                | Removido da registry V2 por obsolescência.                  |
| `srw`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `x3f`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `iiq`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `mos`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `rwl`      | 🟢    | 🟢    | Estável via LibRaw.                               | Suporte multicamadas (LibRaw + BruteForce JPEG).            |
| `mrw`      | 🟢    | 🔴    | Estável via LibRaw.                               | **Não registrado na V2.**                                   |
| `gpr`      | 🟢    | 🔴    | Estável via LibRaw.                               | **Não registrado na V2.**                                   |
| `raw`      | 🟢    | 🔴    | Suporte LibRaw (Genérico).                        | **Não registrado na V2.**                                   |
| `mef`      | 🟢    | 🔴    | Suporte LibRaw (Mamiya).                          | **Não registrado na V2.**                                   |
| `avif`     | 🟢    | 🟢    | Suporte moderno.                                  | Nativo via `ModernImageFormatProvider`.                     |
| `heic`     | 🟠    | 🟠    | Problemas intermitentes de visualização (FFmpeg)  | Instabilidade no decodificador M3U8 local (Sprint 10.12).   |
| `heif`     | 🟠    | 🟠    | Problemas intermitentes de visualização (FFmpeg)  | Instabilidade no decodificador M3U8 local (Sprint 10.12).   |
| `cur`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `dds`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `exr`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `ExrFormatProvider`.                   |
| `hdr`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `pam`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `pbm`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `pgm`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `pnm`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `ppm`      | 🟢    | 🟢    | Estável.                                          | Suporte nativo via `ImageFormatProvider`.                   |
| `indd`     | 🟠    | 🟠    | Apenas ícone genérico/stub                        | Stub de ícone via `IconFormatProvider`.                     |
| `idml`     | 🟠    | 🟠    | Apenas ícone genérico/stub                        | Stub de ícone via `IconFormatProvider`.                     |
| `jxl`      | 🟠    | 🟢    | Apenas ícone genérico/stub                        | Suporte via `ModernImageFormatProvider` (FFmpeg).           |
| `icns`     | 🟠    | 🟢    | Apenas ícone genérico/stub                        | Suporte completo via `IconFormatProvider`.                  |

### 3D & CAD
| Extensão   | V1    | V2    | V1 Notes                                          | V2 Notes (Architecture Parity)                              |
|------------|-------|-------|---------------------------------------------------|-------------------------------------------------------------|
| `glb`      | 🟢    | 🟢    | Estável via Three.js/Assimp.                      | Nativo via `Model3DFormatProvider`.                         |
| `gltf`     | 🟢    | 🟢    | Estável via Three.js/Assimp.                      | Nativo via `Model3DFormatProvider`.                         |
| `obj`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `fbx`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `stl`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `dae`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `3ds`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `3mf`      | 🔴    | 🟢    | Não suportado.                                    | Novo suporte via Assimp (V2).                               |
| `dxf`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `dwg`      | 🟢    | 🔴    | Suporte parcial (Autodesk SDK).                   | **Ausente no registro V2.**                                 |
| `lws`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `lwo`      | 🟢    | 🟢    | Suporte via Assimp.                               | Conversão Assimp -> GLB (V2).                               |
| `usdz`     | 🟢    | 🟢    | Suporte nativo (macOS).                           | Nativo via `UsdFormatProvider`.                            |
| `usd`      | 🟢    | 🟢    | Suporte nativo (macOS).                           | Nativo via `UsdFormatProvider`.                            |
| `usda`     | 🟢    | 🟢    | Suporte nativo (macOS).                           | Nativo via `UsdFormatProvider`.                            |
| `usdc`     | 🟢    | 🟢    | Suporte nativo (macOS).                           | Nativo via `UsdFormatProvider`.                            |
| `step`     | 🟢    | 🟢    | Metadados apenas.                                 | Suporte a metadados via `CadFormatProvider`.                |
| `stp`      | 🟢    | 🟢    | Metadados apenas.                                 | Suporte a metadados via `CadFormatProvider`.                |
| `iges`     | 🟢    | 🟢    | Metadados apenas.                                 | Suporte a metadados via `CadFormatProvider`.                |
| `igs`      | 🟢    | 🟢    | Metadados apenas.                                 | Suporte a metadados via `CadFormatProvider`.                |
| `zpr`      | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `ztl`      | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `sculpt`   | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `blend`    | 🟢    | 🟢    | Extração de preview REND block.                   | Nativo via `Model3dFormatProvider` (REND block).            |
| `ttf`      | 🟢    | 🟢    | Estável.                                          | Gerador de thumbnail SVG nativo funcional.                  |
| `otf`      | 🟢    | 🟢    | Estável.                                          | Gerador de thumbnail SVG nativo funcional.                  |
| `ttc`      | 🟢    | 🔴    | Estável.                                          | **Faltando no registro V2.**                                |
| `woff`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `FontFormatProvider`.                  |
| `woff2`    | 🟢    | 🟢    | Estável.                                          | Suporte completo via `FontFormatProvider`.                  |
| `eof`      | 🔴    | 🔴    | Não suportado pela extração de glifos             | Não suportado.                                              |

### Archives (Novo na V2)
| Extensão   | V1    | V2    | V1 Notes                                          | V2 Notes (Architecture Parity)                              |
|------------|-------|-------|---------------------------------------------------|-------------------------------------------------------------|
| `zip`      | 🔴    | 🟢    | Apenas listagem de arquivos.                      | Suporte a extração de thumbnails internas.                  |
| `cbz`      | 🔴    | 🟢    | Apenas listagem de arquivos.                      | Suporte a extração de thumbnails internas.                  |
| `rar`      | 🔴    | 🟠    | Não suportado.                                    | Apenas metadados, sem thumbnail (V2).                       |
| `7z`       | 🔴    | 🟠    | Não suportado.                                    | Apenas metadados, sem thumbnail (V2).                       |
| `tar`      | 🔴    | 🟠    | Não suportado.                                    | Apenas metadados, sem thumbnail (V2).                       |
| `gz`       | 🔴    | 🟠    | Não suportado.                                    | Apenas metadados, sem thumbnail (V2).                       |
| `mp4`      | 🟢    | 🟢    | Estável (FFmpeg).                                 | Estável. Playback via Linear HLS.                           |
| `m4v`      | 🟢    | 🟢    | Estável (FFmpeg).                                 | Estável. Playback via Linear HLS.                           |
| `mov`      | 🟢    | 🟢    | Estável (FFmpeg).                                 | Estável. Playback via Linear HLS.                           |
| `qt`       | 🟢    | 🟢    | Estável (FFmpeg).                                 | Estável. Playback via Linear HLS.                           |
| `webm`     | 🟢    | 🟢    | Estável (FFmpeg).                                 | Estável. Playback nativo/HLS.                               |
| `wmv`      | 🟢    | 🟢    | Estável via FFmpeg.                               | Suporte completo via `VideoFormatProvider` + HLS.           |
| `asf`      | 🟢    | 🟢    | Estável via FFmpeg.                               | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mkv`      | 🟢    | 🟢    | Estável (FFmpeg).                                 | Estável. Playback via Linear HLS.                           |
| `flv`      | 🟢    | 🟢    | Estável via FFmpeg.                               | Suporte completo via `VideoFormatProvider` + HLS.           |
| `f4v`      | 🟢    | 🟢    | Estável via FFmpeg.                               | Suporte completo via `VideoFormatProvider` + HLS.           |
| `avi`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `divx`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mxf`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `ts`       | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mts`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `vob`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `m2ts`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `3gp`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `3g2`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `wtv`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `rm`       | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `rmvb`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `ogv`      | 🟠    | 🟢    | Transcode falha ou perde referência               | Suporte completo via `VideoFormatProvider` + HLS.           |
| `swf`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mpg`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mpeg`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `m2v`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mjpeg`    | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `mjpg`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `hevc`     | 🟠    | 🟠    | Instabilidade no decodificador M3U8 local         | Instabilidade no decodificador M3U8 local (Sprint 10.12).   |
| `h264`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `h265`     | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `y4m`      | 🟢    | 🟢    | Estável.                                          | Suporte completo via `VideoFormatProvider` + HLS.           |
| `aep`      | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `prproj`   | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `fcpxml`   | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `drp`      | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `braw`     | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `r3d`      | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `ari`      | 🟠    | 🟠    | Stub mostra ícone genérico                        | Stub de ícone via `IconFormatProvider`.                     |
| `mp3`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `wav`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `aac`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `m4a`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `m4r`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `flac`     | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `mp2`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `ogg`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `oga`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `opus`     | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `wma`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `ac3`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `dts`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `wv`       | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo ou HLS.                            |
| `aiff`     | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `aif`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `aifc`     | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `spx`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `ra`       | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `mka`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `amr`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `ape`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `caf`      | 🟢    | 🟢    | Estável.                                          | Estável. Playback nativo or HLS.                            |
| `aax`      | 🔴    | 🟢    | Codificação protegida ou erro de transcode        | Transcoding via FFmpeg funcional na V2.                     |
| `mid`      | 🔴    | 🟢    | Erro de transcode (falta Soundfont)               | Melhorado via transcoding HLS (Sprint 10.12).               |
| `midi`     | 🔴    | 🟢    | Erro de transcode (falta Soundfont)               | Melhorado via transcoding HLS (Sprint 10.12).               |
| `bwf`      | 🔴    | 🟢    | Erro de transcode                                 | Estável via `AudioFormatProvider`.                          |
| `ari`      | 🟢    | 🟠    | Ícone nativo.                                     | Suporte via ícone genérico.                                 |
| `heifs`    | 🟢    | 🔴    | Suporte nativo (Sequência).                       | **Não registrado na V2.**                                   |
| `avifs`    | 🟢    | 🔴    | Suporte nativo (Sequência).                       | **Não registrado na V2.**                                   |

### Documentos
| Extensão   | V1    | V2    | V1 Notes                                          | V2 Notes (Architecture Parity)                              |
|------------|-------|-------|---------------------------------------------------|-------------------------------------------------------------|
| `txt`      | 🟢    | 🟠    | Preview de texto estável.                         | Stub de ícone (V2).                                         |
| `md`       | 🟢    | 🟠    | Renderização Markdown estável.                    | Stub de ícone (V2).                                         |
| `doc`      | 🟠    | 🟠    | Ícone apenas.                                     | Stub de ícone (V2).                                         |
| `docx`     | 🟠    | 🟠    | Ícone apenas.                                     | Stub de ícone (V2).                                         |
| `xls`      | 🟠    | 🟠    | Ícone apenas.                                     | Stub de ícone (V2).                                         |
| `xlsx`     | 🟠    | 🟠    | Ícone apenas.                                     | Stub de ícone (V2).                                         |
