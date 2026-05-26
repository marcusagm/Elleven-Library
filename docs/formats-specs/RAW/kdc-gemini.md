# Kodak Digital Camera RAW (.kdc) File Format Technical Specification

## Format Overview

* **Extension Name**: .kdc
* **Possible Origin**: Eastman Kodak Company.
* **Category**: Camera RAW Image.
* **LibRaw Support**: Suportado. O LibRaw possui suporte maduro e herdado (via dcraw) para praticamente todos os modelos de câmeras Kodak que geram arquivos KDC.
* **FFMPEG Support**: Extremamente limitado e não confiável. O FFmpeg pode tentar ler o arquivo baseando-se no fato de ser estruturalmente um TIFF, extraindo no máximo um thumbnail quebrado, mas falhará em decodificar corretamente os dados brutos da matriz de cores (CFA) e os perfis de cor da Kodak.
* **Rust alternative converters**:
* **libraw-rs**: Essencial para a decodificação completa do RAW, debayering e precisão de cores.
* **kamadak-exif / tiff**: Crates nativos em Rust excelentes para parsing da estrutura TIFF e extração do thumbnail JPEG embutido sem depender de C/C++.



## File structure

A estrutura do formato KDC depende bastante da época em que a câmera foi lançada, mas, de modo geral, ele é **baseado no padrão TIFF (Tagged Image File Format)** ou na variante TIFF/EP (para os modelos mais recentes como a série P e Pro SLR).

1. **Header TIFF Clássico**:
* Inicia com o "Byte Order" (geralmente II para Little-Endian ou MM para Big-Endian).
* Seguido pelo "Magic Number" do TIFF (42 / 0x002A).
* Aponta para a primeira IFD (Image File Directory).


2. **Organização em IFDs (Image File Directories)**:
* **IFD Principal (Metadata & Thumbnail)**: Onde residem os dados EXIF básicos (fabricante, modelo, tempo de exposição, ISO) e, frequentemente, um thumbnail de baixa resolução.
* **Sub-IFDs (Preview)**: A Kodak costumava adicionar diretórios secundários contendo uma imagem de pré-visualização maior, totalmente processada pela câmera (geralmente em JPEG).
* **Raw Data IFD**: O diretório que aponta para os dados reais capturados pelo sensor (Matriz de Filtro de Cor - CFA). Estes dados podem estar descomprimidos, mas frequentemente utilizam compressão sem perdas (lossless) proprietária da Kodak baseada em DPCM ou variantes de Huffman.


3. **Tags Proprietárias (Kodak MakerNotes)**:
* O arquivo possui metadados específicos da Kodak embutidos nas MakerNotes que definem perfis de cor, balanço de branco e curvas tonais proprietárias (conhecidas como "Kodak Color Science").



## Strategy for Thumbnail Generation

Como o KDC é um contêiner TIFF, a extração de miniaturas é extremamente eficiente e não exige processamento da imagem crua.

1. **Parser de TIFF/EXIF em Rust**: Utilize o crate kamadak-exif ou tiff para ler o cabeçalho e saltar diretamente para os blocos de diretório (IFDs).
2. **Navegação de IFDs**: Busque pelas tags relacionadas à miniatura/preview, como Compression (onde 6 geralmente indica JPEG).
3. **Leitura de Offset e Tamanho**: Identifique as tags JpegIFOffset (onde o JPEG começa) e JpegIFByteCount (o tamanho do JPEG em bytes).
4. **Extração Direta**:
* Posicione o cursor de leitura do arquivo no offset encontrado.
* Leia a quantidade de bytes especificada diretamente para um buffer em memória.
* Repasse este buffer para o crate image (função image::load_from_memory) para garantir que o JPEG está íntegro.
* Redimensione (se necessário) e codifique o resultado final como webp. Essa operação é quase instantânea (zero decodificação de matriz RAW).



## Strategy for Visualization

Diferente da extração do thumbnail, visualizar o RAW real com qualidade máxima exige o processo completo de revelação (pipeline de processamento de imagem), pois os sensores Kodak são baseados em matrizes Bayer clássicas.

1. **Delegação via FFI**: Não tente decodificar o mosaico do sensor e os metadados de cor em Rust puro. Utilize libraw-rs.
2. **Pipeline de Renderização**:
* Carregue o .kdc através do contexto do LibRaw.
* Extraia os multiplicadores de balanço de branco calculados pela câmera no momento do disparo.
* Execute a etapa de **Demosaicing (Debayering)**. O LibRaw aplicará os algoritmos padrão (como AHD ou VNG) para transformar os pixels de uma única cor (R, G ou B) do sensor Bayer em pixels RGB completos.
* **Gerenciamento de Cores**: Aplique a matriz de cores específica da câmera (Camera Matrix) para transpor os valores lineares lidos pelo sensor para um espaço de cor padronizado como sRGB.
* Importe a matriz final de bytes interpolada de volta para o Rust e renderize na tela via WGPU/OpenGL, ou exporte para um formato rasterizado de alta qualidade.



## Uncertain Points

* **Fragmentação de Gerações**: KDCs gerados por câmeras dos anos 90 (como DC40, DC50 e DC120) possuem uma estrutura bastante esotérica que se afasta da conformidade com o padrão TIFF moderno. Parsers de TIFF genéricos em Rust podem falhar catastroficamente nestes arquivos muito antigos na tentativa de extrair o thumbnail, exigindo tratamento de exceções robusto e "fallbacks" para decodificação total via LibRaw.
* **Ausência de Padrão JPEG**: Em modelos mais antigos, a imagem de preview pode não estar comprimida em JPEG, mas sim ser um bitmap RGB em baixa resolução e sem compressão. O seu extrator precisa checar a tag Compression antes de assumir que é um JPEG.

## Other informations

* A divisão de câmeras da Kodak foi descontinuada há anos, tornando o formato um artefato histórico. No entanto, muitos fotógrafos ainda mantêm esses arquivos vivos por causa da lendária "Kodak Color Science" (ciência de cores da Kodak), que oferecia tons de pele muito procurados na época.
* Quando processamos um .kdc via open-source (LibRaw), obtemos uma decodificação perfeitamente precisa matematicamente, mas muitas vezes ela não se parece *exatamente* com as cores vibrantes que o software original da Kodak (Kodak Professional DCS) entregava, pois o software proprietário aplicava curvas tonais não documentadas.
