# Phase One Intelligent Image Quality (`.iiq`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.iiq` (Intelligent Image Quality)
* **Possible Origin**: Phase One (e frequentemente associado a costas digitais Mamiya Leaf).
* **Category**: Camera RAW Image (Medium Format).
* **LibRaw Support**: Suportado. O LibRaw possui algoritmos específicos para decodificar as duas principais variantes de compressão deste formato (IIQ-L e IIQ-S).
* **FFMPEG Support**: Limitado a nulo. O FFmpeg falhará em tentar ler os dados brutos do sensor, esbarrando na compressão proprietária, mas pode ser capaz de enxergar o arquivo superficialmente como um TIFF genérico.
* **Rust alternative converters**:
* **`libraw-rs`**: Mandatório para lidar com a decodificação da pesada matriz Bayer e a descompressão proprietária da Phase One.
* **`tiff` / `kamadak-exif**`: Soluções puras em Rust perfeitas para navegar pela estrutura TIFF e extrair o JPEG embutido sem depender de bibliotecas C/C++.



## File structure

O formato `.iiq` é construído sobre a arquitetura sólida e padronizada do **TIFF (Tagged Image File Format)**, projetado para lidar com volumes massivos de dados gerados por sensores de médio formato (que podem ultrapassar 150 megapixels).

1. **Header TIFF**: Inicia com o padrão de ordem de bytes (`II` ou `MM`) seguido pela assinatura TIFF (`42`).
2. **Diretórios de Imagem (IFDs)**:
* **IFD Principal/Exif**: Armazena metadados padronizados sobre a captura (ISO, abertura, velocidade) e os metadados específicos da Phase One (MakerNotes).
* **Preview IFD**: Contém um JPEG de alta resolução e excelente qualidade já processado pela câmera. A Phase One costuma incluir previews maiores e melhores que a média do mercado para garantir zoom rápido no display da câmera.


3. **Raw Data IFD**: Aponta para os dados reais do sensor. A característica vital deste formato é a ramificação em dois tipos de gravação (compressão):
* **IIQ-L (Large / Lossless)**: Compressão matemática estritamente sem perdas, gerando arquivos enormes com 16-bit de profundidade por pixel.
* **IIQ-S (Small / Smart)**: Compressão visualmente sem perdas (lossy inteligente), que reduz drasticamente o tamanho do arquivo (quase pela metade ou um terço) descartando informações tonais que a Phase One garante não afetar a percepção humana nas sombras e highlights.



## Strategy for Thumbnail Generation

A extração para WebP pode ser feita em milissegundos utilizando ferramentas nativas em Rust, isolando a complexidade da imagem RAW.

1. **Parsing da Estrutura**: Inicie a leitura do binário usando `kamadak-exif` ou `tiff` para pular o cabeçalho e ler a tabela de diretórios.
2. **Localização do Preview**: Itere pelas IFDs buscando o diretório que possui a tag `Compression` definida como `6` (JPEG) ou que contenha a tag `NewSubfileType` indicando um arquivo de pré-visualização.
3. **Extração de Bytes**: Capture as tags `StripOffsets` (posição de início do JPEG no binário) e `StripByteCounts` (tamanho do JPEG).
4. **Conversão e Redimensionamento**:
* Faça o slice direto desses bytes em memória (`&[u8]`).
* Carregue o buffer através da biblioteca `image` (`image::load_from_memory`).
* Aplique o redimensionamento necessário para sua interface.
* Codifique e salve a saída final em formato `webp`.



## Strategy for Visualization

Para a visualização do arquivo RAW cru, o pipeline requer processamento robusto para lidar com as resoluções massivas.

1. **Delegação da Descompressão (FFI)**: Não implemente a descompressão de IIQ-L ou IIQ-S do zero em Rust. Carregue o arquivo via `libraw-rs`. A biblioteca C base lidará com o algoritmo complexo e a leitura matemática do sensor de 16-bit ou 14-bit comprimido.
2. **Demosaicing e Consumo de Memória**: O debayering de uma imagem de 100 a 150 megapixels exigirá alocações significativas de RAM. Execute o processamento de forma assíncrona ou em uma thread paralela dedicada (como `rayon` ou `tokio::task::spawn_blocking`) para não travar a interface de usuário durante os cálculos.
3. **Gerenciamento de Cores Base**: O LibRaw vai extrair os vetores de balanço de branco gravados no disparo e aplicar a matriz de câmera genérica. Transponha o array RGB linear retornado para um buffer sRGB (para monitores padrão) ou Display P3, utilizando os shaders da sua engine de renderização gráfica (como `wgpu`).

## Uncertain Points

* **Fidelidade da Ciência de Cores**: A Phase One também é a desenvolvedora do aclamado software **Capture One**. A forma como os arquivos `.iiq` são revelados no LibRaw será técnica e linearmente correta, mas dificilmente alcançará a rolagem tonal incrivelmente suave, a recuperação de altas luzes e o micro-contraste que o Capture One aplica secretamente ao interpretar os perfis ocultos nesses mesmos arquivos.
* **Formatos de Sensor Mutáveis**: A Phase One altera constantemente a arquitetura de seus sensores (de CCDs antigos para BSI-CMOS modernos), o que significa que o tipo de ruído e arranjo de pixels dentro do contêiner `.iiq` varia brutalmente entre gerações, podendo exigir correções de black-level diferentes (geralmente tratadas internamente pelo LibRaw, mas que podem apresentar anomalias em câmeras super recentes).

## Other informations

* Câmeras de médio formato da Phase One não costumam aplicar redução de ruído intensa no hardware, confiando inteiramente no motor do software (Capture One). Ao renderizar a imagem via open-source para visualização profunda (100% de zoom), o resultado pode parecer consideravelmente mais ruidoso do que o JPEG embutido ou o preview oficial.
* O formato frequentemente trabalha associado a perfis ICC específicos para calibrar a câmera sob iluminações controladas (reprodução de arte, museus, onde a Phase One domina). Se a aplicação suportar leitura de Box/IFD externa para ICC, o usuário de estúdio terá uma resposta muito melhor na visualização das cores no seu software.
