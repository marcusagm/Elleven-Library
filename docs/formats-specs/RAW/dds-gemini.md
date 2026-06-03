# DirectDraw Surface (`.dds`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.dds`
* **Possible Origin**: Microsoft (Introduzido nativamente com o DirectX 7.0 em 1999).
* **Category**: Raster Image / Game Texture Container.
* **LibRaw Support**: Não suportado / Não aplicável. O `.dds` é um contêiner de textura focado na GPU para renderização em tempo real (videogames e modelagem 3D). Ele não guarda dados brutos de sensores de câmera (mosaico Bayer), portanto o LibRaw não tem função aqui.
* **FFMPEG Support**: Totalmente suportado. O FFmpeg possui decodificadores robustos para ler e converter arquivos DDS, suportando a esmagadora maioria das compactações de bloco tradicionais (DXT1 a DXT5).
* **Rust alternative converters**:
* **`ddsfile`**: O crate definitivo em Rust para fazer o parsing completo dos cabeçalhos, mipmaps, arrays e faces de cubemaps, incluindo suporte ao cabeçalho moderno DX10.
* **`texture2ddecoder` / `bcndecode**`: Como os dados do DDS frequentemente estão "compactados em blocos" (Block Compression), você precisará de crates como estes para traduzir a matemática do BC1-BC7 em pixels RGBA legíveis se for fazer isso via CPU.
* **`image`**: A biblioteca padrão de imagem do Rust suporta leitura de DDS, mas historicamente possui limitações com formatos DX10 mais obscuros ou compressões modernas (BC7), sendo melhor combiná-la com o `ddsfile`.



## File structure

O DDS foi desenhado para ser uma cópia quase exata da forma como a memória de vídeo (VRAM) estrutura as texturas, permitindo o carregamento direto do HD para a GPU sem muito processamento da CPU.

1. **Magic Word (4 bytes)**: Inicia sempre com a string ASCII `DDS ` (`0x20534444`).
2. **DDS_HEADER (124 bytes)**:
* Armazena metadados cruciais: Altura, Largura, Pitch (tamanho da linha) e Profundidade (se for textura 3D).
* **Mipmap Count**: Informa quantas versões reduzidas da imagem estão embutidas no arquivo.
* **PixelFormat (32 bytes)**: Uma subestrutura que define como as cores são lidas. Pode indicar um formato RGBA cru, ou conter os identificadores "FourCC" (como `DXT1`, `DXT5`, `ATI2`) que sinalizam a compactação de blocos em uso.


3. **DDS_HEADER_DXT10 (Opcional)**: Presente apenas se o PixelFormat contiver o FourCC `DX10`. Adiciona suporte a arrays de textura (múltiplas imagens no mesmo arquivo), formatos DXGI modernos (como ponto flutuante HDR de 16/32 bits) e compactações BC6H/BC7.
4. **Data / Payload (Pixels e Mipmaps)**:
* **Main Surface**: Os dados da imagem em resolução máxima vêm primeiro. Diferente do JPEG, a imagem não é lida linha por linha, mas em blocos independentes de 4x4 pixels se estiver comprimida.
* **Mipmap Chain**: Imediatamente após a imagem principal, o arquivo anexa versões da imagem progressivamente menores (divididas por 2: 512x512, 256x256, 128x128... até 1x1).
* **Cubemaps/Arrays**: Se for um "Skybox" (Cubemap), os dados armazenarão a Face +X com todos os seus mipmaps, depois a Face -X, etc.



## Strategy for Thumbnail Generation

O DDS oferece uma das maiores vantagens de performance para extração de thumbnail se aproveitarmos a sua natureza de renderização 3D: os **Mipmaps**.

1. **Ignorar a Descompressão Total**: Nunca decodifique a imagem de resolução máxima para gerar uma miniatura.
2. **Parsing do Cabeçalho via `ddsfile**`: Leia o arquivo para inspecionar o cabeçalho e verificar o `mipmap_count`.
3. **Seleção Direta do Mipmap**: Identifique o nível de mipmap que melhor atende à sua interface (por exemplo, o nível 3 ou 4 que pode corresponder a 256x256 pixels).
4. **Decodificação de Bloco Isolado**: Isole aquele sub-bloco de memória (offset específico do mipmap escolhido). Descomprima (usando `bcndecode`) apenas aquele bloco de 256x256, transpondo os dados de DXT para um array RGBA de 8-bits puros.
5. **Codificação Rápida**: Envie o buffer linear resultante para a biblioteca `image` (ou semelhante) e salve como `webp`. Isso transforma uma operação pesada de CPU em um recorte instantâneo de memória.

## Strategy for Visualization

O objetivo na visualização de um DDS é reproduzir exatamente o que a placa de vídeo desenharia em um motor de jogo (Game Engine).

1. **Aceleração Direta via GPU (WGPU/OpenGL)**: A maneira mais eficiente e com maior "fidelidade" de visualizar um DDS comprimido (BCn) em Rust não é decodificá-lo na CPU. GPUs modernas entendem os formatos DXT/BC1-BC7 **nativamente**.
* Você lê os bytes puros do arquivo (pulando o cabeçalho).
* Envia esses bytes comprimidos diretamente como uma textura (`wgpu::TextureFormat::Bc1RgbaUnorm`, etc.) para a placa de vídeo através do WGPU.
* O shader cuida de exibir na tela. O uso de CPU e RAM será ínfimo.


2. **Conversão de Formato (Exportação)**: Se o usuário desejar converter e exportar o DDS com qualidade máxima para PNG/JPEG, você será obrigado a instanciar um decodificador em CPU (como o `texture2ddecoder`).
* Você decodificará o formato BC para RGBA linear em memória alocada.
* **Atenção aos Normal Maps**: Muitos DDS não são fotos coloridas, mas "Normal Maps" (Texturas de relevo, frequentemente usando a compressão `BC5` ou `ATI2`). Onde os canais Red e Green representam eixos X e Y geométricos. Salvar isso como JPEG pode destruir os vetores, exigindo formatação PNG.



## Uncertain Points

* **Normalização de Eixo Y (Flipping)**: DirectX e OpenGL utilizam coordenadas de textura com o eixo Y invertido entre si. O formato DDS (sendo do DirectX) assume originalmente que a origem (0,0) está no **topo-esquerdo**, mas muitas ferramentas e engines de OpenGL salvam arquivos DDS com a origem na **base-esquerda**. O arquivo não possui uma tag que declare quem está certo. Ao renderizar, você pode ocasionalmente esbarrar em texturas de cabeça para baixo que o usuário precisará inverter manualmente no seu software.
* **Canal Alpha Multiplicado (Premultiplied Alpha)**: Certas texturas (especialmente de efeitos visuais e fogo) usam Alpha Pré-multiplicado, onde a cor RGB já foi escurecida pelo valor da transparência antes de ser salva no disco. O cabeçalho DDS não avisa quando isso ocorre. O processamento para outros formatos ignorando o estado pré-multiplicado pode resultar em bordas pretas grotescas ao redor da imagem na conversão.

## Other informations

* O DDS é um formato inerentemente destrutivo (Lossy) para a maioria dos casos práticos (excluindo os raros formatos float descomprimidos). A compressão por blocos DXT destrói micro-contrastes e muitas vezes gera "banding" (blocos visíveis no céu e degradês suaves). O usuário do seu software precisa entender que a "fidelidade máxima" na visualização do DDS significa ver o arquivo exatamente com os artefatos de bloco com os quais ele foi gravado; não há como recuperar os pixels perdidos para a compressão.
* Se o DDS contiver um **Cubemap** (usado para reflexos e céus em jogos), o seu interpretador não deve achatar as 6 faces. O ideal é que o seu visualizador extraia a Face 0 (Positiva X ou Positiva Z) como a imagem principal, ou disponha a visualização como uma cruz desdobrada (Net format) se você focar também em modders e desenvolvedores 3D.
