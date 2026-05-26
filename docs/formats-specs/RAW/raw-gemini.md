# Leica / Panasonic RAW (`.raw`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.raw` (Frequentemente substituído posteriormente por `.rwl` na Leica e `.rw2` na Panasonic).
* **Possible Origin**: Desenvolvido em parceria entre Leica Camera AG e Panasonic (utilizado em modelos clássicos resultantes dessa colaboração, como a Leica Digilux 2 / Panasonic Lumix DMC-LC1 e as primeiras gerações da série D-Lux).
* **Category**: Camera RAW Image.
* **LibRaw Support**: Suportado. O LibRaw possui as rotinas necessárias para decodificar o mosaico do sensor e interpretar os metadados fotográficos embutidos pela Leica/Panasonic.
* **FFMPEG Support**: Limitado a nulo. O FFmpeg falhará em decodificar os dados brutos da malha Bayer, podendo no máximo enxergar superficialmente se a estrutura TIFF estiver padronizada, mas sem capacidade de gerar saída da imagem real.
* **Rust alternative converters**:
* **`libraw-rs`**: Solução essencial via FFI para o cálculo matemático da matriz do sensor e interpretação das cores e MakerNotes exclusivas.
* **`kamadak-exif` / `tiff**`: Bibliotecas nativas muito úteis para varrer o cabeçalho do arquivo e extrair apenas a miniatura embarcada sem processamento pesado.



## File structure

O formato `.raw` original da Leica/Panasonic é uma ponte de transição tecnológica. Ele é estruturalmente **baseado em TIFF (Tagged Image File Format)**, mas possui idiossincrasias significativas em suas primeiras versões antes da indústria padronizar os RAWs.

1. **Header (Assinatura)**: Modelos iniciais podem não seguir estritamente o "Magic Number" `42` do TIFF convencional ou usar um identificador levemente alterado de ordem de bytes (`II` ou `MM`), o que quebra alguns parsers genéricos rigorosos.
2. **IFDs (Image File Directories)**: A estrutura divide os dados da mesma forma que o TIFF:
* **IFD Principal**: Metadados (Exif, abertura, exposição).
* **MakerNotes (O Coração da Leica)**: Bloco de dados proprietários. Em arquivos gerados por câmeras Leica, esse bloco frequentemente inicia com a string ASCII `LEICA` seguida de bytes nulos (`0x00 0x00 0x00`). Ele guarda parâmetros de calibração específicos da lente Leica DC Vario-Summicron acoplada à câmera.
* **Preview/Thumbnail IFD**: Um diretório que guarda uma versão JPEG processada pela própria câmera (variando em resolução dependendo da geração da câmera).


3. **Raw Data IFD**: O diretório que aponta para os dados do sensor bruto (frequentemente 12-bit nos modelos clássicos como a Digilux 2). Em seus primeiros dias, esses dados muitas vezes não eram comprimidos, gerando arquivos grandes (aprox. 10MB para meros 5 Megapixels).

## Strategy for Thumbnail Generation

O objetivo para a geração de thumbnails é isolar o JPEG embutido sem invocar o pesado processo de debayering (que processaria a malha de 12-bit).

1. **Leitura Flexível de IFD**: Utilize `kamadak-exif` em Rust. Como os primeiros `.raw` podem ter cabeçalhos levemente fora do padrão rigoroso, garanta que seu parser ignore erros de conformidade estrita e pule para a tabela de offsets.
2. **Localização do JPEG**: Identifique o diretório que possua a tag indicando compressão (geralmente valor `6` para JPEG).
3. **Mapeamento de Bytes**: Faça a leitura das tags referentes ao offset (`JpegIFOffset` ou `StripOffsets`) e ao tamanho (`JpegIFByteCount` ou `StripByteCounts`).
4. **Extração e Conversão**:
* Com o valor do offset, copie aquele exato bloco de bytes do arquivo original para a memória.
* Entregue o slice de bytes (`&[u8]`) para o crate `image` via `image::load_from_memory`.
* Redimensione usando um algoritmo eficiente (como Triangle ou Lanczos3, dependendo da necessidade de qualidade vs velocidade).
* Realize o encoding final e salve o buffer resultante como `webp`.



## Strategy for Visualization

O `.raw` é efetivamente o "negativo digital", portanto exibir a imagem real requer reinterpretar a luz capturada pelo sensor primário.

1. **Processamento Obrigatório via FFI**: Não tente escrever o demosaicing (debayering) desses arquivos em Rust puro. Modelos clássicos usam arranjos ou métodos de padding de bits não documentados para os dados de 12-bit. Utilize `libraw-rs`.
2. **Demosaicing**: O LibRaw vai pegar os dados brutos de um único canal (Bayer) e aplicar algoritmos de interpolação (como VNG ou AHD) para recriar os canais ausentes (RGB completos para cada pixel).
3. **Balanço de Branco e Colorimetria**:
* Extraia os multiplicadores de balanço de branco aplicados no disparo.
* O LibRaw vai cruzar esses metadados com as MakerNotes (se disponíveis) para garantir que as cores da imagem linear se aproximem das decisões do firmware original.


4. **Pipeline WGPU/Interface**: O `libraw-rs` retornará um array linear de pixels (ex: `f32` ou `u16`). Você passará esse array para o seu renderizador gráfico para aplicar a curva Gamma (sRGB) e jogar na tela do usuário, mantendo fidelidade sem os artefatos de compressão da época.

## Uncertain Points

* **Fragmentação do Cabeçalho TIFF**: Sendo um formato pioneiro (início dos anos 2000), o nível de adesão à estrutura TIFF variava *por firmware*. Você pode encontrar arquivos `.raw` da mesma câmera onde bibliotecas puras em Rust falharão em ler a árvore de diretórios por divergências em bytes de preenchimento (padding). O tratamento de exceções (fallbacks) na etapa de thumbnail é vital.
* **Distorção e Correção de Lente Oculta**: As câmeras Leica/Panasonic dessa era possuíam lentes fixas complexas e aplicavam correções geométricas pesadas *no firmware* para gerar o JPEG. Softwares open-source lendo o dado bruto do `.raw` frequentemente exibem imagens com fortes distorções de barril (barrel distortion), já que as rotinas de correção proprietárias não são transferidas abertamente para o LibRaw. O usuário pode achar o RAW decodificado mais "feio" ou distorcido do que o thumbnail.

## Other informations

* **Histórico de Padronização**: Pouco tempo após a vida útil desses modelos clássicos em `.raw`, a Leica percebeu o problema da fragmentação. Ela se tornou a primeira grande fabricante a adotar integral e nativamente o padrão Adobe **DNG (Digital Negative)** como seu formato RAW oficial (a partir da Leica M8 e modelos subsequentes). Já a Panasonic decidiu manter seu formato proprietário e evoluiu o `.raw` para o `.rw2`.
* **Micro-contraste e Moiré**: Os sensores antigos que usavam `.raw` muitas vezes tinham filtros anti-aliasing fracos (ou ausentes) para maximizar a nitidez da lente Leica. Ao decodificar via LibRaw, é muito comum que padrões repetitivos na imagem exibam fortes aberrações cromáticas (Moiré) que precisariam de tratamento específico de redução matemática em nível de software, algo que a câmera mascarava internamente ao gravar seus arquivos JPEG originais.
