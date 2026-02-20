# Arquivos Suportados pelo Mundam

O Mundam foi construído para lidar com a imensa variedade de arquivos encontrados nas bibliotecas de artistas e designers. Este documento detalha o nível de suporte atual para cada tipo de arquivo, desde a visualização em alta qualidade até as gerações de miniaturas (thumbnails).

## 🖼️ Imagens
O aplicativo possui suporte robusto à maioria dos formatos de imagem, garantindo visualização rápida e miniaturas de alta resolução.

### Padrões e Exibição Direta
Estes são os formatos nativos suportados pela Web ou processados com conversões simples e de altíssima fidelidade.
- `jpg`, `jpeg`, `jpe`, `jfif`, `webp`, `png`, `gif`, `bmp`, `ico`, `tga` - Com miniatura e visualização completa.
- `tif`, `tiff` - Com miniatura e visualização completa.

### Especializados e High Dynamic Range
Formatos avançados para texturização 3D, fotografia especializada e afins.
- `dds`, `exr`, `hdr`, `pam`, `pbm`, `pgm`, `pnm`, `ppm` - Com miniatura e visualização completa.
- `avif` - Com miniatura e visualização completa.

### Vetoriais 
- Adobe Illustrator (`ai`) - Com miniatura e visualização completa.
- Inkscape (`svg`) - Com miniatura e visualização completa.

## ✍️ Projetos Artísticos (Digital Painting e UI)
O Mundam consegue extrair as miniaturas de dentro dos projetos salvos pelos principais softwares do mercado.

### Photo and digital painting
- Adobe Photoshop (`psd`, `psb`) - Com miniatura e visualização completa.
- Affinity (`afdesign`, `afpub`, `afphoto`) - Com miniatura e visualização completa.
- Clipstudio (`clip`) - Com miniatura e visualização completa.
- Corel Painter (`rif`, `riff`) - Com miniatura e visualização completa.
- Krita (`kra`) - Com miniatura e visualização completa.
- Rebelle (`reb`) - Com miniatura e visualização completa.
- SkecthBook Pro (`tif`, `tiff`) - Com miniatura e visualização completa.

### PixelArt
- Aseprite (`aseprite`) - Com miniatura e visualização completa.

### Design, ui & ux
- Figma (`fig`) - Com miniatura e visualização completa.
- Sketch (`sketch`) - Com miniatura e visualização completa.

### Others
- xMind (`xmind`) - Com miniatura e visualização completa.


## 📸 RAW (Fotografia)
Arquivos diretos de câmeras digitais profissionais recebem suporte robusto de indexação e visualização no aplicativo.
- `3fr`, `arw`, `cr2`, `cr3`, `crw`, `dng`, `erf`, `kdc`, `mos`, `mrw`, `nef`, `nrw`, `orf`, `pef`, `raf`, `rw2`, `rwl`, `sr2`, `srf`, `srw`, `tif`, `tiff` - Com miniatura e visualização completa.


## 🧊 Modelos 3D
Os arquivos 3D podem ser inspecionados iterativamente com rotação e pan direto na visualização do Mundam. No momento, a maioria exibe apenas o modelo na visualização e aguarda atualização para geração da miniatura.
- `stl`, `obj`, `lws`, `lwo`, `glb`, `gltf`, `fbx`, `dae`, `3ds`, `dxf`, `blend` - Sem miniatura, mas possuem visualização completa.


## 🎬 Vídeos e Animações
O Mundam tira proveito de um conversor em segundo plano para extrair previews em tempo real para arquivos pesados. 

- `mp4`, `m4v`, `mov` - Com miniatura e player nativo.
- `asf`, `avi`, `f4v`, `flv`, `webm`, `wmv`, `mkv`, `mxf`, `ts`, `mts`, `vob`, `m2ts`, `ts`, `3gp`, `3g2`, `wtv`, `rm`, `swf` - Com miniatura e player usa HLS (streaming local).
- `swf`, `m2v`, `mpg`, `mpeg`, `mjpeg` - Com miniatura e player usa HLS Linear.

## 🎶 Áudio 
As mídias em áudio possuem player dedicado com visualização.
- `aac`, `ac3`, `aif`, `aifc`, `aiff`, `amr`, `ape`, `caf`, `dts`, `flac`, `m4a`, `m4r`, `mp2`, `mp3`, `mka`, `ra`, `ogg`, `oga`, `opus`, `spx`, `wav`, `wma`, `wv` - Suportados para reprodução.

## 🔡 Fontes Tipográficas
Arquivos de fontes possuem um leitor de glifos dedicado.
- `otf`, `ttc`, `ttf`, `woff`, `woff2` - Com miniatura e visualização completa.

---

## ⚠️ Limitações e Problemas Identificados
Devido à natureza complexa dos formatos proprietários ou características pesadas de renderização, alguns formatos apresentam problemas já mapeados ou qualidade reduzida.

### Qualidade Reduzida ou Inconsistências
Estes formatos podem ser abertos, mas poderão exibir a miniatura de forma pixelada/baixa qualidade, ou o preview pode não representar todas as camadas perfeitamente:
- Corel Draw (`cdr`) - Baixa qualidade de miniatura (thumbnail) e preview.
- Gimp (`xcf`) - Problemas com documentos muito elaborados (modos de camada e máscaras).
- Medibang / FireAlpaca (`mdp`) - Baixa qualidade de miniatura (thumbnail) e preview.
- Paint Tool SAI (`sai`) - Baixa qualidade de miniatura (thumbnail) e preview.
- Áudio/MIDI (`aax`, `mid`, `midi`, `bwf`) - Estão com problemas de reprodução.
- Vídeo Ogg (`ogv`) - Revisar thumbnail e player.
- Vídeo HEVC (`hevc`) - Revisar thumbnail, o player usa HLS linear.
- Apple `heic`, `heif` - Apresentam thumbnail funcional, porém algumas têm problemas para visualização.
- Penpot (`penpot`) - **Apenas a versão 1 funciona**. A versão 2 não possui thumbnail ou preview, sendo necessário o Mundam realizar um render completo do projeto (não suportado atualmente).

### Sem Suporte Vigente (Arquivo não abre / Sem arte)
Os formatos abaixo não conseguem ser abertos corretamente. O app não exibe a arte visual (nem thumbnail, nem preview):
- Paint Tool SAI 2 (`sai2`) - Sem thumbnail e sem preview.
- RAW Estuturais: `dcr`, `fff`, `iiq`, `raw`, `x3f`, `mef`, `mdc` - Sem thumbnail e sem preview.
- PostScript Clássico (`eps`, `ps`) - Sem miniatura (thumb) e sem preview.
- EOT (`eot`) - Formato de fonte antiga EOT não está sendo trabalhado.
