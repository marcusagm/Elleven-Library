# GoPro RAW (.gpr) File Format Technical Specification

## Format Overview

* **Extension Name**: .gpr
* **Possible Origin**: GoPro (Introduzido a partir da GoPro HERO5 Black).
* **Category**: Camera RAW Image (Baseado no padrão Adobe DNG).
* **LibRaw Support**: Suportado. O LibRaw possui integração para decodificar a compressão VC-5 usada no formato, permitindo a extração completa dos dados do sensor.
* **FFMPEG Support**: Extremamente limitado a nulo. Embora o FFmpeg possa reconhecer o contêiner como TIFF/DNG, ele falha em decodificar o fluxo de vídeo/imagem raw interno porque não suporta nativamente o algoritmo de compressão VC-5 (CineForm) para imagens estáticas.
* **Rust alternative converters**:
* **libraw-rs**: A solução mais robusta via FFI para lidar com o pipeline RAW e a descompressão.
* **gpr_sdk (via bindings FFI)**: A GoPro abriu o código do seu SDK em C++. Criar um wrapper em Rust (usando bindgen) para esta biblioteca oficial é a forma mais nativa e garantida de lidar com o formato.
* **kamadak-exif / tiff**: Crates em Rust puro perfeitos para navegar pelo contêiner DNG/TIFF e extrair os JPEGs embutidos.



## File structure

A grande sacada técnica do formato .gpr é que ele é, na verdade, **um arquivo Adobe DNG (Digital Negative) válido**. A estrutura do contêiner segue rigorosamente a especificação TIFF/EP da Adobe. A diferença fundamental está na compressão dos dados do sensor.

1. **Header e Estrutura TIFF**:
* Arquivo inicia com a assinatura TIFF (II ou MM e o magic number 42).
* É organizado em IFDs (Image File Directories).


2. **IFD0 e Sub-IFDs (Previews)**:
* A IFD principal geralmente contém os metadados EXIF/MakerNotes e aponta para um JPEG de resolução total ou reduzida perfeitamente processado pela própria câmera.
* Pode haver outras IFDs menores com miniaturas adicionais.


3. **Raw Data IFD (O Segredo do GPR)**:
* O diretório que aponta para os dados RAW da matriz Bayer possui a tag Compression configurada para um valor proprietário que aciona o **VC-5 (CineForm)**.
* Em vez da tradicional compressão Lossless JPEG usada em DNGs comuns, a GoPro usa compressão baseada em *wavelets* (VC-5). Isso permite que arquivos que teriam 20MB em um DNG comum caiam para ~10MB, permitindo gravação super-rápida no cartão SD da câmera sem perda de qualidade visual.



## Strategy for Thumbnail Generation

Como o formato obedece ao padrão DNG/TIFF, a extração de miniaturas é extremamente padronizada e não exige lidar com a compressão complexa da GoPro.

1. **Leitura de Estrutura DNG em Rust**: Utilize os crates kamadak-exif ou tiff para ler as IFDs.
2. **Localização do JPEG Embutido**:
* Itere sobre as IFDs procurando por aquelas cuja tag Compression seja 6 (JPEG) ou que contenham a tag NewSubfileType indicando um preview/thumbnail (1 = imagem de resolução reduzida).
* Localize as tags StripOffsets (ou JpegInterchangeFormat) e StripByteCounts (ou JpegInterchangeFormatLength).


3. **Extração e Conversão Rápida**:
* Faça o slice no buffer do arquivo usando o *offset* e o tamanho encontrados.
* Esse slice é um arquivo JPEG completo. Passe-o para image::load_from_memory.
* Redimensione e salve como webp. Essa operação consome pouquíssima CPU.



## Strategy for Visualization

Para visualizar os dados brutos reais, o desafio central é a descompressão do VC-5.

1. **Contorno da Compressão via FFI**: É inviável reescrever um decodificador de wavelets VC-5 puro em Rust a curto prazo. A estratégia correta é delegar isso.
2. **Duas vias de arquitetura**:
* **Via LibRaw**: Usar libraw-rs para carregar o arquivo, executar a descompressão VC-5 internamente, fazer o Debayering, aplicar a matriz de cor DNG e devolver os bytes em sRGB 16-bit ou 8-bit para o Rust renderizar via GPU (WGPU).
* **Via GPR SDK Oficial**: O SDK da GoPro permite carregar o .gpr e convertê-lo diretamente em memória para um buffer DNG descompactado (linear). Você pode então usar pipelines de renderização genéricos de DNG para mostrar a imagem.


3. **Gerenciamento de Cor DNG**: O .gpr inclui tags padrão Adobe (ColorMatrix1, ColorMatrix2, AsShotNeutral, ForwardMatrix). O pipeline de renderização precisará aplicar essas matrizes matemáticas para transformar os dados RGB lineares do sensor em cores precisas visíveis em monitores.

## Uncertain Points

* **Distorção de Lente (Fisheye)**: Câmeras GoPro possuem lentes ultra-angulares com altíssima distorção de barril (fisheye). O arquivo .gpr inclui "OpcodeLists" (instruções DNG) para corrigir matematicamente essa geometria (warp). Extrair a imagem base vai mostrá-la arredondada. O LibRaw nem sempre aplica Opcodes geométricos automaticamente. Pode ser necessário passar os parâmetros de correção para um shader WGPU customizado para "desentortar" a imagem RAW se o usuário quiser a visão linear/Wide padrão da GoPro.
* **Implementação do VC-5 Codec**: Embora aberto pela GoPro, integrar o SDK nativo da GoPro no Rust via FFI pode ser desafiador do ponto de vista de build systems (CMake/Cargo), especialmente garantindo cross-compilation para Windows, Mac e Linux.

## Other informations

* Quando um usuário tira uma foto RAW na GoPro, ela geralmente salva um par de arquivos: .jpg e .gpr. Se o seu software tem acesso ao diretório do usuário e encontra o JPEG com o mesmo nome, usar esse arquivo paralelo para gerar a miniatura WebP pode ser ainda mais rápido do que fazer parsing dentro do arquivo .gpr.
* O algoritmo VC-5 não é estritamente "lossless" (sem perdas) matemático, mas sim "visually lossless". Ele descarta informações microscópicas que o olho humano não percebe, o que justifica seu tamanho incrivelmente otimizado se comparado a arquivos de câmeras DSLR ou Mirrorless.
