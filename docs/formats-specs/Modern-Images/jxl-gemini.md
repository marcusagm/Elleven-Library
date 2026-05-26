# JPEG XL (jxl) File Format Technical Specification

## Format Overview

* **Extension Name**: .jxl
* **Possible Origin**: Joint Photographic Experts Group (JPEG), padronizado entre 2021 e 2022 (ISO/IEC 18181).
* **Category**: Next-Generation Raster Image (Lossless, Lossy, HDR, Animação e Recompressão).
* **LibRaw Support**: Não aplicável / Não suportado. O JXL é um codec de entrega/compressão de imagem final. Embora fotógrafos usem JXL para exportar arquivos devido ao suporte HDR de 32-bit flutuante, ele não é um formato de dados brutos de sensor de câmera (Bayer/X-Trans), que é o foco do LibRaw.
* **FFMPEG Support**: Suportado via libjxl. Requer que a compilação do FFmpeg tenha sido feita com a flag --enable-libjxl. Frequentemente, até versões recentes do FFmpeg exigem a passagem da flag -strict experimental ou -strict unofficial na linha de comando para permitir operações com este formato.
* **Rust alternative converters**:
* **jxl-oxide**: Uma biblioteca recente e revolucionária, escrita em **100% Rust puro**. É um decodificador seguro e de altíssimo desempenho, dispensando totalmente o C/C++ para leitura.
* **jpegxl-rs**: Bindings (FFI) para a biblioteca de referência oficial em C++ (libjxl). Extremamente robusto e obrigatório se sua aplicação precisar *codificar* (salvar) imagens em JXL, já que os codificadores em Rust puro ainda não são maduros.



## File structure

O JXL possui uma arquitetura dupla; o arquivo em disco pode assumir duas formas:

1. **Bare Codestream**: Não possui contêiner estrutural. Começa diretamente com a assinatura mágica de 2 bytes 0xFF 0x0A (\xFF\x0A). Contém apenas o cabeçalho base da imagem e o fluxo de compressão direto.
2. **ISOBMFF Container (Box-based)**: Baseado no formato ISO de contêineres multimídia (mesma estrutura raiz do MP4, HEIF e AVIF).
* Começa com uma assinatura de 12 bytes: 0x00 0x00 0x00 0x0C 0x4A 0x58 0x4C 0x20 0x0D 0x0A 0x87 0x0A (identificando a caixa JXL ).
* Permite organizar metadados complexos em "Caixas" (Boxes) sem tocar nos dados da imagem:
* Caixa Exif: Metadados fotográficos convencionais.
* Caixa xml : Metadados XMP (Adobe).
* Caixa icc : Perfil de gerenciamento de cor ICC.
* Caixa jxlc ou jxlp: Onde os dados dos pixels residem de fato.





A compressão da imagem em si (dentro do codestream) é ramificada em dois algoritmos principais operando juntos:

* **VarDCT (Variable DCT)**: Baseado em blocos, desenhado para substituir fotografias, extremamente eficiente para imagens naturais e introduzindo perdas (lossy). Usa um espaço de cor interno perceptivo avançado chamado **XYB**.
* **Modular Mode**: Otimizado para arte digital, gráficos vetoriais rasterizados e imagens geradas por computador. Responsável pela compressão matemática 100% sem perdas (lossless).

## Strategy for Thumbnail Generation

Diferente de formatos RAW que embutem JPEGs legados, o JXL é tão eficiente que a "miniatura" muitas vezes é simplesmente extraída renderizando os primeiros estágios da própria imagem principal.

1. **Parser Puro em Rust**: Evite o FFI para a geração de miniaturas. Use o crate jxl-oxide. Ele é seguro em memória e extremamente rápido em arquiteturas modernas.
2. **Extração Progressiva (A Grande Vantagem)**: Imagens JXL (especialmente VarDCT) são inerentemente progressivas. O jxl-oxide permite decodificar apenas as frequências espaciais mais baixas da imagem (conhecidas como blocos **DC**, com proporção de 1:8 do tamanho real) ignorando o resto do arquivo. Você obtém uma miniatura diretamente do fluxo de bits base sem alocar a imagem inteira em RAM e sem usar processamento para redução matemática.
3. **Conversão e Empacotamento**: A imagem em escala reduzida resultante (decodificada em sRGB 8-bit pelo próprio decoder) é alocada num buffer linear [u8] e repassada ao encoder do webp para ser gerada em disco/cache.

## Strategy for Visualization

O JXL foi criado para as telas de última geração. O foco visual não é apenas resolução, mas **HDR** e **Wide Color Gamut (WCG)**.

1. **Decodificação de Alta Fidelidade**: Na hora da visualização principal, você utilizará o jxl-oxide ou jpegxl-rs para decodificar os pixels em buffers de **ponto flutuante (32-bit f32 por canal)**, em vez de u8. O JXL costuma reter ranges dinâmicos gigantescos.
2. **Transformação XYB**: Na decodificação, o decoder fará o cálculo do espaço de cor interno perceptivo XYB para transformar a matriz de volta para um espaço de cor linear RGB clássico baseado no perfil exigido pela sua tela.
3. **Gerenciamento de Perfis ICC**: Se o arquivo JXL utilizar o contêiner ISOBMFF, ele pode possuir um perfil ICC embutido em vez das cores matemáticas padrão. Você deverá extrair a Box icc , passar para um crate como o lcms2 (Little CMS) e cruzar os dados da imagem com o perfil ICC do monitor do sistema operacional.
4. **Renderização Acelerada por GPU**: Para uma fidelidade pura e sem travamentos ao dar zoom ou aplicar "pan" na imagem, passe o array de floats RGB direto para texturas em wgpu e deixe os shaders tratarem a conversão linear->SDR (Tone Mapping) caso a tela do usuário não seja HDR.

## Uncertain Points

* **Encode em Rust Puro**: A biblioteca jxl-oxide escreve um capítulo perfeito para a *leitura* no ecossistema Rust. No entanto, criar ou salvar um arquivo .jxl com alta compressão puramente em Rust (sem depender das bibliotecas C++) ainda não é possível em nível de produção. Toda operação de saída para JXL precisará carregar a dependência complexa do FFI da libjxl.
* **Animações (Arquivos Multiframe)**: O formato suporta animações de maneira superior ao GIF/WebP. Ao implementar a estratégia de extração de miniatura, seu parser precisará verificar o header. Se a tag de animação estiver ativa, decodificar de forma leviana pode resultar na captura acidental de um *frame delta* opaco, extraindo um thumbnail quebrado em vez do primeiro quadro-chave legível.

## Other informations

* **Superpoder de Transição (Legacy JPEG Recompression)**: A ferramenta cjxl (e as APIs internas) possuem um modo especial para ingerir um .jpg antigo de 20 anos atrás e empacotá-lo em um contêiner .jxl resultando num arquivo até 20% menor **sem nenhuma alteração de pixel**. Posteriormente, ele pode reconstruir o .jpg exato (mesmo hash). Se a sua aplicação lidar com backups de usuários, esse recurso é incrível, e a leitura desse "JPEG transcodificado" interno é muito veloz.
* **Ecossistema e Disputas de Mercado**: A adoção de JXL é polêmica em aplicações web. O Google removeu o código de decodificação JXL do Chromium justificando pouca adesão, enquanto a Apple integrou suporte nativo maciço em 2023 no macOS, iOS, Finder e Safari (através de extensões do framework ImageIO), tornando-o excelente para aplicações desktop focadas no ecossistema Apple, mas desafiador se seu projeto visar visualização direta em Chromium/Electron.
