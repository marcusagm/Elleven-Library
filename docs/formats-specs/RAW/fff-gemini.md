
# Hasselblad Flexible File Format (`.fff`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.fff`
* **Possible Origin**: Originalmente desenvolvido pela Imacon para seus scanners de filme de altíssima resolução de ponta (Flextight) e costas digitais (Ixpress). O formato foi herdado e expandido pela Hasselblad após a fusão das duas empresas.
* **Category**: Camera RAW Image / High-End Scanner RAW.
* **LibRaw Support**: Suportado. O LibRaw lida com as complexidades da matriz de cor e da descompressão dos sensores Hasselblad/Imacon modernos e antigos.
* **FFMPEG Support**: Nulo ou extremamente limitado. Assim como no caso do `.3fr`, o FFmpeg não compreende os algoritmos de dados brutos e os metadados proprietários atrelados ao formato, falhando ao tentar extrair a imagem RAW real.
* **Rust alternative converters**:
* `libraw-rs` para lidar com a conversão RAW complexa e as interpolações.
* Crates nativos como `tiff` e `kamadak-exif` para navegação super rápida no contêiner binário e extração das pré-visualizações embarcadas.



## File structure

Assim como o seu formato irmão `.3fr`, o formato `.fff` baseia-se na sólida estrutura do padrão **TIFF (Tagged Image File Format)**, mas com propósitos estendidos:

* **TIFF/EP Base**: Utiliza a estrutura de Image File Directories (IFDs) para organizar diferentes versões da imagem (miniatura, preview) e dados cruciais (sensor RAW).
* **Contêiner "Flexível"**: A principal diferença arquitetônica em relação ao `.3fr` (que é apenas um contêiner de captura direta "in-camera") é que o `.fff` foi projetado como um arquivo de trabalho e arquivamento. Ele possui blocos de dados alocados para armazenar edições não-destrutivas, histórico de processamento de imagem, ajustes de curva tonal e perfis ICC completos, tudo anexado ao final ou em sub-IFDs do arquivo original.
* **MakerNotes Hasselblad/Imacon**: Contém metadados extremamente ricos sobre calibração de lentes específicas, compensação de queda de luz periférica (vignetting) e aberração cromática, que o software oficial lê para aplicar correções automáticas perfeitas.
* **Variantes de Origem**: Estruturalmente, um `.fff` pode vir de uma câmera de médio formato (matriz Bayer) ou de um scanner cilíndrico Flextight (onde os dados podem ser um RGB linear gigante em vez de uma matriz mosaico).

## Strategy for Thumbnail Generation

A estratégia segue o mesmo caminho de alta eficiência usado para outros contêineres TIFF, sendo ideal para execução em Rust puro:

* **Leitura de Cabeçalho TIFF**: Utilize `kamadak-exif` ou `tiff` para ler os primeiros bytes e localizar o ponteiro do primeiro diretório IFD.
* **Filtro de Metadados de Compressão**: Varra as IFDs em busca do identificador de JPEG (tag `Compression` com valor `6`). A Hasselblad sempre embute um JPEG de excelente qualidade (muitas vezes em tamanho real) para visualização rápida no seu software Phocus ou sistemas operacionais.
* **Cópia de Bytes**: Extraia os valores numéricos de `StripOffsets` e `StripByteCounts`. Com esses dados, você sabe exatamente onde o JPEG começa e qual o seu tamanho. Faça um slice direto do arquivo em disco (`mmap` ou carregamento parcial em buffer).
* **Conversão Direta**: Passe o slice em memória para `image::load_from_memory`. Aplique o redimensionamento desejado (ex: Lanczos3) e grave o fluxo de saída em formato `webp`. Isso dispensa totalmente o uso de C++ e ocorre em frações de segundo.

## Strategy for Visualization

Para exibir a imagem RAW real com máxima fidelidade e permitir a exportação para formatos como TIFF/PNG:

* **Delegação de Decodificação via FFI**: O debayering de uma imagem de médio formato que pode chegar a 100+ megapixels (e com um arranjo de pixels específico) deve ser repassado ao `libraw-rs`.
* **Verificação de Origem (Scanner vs. Câmera)**: Ao usar o LibRaw, o seu pipeline precisará verificar os metadados para saber se os dados requerem interpolação (demosaicing para sensores Bayer) ou se os bytes RAW já vêm das linhas de varredura tricolor de um scanner Imacon Flextight (onde cada pixel já possui as 3 cores puras capturadas pelos tubos fotomultiplicadores).
* **Pipeline de Cores Físico**: O LibRaw calculará o balanço de branco a partir dos metadados da captura e aplicará a matriz de cores do fabricante para converter os valores crus do sensor em um espaço linear, que depois você pode mapear para o espaço sRGB para jogar na tela do usuário via uma textura no `wgpu` ou outra biblioteca gráfica nativa do Rust.

## Uncertain Points

* **Fidelidade da Hasselblad Natural Colour Solution (HNCS)**: Assim como no `.3fr`, reconstruir a ciência de cores lendária da Hasselblad em um leitor genérico open-source é matematicamente impossível, pois os perfis ICC e as matrizes tonais (Look-up Tables 3D) do software Phocus são fechados. As cores geradas pelo LibRaw serão corretas e lineares, mas não terão o "look" Hasselblad final automático.
* **Parâmetros de Edição Embutidos**: Como o `.fff` permite guardar os ajustes feitos pelo fotógrafo dentro do arquivo, um leitor construído com LibRaw muito provavelmente irá ignorar esses metadados de edição (ex: aumento de contraste, sombras, highlights) que foram feitos no software nativo, decodificando a foto puramente no estado "Zero" (As Shot).

## Other informations

* **Fluxo de Trabalho Tethered**: Se um usuário estiver utilizando a câmera Hasselblad conectada diretamente ao computador por um cabo (tethering) usando o software Phocus, a imagem sequer existe como `.3fr`. O software captura o fluxo de dados diretamente do sensor e cria o arquivo `.fff` instantaneamente no disco rígido.
* **Uso de Memória**: Por tratar-se de resoluções colossais (arquivos podem ter 100MB a 300MB cada), é vital que sua implementação em Rust gerencie rigorosamente a alocação de memória ao extrair a matriz RAW de dentro do LibRaw, destruindo os ponteiros não utilizados (contextos do FFI) o mais rápido possível para evitar OOM (Out Of Memory) ao navegar por pastas contendo muitos arquivos.
