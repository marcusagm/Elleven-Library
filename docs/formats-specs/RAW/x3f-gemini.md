# Sigma Foveon RAW (.x3f) File Format Technical Specification

## Format Overview

* **Extension Name**: .x3f
* **Possible Origin**: Sigma Corporation (desenvolvido originalmente pela Foveon Inc., adquirida pela Sigma).
* **Category**: Camera RAW Image.
* **LibRaw Support**: Suportado. O LibRaw possui decodificadores específicos (frequentemente herdados do código legado x3f_extract ou engenharias reversas do dcraw) para lidar com a arquitetura única do sensor Foveon.
* **FFMPEG Support**: Extremamente limitado a nulo. O FFmpeg nativamente tem muita dificuldade com metadados e compressão proprietária de arquivos X3F, muitas vezes falhando ao tentar extrair o RAW ou as miniaturas embutidas sem o auxílio de bibliotecas externas de fotografia.
* **Rust alternative converters**:
 - **libraw-rs**: A melhor alternativa para renderização completa e fiel, utilizando FFI (Foreign Function Interface) para a biblioteca em C/C++.
 - **nom ou binrw**: Para a extração *apenas* do thumbnail (que é um JPEG embutido), é altamente recomendável escrever um parser nativo em Rust para navegar na árvore de diretórios do arquivo binário e extrair o buffer do JPEG, dispensando bibliotecas pesadas de RAW.
 - **rawloader**: O crate nativo rawloader foca em sensores Bayer e X-Trans, não oferecendo suporte robusto ou decodificação de imagem completa para sensores Foveon.



## File structure

O formato X3F não é baseado no padrão TIFF/EP (como DNG, CR2 ou NEF), o que o torna fundamentalmente diferente de quase todos os outros formatos RAW do mercado. Ele é um formato de contêiner em bloco (block-structured).

1. **Header Principal (File Header)**:
* Inicia com o "Magic Number" (assinatura): FOVb (Foveon binary).
* Contém a versão do formato (ex: 2.0, 3.0, 4.0).
* Aponta para um ponteiro de diretório (Directory Pointer) que indica onde o índice de blocos está localizado no arquivo.


2. **Seções / Blocos (Sections)**:
Cada seção possui um cabeçalho identificador de 4 bytes e armazena dados específicos. As seções mais comuns são:
* SECp (Properties): Metadados em formato de texto (pares chave/valor), incluindo ISO, tempo de exposição, abertura, etc.
* SECi (Image Data): Blocos de imagem. O arquivo geralmente contém múltiplos blocos SECi:
* Miniatura (Thumbnail): Geralmente um JPEG de baixa resolução.
* Pré-visualização (Preview): Geralmente um JPEG em média/alta resolução com balanço de branco e cores aplicados pela câmera.
* RAW Data: Os dados brutos do sensor (frequentemente comprimidos usando um algoritmo de Huffman proprietário).


* SECc (Camera Data/CAMF): Metadados estruturados da câmera, matrizes de calibração de cores e curvas de resposta do sensor necessárias para revelar o RAW corretamente.


3. **Diretório (Directory)**:
Uma tabela localizada no final ou indicada pelo header principal que mapeia o deslocamento (offset) em bytes e o tamanho de cada bloco SEC presente no arquivo, permitindo leitura por acesso aleatório (random access).

## Strategy for Thumbnail Generation

Para gerar um thumbnail de qualidade convertendo para webp de forma extremamente rápida e com baixo custo de CPU:

1. **Evitar decodificação RAW**: **Não** tente decodificar os dados do sensor para gerar um thumbnail.
2. **Parser Binário Direto**: Implemente um parser simples usando binrw ou leitura de ponteiros de bytes em Rust.
3. **Leitura da Árvore de Diretórios**: Leia o Header FOVb e pule diretamente para o Offset do Diretório.
4. **Localização do Preview**: Itere pelas entradas do diretório procurando por blocos SECi (Image). Analise o cabeçalho do bloco SECi para identificar o tipo de imagem contida. Procure pelo formato da imagem (geralmente há um identificador para "JPEG").
5. **Extração e Conversão**:
* Faça a cópia do buffer de bytes contendo o JPEG embutido (Preview de alta resolução).
* Passe esse buffer (em memória) para o crate image (via image::load_from_memory).
* Redimensione caso necessário e encode diretamente para webp.
* *Vantagem*: Essa operação leva milissegundos, pois você está apenas copiando bytes e reencodando um JPEG existente, ignorando a complexidade do sensor.



## Strategy for Visualization

Para a visualização da imagem RAW real com máxima fidelidade (permitindo zoom profundo e exportação para outros formatos):

1. **Compreensão do Sensor Foveon**: Diferente de câmeras convencionais (Bayer), o sensor Foveon captura Vermelho, Verde e Azul *no mesmo pixel* (através de camadas empilhadas de silício). Portanto, **não existe etapa de Demosaicing (Debayering)**.
2. **Uso de Bibliotecas C/C++ via FFI**: Como a decodificação dos dados comprimidos com Huffman do formato X3F e a aplicação das matrizes de cor do bloco SECc requerem algoritmos não documentados oficialmente e extremamente complexos, a estratégia correta em Rust é criar um binding para o LibRaw (via libraw-rs).
3. **Pipeline de Renderização**:
* Inicializar o contexto do LibRaw e carregar o arquivo .x3f.
* Ler os parâmetros de balanço de branco do bloco de metadados.
* Solicitar ao LibRaw o processamento (descompressão do RAW Foveon).
* Aplicar a correção de cor (Camera Matrix) específica do modelo (Merrill, Quattro, etc.).
* Converter o buffer de 16-bit linear resultante em um espaço de cor padrão (sRGB ou AdobeRGB).
* Transmitir a matriz de pixels RGB bruta decodificada do LibRaw de volta para o Rust para ser exibida via OpenGL/WGPU ou convertida.



## Uncertain Points

* **Arquitetura Quattro (Sensores recentes da Sigma)**: Os sensores Quattro (ex: dp2 Quattro) possuem uma proporção assimétrica nas camadas de silício (a camada azul tem 4 vezes mais resolução que a vermelha e a verde). Isso introduz uma etapa híbrida semelhante ao demosaicing para reconstruir a resolução total, cujos algoritmos nativos da Sigma são muito superiores aos de código aberto. A fidelidade open-source para a geração Quattro pode sofrer com ruído e artefatos de cor.
* **Compressão Proprietária**: A compressão sem perdas (lossless) dentro dos blocos de dados RAW utiliza esquemas proprietários que variam entre as gerações das câmeras Sigma. Implementar a descompressão puramente em Rust do zero exige extrema engenharia reversa.

## Other informations

* A Sigma disponibiliza oficialmente o software **Sigma Photo Pro** para o processamento de arquivos X3F. Os resultados de cor e micro-contraste desse software costumam ser muito difíceis de replicar em softwares open-source devido a algoritmos fechados de nitidez que tiram vantagem da ausência do filtro antialiasing do sensor Foveon.
* Nas câmeras mais modernas (linha Sigma fp e fp L), a empresa abandonou a obrigatoriedade do .x3f e adotou o .dng (Digital Negative da Adobe) padrão, além de migrar para sensores Bayer clássicos, deixando o .x3f como um formato histórico para seus corpos Foveon legados.
