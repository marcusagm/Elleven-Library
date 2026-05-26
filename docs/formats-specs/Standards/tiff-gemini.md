# Tagged Image File Format (`.tiff` / `.tif`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.tiff`, `.tif`
* **Possible Origin**: Criado pela Aldus Corporation (atualmente Adobe Systems) em meados da década de 1980 para padronizar imagens escaneadas.
* **Category**: Raster Image / Container Format.
* **LibRaw Support**: Limitado/Não aplicável. O LibRaw foca em imagens RAW de matriz de filtro de cor (Bayer/X-Trans). Embora muitos RAWs usem a estrutura TIFF (como DNG), um arquivo `.tiff` padrão já é uma imagem processada (demosaiced) em RGB, CMYK ou Tons de Cinza. O LibRaw não é a ferramenta correta para TIFFs convencionais.
* **FFMPEG Support**: Totalmente suportado. O FFmpeg possui codificadores e decodificadores nativos muito maduros para ler sequências de TIFFs ou arquivos individuais, suportando as compressões mais comuns (LZW, Deflate, PackBits, RAW).
* **Rust alternative converters**:
* **`tiff`**: O crate nativo em Rust padrão da comunidade. Altamente recomendado para decodificar strips, tiles e a estrutura de IFDs.
* **`image`**: Crate de alto nível que encapsula o crate `tiff` para operações simples, permitindo a conversão fácil para outros formatos (embora perca o controle fino sobre IFDs secundárias).



## File structure

O TIFF é o "avô" da maioria dos formatos RAW modernos. Ele é um contêiner extremamente flexível baseado em diretórios, permitindo armazenar múltiplas imagens e metadados arbitrários em um único arquivo.

1. **Header (Cabeçalho)**: Exatos 8 bytes de tamanho.
* **Byte Order**: Os dois primeiros bytes indicam a ordem (Endianness). `II` (`0x4949`) para Little-Endian (Intel) ou `MM` (`0x4D4D`) para Big-Endian (Motorola).
* **Magic Number**: O número 42 (`0x002A`), que identifica o arquivo como um TIFF.
* **Offset**: Um inteiro de 32 bits apontando para o endereço de bytes onde o primeiro diretório (IFD) começa.


2. **IFD (Image File Directory)**: O núcleo estrutural.
* Começa com o número de entradas (tags).
* Cada tag possui 12 bytes: Tag ID (2 bytes), Tipo de Dado (2 bytes), Contagem (4 bytes) e o Valor ou Offset para o valor (4 bytes).
* Termina com um offset de 4 bytes apontando para a próxima IFD (permitindo arquivos multi-página), ou 0 se for a última.


3. **Organização dos Pixels**:
* **Strips (Faixas)**: A imagem é fatiada em linhas horizontais. As tags `StripOffsets` e `StripByteCounts` indicam onde cada fatia está armazenada.
* **Tiles (Ladrilhos)**: Comum em TIFFs gigantes, a imagem é dividida em blocos retangulares (ex: 256x256), gerenciados pelas tags `TileOffsets` e `TileByteCounts`.


4. **Compressão**: Definida pela tag `Compression`. Pode ser Descomprimido (1), CCITT (para fax), LZW (5), JPEG (6 e 7), Deflate/ZIP (8) ou PackBits (32773).

## Strategy for Thumbnail Generation

Em arquivos TIFF padrão, a miniatura pode ou não estar presente como uma página secundária. A abordagem em Rust deve ser progressiva.

1. **Varredura de IFDs (Páginas)**: Utilize o crate `tiff` para ler a estrutura do arquivo sem carregar os pixels principais. Procure por uma IFD onde a tag `NewSubfileType` tenha o bit 0 ativado (valor `1`), o que indica que esta página é uma versão de resolução reduzida (thumbnail/preview) da imagem principal.
2. **Extração de Thumbnail Embutido**: Se a IFD reduzida for encontrada, decodifique apenas os strips/tiles dessa IFD específica, carregue o buffer na memória e repasse para o codificador WebP.
3. **Fallback para Redimensionamento**: Se o TIFF contiver apenas a imagem principal (uma única IFD), você precisará decodificar a imagem toda.
* Leia os pixels usando o crate `image`.
* Aplique a função genérica `image::imageops::resize` utilizando um filtro de interpolação rápido (como Triangle ou Lanczos3).
* Salve o buffer alocado resultando no formato `webp`.



## Strategy for Visualization

O desafio da visualização do TIFF não é o demosaicing (como nos formatos RAW), mas sim a vasta quantidade de espaços de cor, profundidades de bits e compressões possíveis.

1. **Decodificação Nativa em Rust**: Utilize o crate `image` ou `tiff` para reconstruir a matriz de pixels a partir das faixas ou ladrilhos, lidando automaticamente com as compressões LZW ou ZIP.
2. **Normalização de Espaço de Cor**:
* **RGB/RGBA**: Podem ser exibidos diretamente via textura WGPU.
* **Grayscale (Tons de Cinza)**: Precisam ser expandidos (replicando o valor do canal único para R, G e B) antes de serem enviados para a GPU.
* **CMYK**: Muito comum em TIFFs de impressão. Você não pode exibir CMYK diretamente. Seu software precisará de uma rotina matemática (ou utilizar a biblioteca `lcms2` em Rust) para converter os valores substrativos CMYK em aditivos RGB.
* **Indexed (Paleta)**: Requer mapeamento dos índices para a tabela de cores (ColorMap) embutida no TIFF.


3. **Profundidade de Bits**: TIFFs frequentemente operam em 16-bit ou 32-bit (float) por canal. Para exibição de máxima fidelidade sem banding, passe o buffer de 16-bit nativo para a textura gráfica e aplique o mapeamento de tons (SDR/HDR) e a curva de Gamma no shader da aplicação.

## Uncertain Points

* **"Thousands of Incompatible File Formats"**: Este é um apelido real dado ao TIFF na indústria. Devido à sua flexibilidade, muitos fabricantes criaram tags privadas não documentadas. Se o seu interpretador depender de premissas estritas, ele falhará ao encontrar um TIFF que possua canais extras arbitrários (Alpha, Spot Colors, Z-Depth).
* **Compressão JPEG Legada**: O TIFF possui duas tags de compressão JPEG: a "Old JPEG" (Compression = 6) implementada no TIFF 6.0 original, que é notoriamente confusa e ambígua, e a "New JPEG" (Compression = 7) introduzida mais tarde. Muitos decodificadores (incluindo o crate `tiff` nativo em Rust) podem ter dificuldades ou falhar ao ler TIFFs antigos que usam a controversa "Old JPEG".

## Other informations

* **BigTIFF**: O padrão TIFF original utiliza offsets de 32 bits, o que limita o tamanho máximo do arquivo a 4 Gigabytes. Para imagens médicas, astronômicas ou escaneamentos de altíssima resolução, foi criado o padrão BigTIFF (com offsets de 64 bits e Magic Number `0x002B`), que suporta arquivos virtualmente infinitos. O crate `tiff` em Rust já suporta a leitura de arquivos BigTIFF.
* **Metadados Abundantes**: Ao contrário do PNG ou WebP, o TIFF é frequentemente o formato preferido por museus e arquivos institucionais porque ele permite anexar blocos colossais de metadados XMP, IPTC, EXIF e perfis ICC completos na mesma estrutura, sem afetar o bloco de pixels. A preservação destas tags em uma eventual conversão requer leitura minuciosa das IFDs.
