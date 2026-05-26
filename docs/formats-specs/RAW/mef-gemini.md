# Mamiya Electronic Format (`.mef`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.mef`
* **Possible Origin**: Mamiya Digital Imaging (usado primariamente na câmera Mamiya ZD e nos primeiros backs digitais Mamiya ZD Back).
* **Category**: Camera RAW Image (Medium Format).
* **LibRaw Support**: Suportado. O formato é perfeitamente legível pelo LibRaw, que decodifica com precisão o arranjo Bayer e aplica os metadados fotográficos.
* **FFMPEG Support**: Limitado ou nulo. O FFmpeg pode identificar o contêiner como um TIFF genérico, mas falhará em ler a matriz de pixels brutos de 12 ou 14 bits para gerar o vídeo ou renderizar a imagem crua.
* **Rust alternative converters**:
* **`libraw-rs`**: Solução via FFI obrigatória para processar os dados lineares do sensor de maneira confiável e extrair as matrizes fotográficas.
* **`tiff` / `kamadak-exif**`: Soluções nativas ideais para ignorar os dados brutos massivos e ler rapidamente as tags estruturais e a pré-visualização embarcada.



## File structure

O formato `.mef` é mais um membro da família de formatos RAW desenhados estritamente sobre a arquitetura **TIFF (Tagged Image File Format)**, padrão predominante na época de seu desenvolvimento (meados de 2004).

* **TIFF Header**: Segue a convenção clássica de ordem de bytes (geralmente `II` para Little Endian) seguido da assinatura mágica de identificação do TIFF.
* **IFDs (Image File Directories)**: A estrutura divide logicamente os dados em pastas lógicas e acessíveis independentemente:
* **IFD Primária**: Guarda as tags EXIF, configurações do disparo (abertura, tempo de exposição, ISO) e metadados descritivos da câmera.
* **Thumbnail/Preview IFD**: Um diretório especificamente devotado a guardar uma versão previamente renderizada da imagem (geralmente em formato JPEG), garantindo pré-visualização em sistemas e monitores de câmera instantaneamente.
* **Raw Data IFD**: O diretório que armazena os dados brutos da matriz de filtro de cores (CFA) do sensor, acompanhados das tags que informam as dimensões exatas dessa matriz.


* **Mamiya MakerNotes**: Uma seção proprietária isolada que a câmera alimenta com perfis de cor, curvas de tom projetadas pela engenharia da Mamiya e matrizes detalhadas de balanço de branco.

## Strategy for Thumbnail Generation

Como o arquivo valida integralmente a especificação TIFF, extrair o thumbnail para uma futura conversão em WebP é uma tarefa determinística, rápida e de baixo custo de CPU em Rust.

1. **Parser Estrutural Rápido**: Empregue os crates `kamadak-exif` ou `tiff` para ler apenas o cabeçalho e localizar imediatamente a tabela de IFDs.
2. **Busca pela Miniatura Interna**: Varra as IFDs lendo a tag `Compression`. Encontre a IFD que apresente o valor `6` (que no padrão TIFF indica a presença de compressão JPEG embutida).
3. **Mapeamento do Bloco de Memória**: Resgate os dados contidos em `StripOffsets` (o byte exato de início da imagem dentro do binário) e `StripByteCounts` (o comprimento em bytes desse bloco de imagem).
4. **Extração Direta e Codificação**:
* Construa um slice numérico na memória partindo diretamente do arquivo referenciado em disco.
* Envie este buffer JPEG ao extrator do crate `image` (função `image::load_from_memory`).
* Dimensione a resolução para casar com os requisitos visuais do front-end.
* Processe e grave a saída diretamente utilizando o encoder `webp`.



## Strategy for Visualization

O processamento da malha RAW de médio formato visando a exibição final de alta fidelidade engloba as etapas padrão da ciência de imagem digital, necessitando do emprego da linguagem C++.

1. **Delegação via LibRaw (FFI)**: O carregamento profundo do `.mef` será feito instanciando o `libraw-rs`. Desenvolver o parser matemático para os dados compactados em 12-bits da Mamiya em Rust seria custoso e perigoso para a confiabilidade de produção sem acesso à documentação fechada da marca.
2. **Cálculo de Demosaicing**: Aplique a rotina de interpolação disponível no LibRaw. Ele converterá a grade de pixels de camada única (Bayer) para uma grade RGB plena, preenchendo as cores ausentes em cada ponto.
3. **Mapeamento de Transições de Cor**: O sensor CCD da Mamiya ZD entrega texturas fotográficas próprias (bastante reconhecidas em retratos de estúdio). O pipeline em Rust puxará o vetor numérico linear vindo do LibRaw, calculará os multiplicadores de balanço de branco registrados e lançará a matriz para adequação a espaços de cor calibrados (sRGB ou Display P3).
4. **Pipeline Acelerado por GPU**: Converta a matriz bruta devolvida pelo wrapper FFI para vetores otimizados de shader (f32 ou matrizes 16-bit float) e renderize o output final alocando uma textura nativa dentro da engine WGPU.

## Uncertain Points

* **Tipos de Empacotamento de Bits**: Certos arquivos `.mef` guardam dados na configuração sem perdas (*uncompressed*), mas embutem o bit-packing (onde valores de 12-bits são entrelaçados para caber em espaços matemáticos estreitos e economizar disco). Tentar transpor esses valores via parsers rústicos acarretará em imagens repletas de anomalias visuais (padrões magenta ou listras verticais); por isso a delegação ao LibRaw é inegociável para a imagem total.
* **Ruído Base Inerente do Sensor CCD**: A ausência do software de estúdio original da época (Mamiya Digital PhotoStudio) revela problemas fundamentais. Os perfis de denoising nativos do software proprietário não operam no LibRaw. Em 100% de zoom em zonas de sombra, o desenvolvedor notará grânulos e anomalias térmicas de CCD em aplicações open-source que o fotógrafo de 2004 não enxergava em seu monitor original.

## Other informations

* O `.mef` assumiu um status definitivo de formato histórico. Quando a Mamiya iniciou uma parceira profunda com a gigante do médio formato Phase One, a cadeia produtiva adotou formatos como `.mos` (Mamiya Operating System) em costas digitais Leaf, para eventualmente consolidar tudo em torno da bandeira corporativa dinamarquesa sob o formato `.iiq` (Intelligent Image Quality).
* Tratando-se de um sistema fotográfico de altíssimo valor aquisitivo e voltado para nichos de estúdio estritos (lançado primeiramente em 2004 com apenas 22 Megapixels), obter samples suficientes contendo diferentes ranges dinâmicos e de luz para formar uma base sólida de testes automatizados pode ser bastante moroso durante o ciclo de desenvolvimento do seu software.
