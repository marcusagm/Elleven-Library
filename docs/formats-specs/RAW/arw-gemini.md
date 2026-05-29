# Sony Alpha RAW (`.arw`) File Format Technical Specification

## Format Overview
- **Extension Name**: `.arw` (Sony Alpha Raw)
- **Possible Origin**: Sony Corporation (Utilizado nas linhas Alpha, NEX, Cyber-shot RX e câmeras de cinema FX).
- **Category**: Camera RAW Image.
- **LibRaw Support**: Excelente. Sendo a Sony uma das maiores fabricantes globais de sensores, o LibRaw possui suporte maduro e otimizado para as diversas gerações do formato ARW, incluindo compressão com perdas (lossy), sem perdas (lossless) e dados não comprimidos.
- **FFMPEG Support**: Limitado a nulo. O FFmpeg falhará em decodificar nativamente a malha RAW do sensor e as compressões proprietárias da Sony. Ele pode conseguir extrair o JPEG embutido caso interprete a casca TIFF corretamente, mas é ineficaz para revelar o negativo digital.
- **Rust alternative converters**:
  - **`libraw-rs`**: Solução via FFI obrigatória para acesso confiável aos dados brutos do sensor, cálculos de matriz de cor e decodificação das compressões complexas da Sony.
  - **`kamadak-exif` / `tiff`**: Crates nativos ideais para navegar rapidamente pela árvore de diretórios e extrair a pré-visualização (JPEG) sem invocar o processamento pesado do C/C++.

## File structure
O formato `.arw` é rigorosamente construído sobre o padrão **TIFF/EP (Tagged Image File Format for Electronic Photography)**. Sua estrutura é robusta, porém a Sony utiliza o padrão de forma bastante específica para alocar suas compressões e metadados.

1. **Header e Ordem de Bytes**: Inicia com o cabeçalho padrão TIFF (geralmente `II` para Little Endian nas câmeras Sony) seguido pelo Magic Number `0x002A` (42).
2. **IFD0 (Image File Directory 0)**: O diretório principal contém os metadados Exif padronizados (ISO, velocidade, abertura) e, quase sempre, um ponteiro para um thumbnail de baixa resolução.
3. **Sub-IFDs (Preview/JPEG)**: O ARW se destaca por incluir um arquivo JPEG frequentemente em **resolução total** e altíssima qualidade (Full-size JPEG) dentro de uma Sub-IFD ou em tags proprietárias da Sony na IFD primária.
4. **MakerNotes da Sony**: Bloco denso e frequentemente criptografado/ofuscado em partes, contendo perfis de lente, status do DRO (Dynamic Range Optimizer), estilos criativos e parâmetros de foco automático.
5. **Raw Data IFD**: O diretório final aponta para a matriz de pixels do sensor. A Sony utiliza três formas principais de gravação destes dados, o que impacta o decodificador:
   - **Uncompressed**: Dados lineares de 14-bit puramente matemáticos.
   - **Lossy Compressed**: Uma compressão agressiva e histórica da Sony usando um esquema de "11+7 bit delta". Reduz drasticamente o tamanho do arquivo, mas introduz perdas.
   - **Lossless Compressed**: Introduzida em câmeras modernas (A7R IV, A1), usa algoritmos complexos de compressão de dados reversível para manter os 14-bits intactos economizando espaço.

## Strategy for Thumbnail Generation
Como a Sony embute um JPEG de altíssima qualidade (muitas vezes em resolução total 4K ou superior), a extração nativa em Rust é extremamente eficiente e dispensará totalmente a leitura do RAW pesado.

1. **Parsing da Árvore TIFF**: Utilize o crate `kamadak-exif` para ler o cabeçalho e saltar para os diretórios. As câmeras da Sony geralmente mapeiam o JPEG principal em uma Sub-IFD.
2. **Localização do Offset**: Busque pelo diretório que possua a tag `Compression` com valor `6` (JPEG).
3. **Extração de Bytes na Memória**:
   - Capture os valores numéricos das tags `StripOffsets` e `StripByteCounts`.
   - Realize um slice do buffer do arquivo em disco referenciando esse offset exato.
4. **Decodificação e Re-encoding Rápido**:
   - Transfira esse slice `&[u8]` para o crate `image` (via `image::load_from_memory`).
   - Como o JPEG embutido no ARW costuma ser muito grande, aplique o redimensionamento usando um filtro de interpolação balanceado e salve o buffer resultante no formato `webp`. Esta operação consome fração de segundos e pouquíssima RAM.

## Strategy for Visualization
O processo de visualização da imagem crua com fidelidade total exige a reconstrução dos dados proprietários da Sony, o que torna o C++ (via LibRaw) indispensável.

1. **Descompressão via FFI**: O `libraw-rs` lidará com as complexidades matemáticas das compressões Lossless e Lossy Delta da Sony. Tentar reescrever o decodificador ARW Lossless em Rust do zero é um projeto massivo e sujeito a falhas, visto que a documentação oficial é fechada.
2. **Cálculo de Black Level Categórico**: Sensores Sony possuem níveis de preto intrincados que variam dependendo se a câmera está usando ISO base, amplificação de ganho secundário (Dual Native ISO) ou obturador eletrônico silencioso. O LibRaw abstrai essa matriz, subtraindo os valores corretos para evitar que as sombras ganhem tons roxos ou verdes.
3. **Debayering e Espaço de Cor**: O pipeline executa a interpolação da matriz Bayer para RGB, aplica os coeficientes de balanço de branco lidos no instante do clique e transpõe os dados via a matriz de cores específica da câmera para o espaço sRGB/Display P3.
4. **Renderização**: Recupere o array de bytes interpolados em `f32` ou `u16` gerado pelo FFI e passe-o diretamente para as texturas do WGPU no seu front-end para exibir o "negativo digital" limpo ao usuário.

## Uncertain Points
- **O Efeito "Star Eater" (Compressão Lossy)**: Nas versões mais antigas do formato ARW com compressão lossy, o algoritmo de redução de ruído espacial e compressão delta da Sony acaba confundindo estrelas isoladas no céu noturno com ruído de pixel quente, destruindo essa informação matematicamente (o famoso "Star Eater"). Visualizadores open-source exibirão esse defeito caso leiam o RAW diretamente, e não há correção de software, pois os dados foram omitidos na gravação.
- **Pixel Shift Multi Shooting**: Câmeras de alta resolução (A7R III em diante) podem disparar 4 ou 16 arquivos `.arw` sequenciais movendo o sensor em 1 pixel para capturar cores reais (dispensando debayering). O ARW final processado pelo usuário é na verdade uma fusão desses arquivos (geralmente gerado no software oficial Sony Imaging Edge e salvo como `.arw` ou `.arq`). A leitura individual de um RAW dessa sequência no LibRaw mostrará apenas 1/4 da exposição ou parecerá incorreta geometricamente sem o software de fusão adequado.

## Other informations
- A Sony tem o costume de armazenar parâmetros de correção de lentes embutidas (especialmente distorção em lentes grande-angulares E-Mount) diretamente nas MakerNotes. O software oficial da Sony e o Adobe Lightroom forçam a aplicação dessa correção no carregamento do RAW, mas o LibRaw por padrão entregará a imagem matematicamente crua (não corrigida). Isso significa que, ao renderizar seu RAW no seu software em Rust, a imagem pode aparecer consideravelmente mais arredondada (fisheye) ou ter vinheta mais escura nas pontas do que o Thumbnail JPEG embutido, que já passou pelo algoritmo de correção da câmera.
