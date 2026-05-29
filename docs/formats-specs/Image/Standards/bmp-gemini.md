# Windows Bitmap (`.bmp`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.bmp`, `.dib` (Device Independent Bitmap).
* **Possible Origin**: Desenvolvido em conjunto pela Microsoft e IBM em meados da década de 1980 para o Windows 1.0 e OS/2.
* **Category**: Uncompressed / RLE Compressed Raster Image.
* **LibRaw Support**: Não suportado / Não aplicável. O BMP é um formato de entrega e armazenamento de matriz rasterizada (já processada em RGB/BGR), não um formato RAW de dados de sensor fotográfico.
* **FFMPEG Support**: Totalmente suportado. O FFmpeg decodifica e codifica BMPs nativamente e com extrema rapidez devido à simplicidade matemática do formato.
* **Rust alternative converters**:
* **`image`**: A biblioteca padrão de imagens do ecossistema Rust possui suporte nativo e maduro para ler e escrever a maioria das variantes de BMP.
* **`bmp`**: Um crate menor, focado puramente neste formato. Excelente se você quiser minimizar dependências no seu projeto e não precisar de outros formatos.



---

## File structure

O formato BMP é um dos contêineres de imagem mais simples da computação, baseando-se em uma arquitetura de blocos diretos e lineares (gravados em **Little-Endian**).

1. **Bitmap File Header (14 bytes)**:
* **Assinatura**: Inicia com os caracteres mágicos `BM` (`0x42 0x4D`). (Existem assinaturas raras de OS/2 como `BA`, `CI`, mas `BM` compreende 99% do mercado).
* **Tamanho**: O tamanho total do arquivo.
* **Offset**: Um ponteiro (4 bytes) indicando o endereço exato onde a matriz de pixels real começa, permitindo pular os cabeçalhos.


2. **DIB Header (Bitmap Information Header)**:
* Este bloco varia de tamanho dependendo da versão do formato (Windows V3, V4, V5, OS/2). O mais comum (BITMAPINFOHEADER) possui 40 bytes.
* Contém a Largura e Altura (em pixels), Número de Planos de Cor (sempre 1), e a **Profundidade de Bits** (1, 4, 8, 16, 24 ou 32 bits por pixel).
* Contém o método de compressão (0 para Não Comprimido, 1 para RLE8, 2 para RLE4, 3 para Bitfields).


3. **Color Table (Palette)**:
* Obrigatória para BMPs de 1, 4 ou 8 bits (imagens indexadas). Define as cores exatas que os índices apontam.
* Opcional ou ausente para BMPs de 16, 24 e 32 bits (True Color).


4. **Pixel Array (Dados Brutos)**:
* A grade de pixels propriamente dita. Historicamente gravada na ordem **BGR** (Blue, Green, Red) e não RGB.
* **Regra do Padding (Preenchimento)**: O tamanho de cada linha (stride) da imagem *deve obrigatoriamente* ser um múltiplo de 4 bytes. Se a largura da imagem não fechar em um múltiplo de 4, bytes nulos (padding) são adicionados ao final de cada linha.



---

## Strategy for Thumbnail Generation

Diferente de formatos como `.arw` ou `.tiff`, o arquivo `.bmp` **não possui miniaturas embutidas**. Gerar um thumbnail exige acessar a matriz principal de pixels.

1. **Parsing Direto**: Como não há descompressão complexa (na maioria dos casos), ler um BMP é essencialmente ler um bloco da memória. Utilize o crate `image` para abrir o arquivo.
2. **Estratégia de Redimensionamento**:
* Como o arquivo já é uma grade RGB/BGR pronta, chame o método de carregamento da biblioteca e aplique um redimensionamento (`image::imageops::resize`).
* Para geração de thumbnails rápidos onde a qualidade perfeita de downscaling não é vital, use o filtro `NearestNeighbor` ou `Triangle`.


3. **Otimização de RAM (Sub-amostragem via IO)**: Se o arquivo BMP for gigantesco (ex: 200 MB não comprimidos) e você quiser evitar alocar tudo em RAM só para gerar um thumbnail de 256x256, você pode usar a interface de I/O nativa do Rust (`std::io::Seek`).
* Leia o DIB header para descobrir a largura e o padding.
* Pule os bytes no disco (`Seek`) lendo apenas um a cada *N* pixels (sub-amostragem) diretamente para um buffer menor em memória.


4. **Encoding**: Transfira o buffer escalonado resultante para o codificador WebP.

---

## Strategy for Visualization

A visualização de um BMP exige contornar algumas idiossincrasias históricas do design da Microsoft dos anos 80.

1. **A Pegadinha do Bottom-Up (De Baixo para Cima)**:
* No cabeçalho DIB, verifique a **Altura**. Se o valor for **positivo**, a imagem foi gravada de baixo para cima (a primeira linha de bytes no arquivo é a base da imagem na tela).
* Se a Altura for **negativa**, a imagem é Top-Down (de cima para baixo, o padrão da computação moderna).
* Se for positivo (o padrão BMP clássico), seu interpretador precisará espelhar a matriz verticalmente no Rust antes de enviar para a tela, ou inverter o eixo Y no mapeamento de textura UV no WGPU/OpenGL.


2. **Descarte de Padding e Reordenação BGR**:
* Seu iterador de leitura de linha precisará saltar (ignorar) matematicamente os bytes de padding no final de cada linha, senão a imagem renderizada ficará totalmente inclinada e distorcida (skewed).
* O array lido estará frequentemente em formato BGR. Se o seu renderizador gráfico exige RGBA, você precisará iterar o buffer (via `chunks_exact_mut`) para fazer o swap dos bytes Blue e Red e injetar o Alpha opaco (255) onde necessário.



---

## Uncertain Points

* **Canal Alpha em BMP de 32-bits**: Tradicionalmente, BMPs de 32-bits (BITMAPINFOHEADER V3) usavam 24 bits para cor e os 8 bits restantes não serviam para nada (padding puro por pixel). No entanto, o Windows moderno e criadores de imagem passaram a usar esses 8 bits para transparência (Formato BGRA). Se o cabeçalho for moderno (V4 ou V5), ele declarará explicitamente o canal Alpha. Se for antigo, seu parser ficará no "escuro" para adivinhar se os bytes extras são lixo matemático ou uma máscara de opacidade válida. A prática comum é assumir que o Alpha é 100% opaco a menos que os cabeçalhos V4/V5 atestem o contrário.
* **Compressão RLE (Run-Length Encoding)**: Embora 99% dos BMPs não tenham compressão, o formato suporta `RLE8` (para 8-bits) e `RLE4` (para 4-bits). Se o seu parser for implementado puramente do zero em vez de usar o crate `image`, a decodificação da lógica de RLE do BMP possui algumas regras excêntricas (marcadores de fim de linha, fim de bitmap e saltos de coordenadas) que são propensas a falhas se não forem tratadas corretamente.

---

## Other informations

* **Performance vs Armazenamento**: O BMP é um formato computacionalmente "gratuito". O uso de CPU para abrir um BMP e jogar na tela é próximo de zero. Em contrapartida, é um desastre de armazenamento. Uma imagem fotográfica de 24 megapixels em BMP pesará invariavelmente ~72 Megabytes, independentemente da complexidade visual da foto.
* **Engenharia e Testes**: Devido à sua ausência de matemática complexa (como wavelets ou DCTs), o BMP é frequentemente o formato escolhido por desenvolvedores Rust ao construir engines gráficas do zero para testar se seus ponteiros de memória e buffers de framebuffer estão operando na ordem correta. Ele será muito útil na sua aplicação para validar o pipeline final de cores.
