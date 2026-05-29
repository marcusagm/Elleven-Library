# AV1 Image File Format (`.avif`) File Format Technical Specification

## Format Overview
- **Extension Name**: `.avif`
- **Possible Origin**: Alliance for Open Media (AOMedia), padronizado em 2019.
- **Category**: Next-Generation Raster Image / Image Sequence (Lossy, Lossless, HDR, Animação).
- **LibRaw Support**: Não suportado / Não aplicável. O AVIF é um formato final de entrega e compressão (baseado no codec de vídeo AV1), não um formato RAW de dados brutos de sensor fotográfico (CFA/Bayer).
- **FFMPEG Support**: Totalmente suportado. O FFmpeg decodifica e codifica AVIF de forma nativa e extremamente robusta, utilizando bibliotecas por baixo dos panos como `libaom`, `libsvtav1`, `librav1e` e o decodificador ultra-rápido `dav1d`.
- **Rust alternative converters**:
  - **`libavif-sys` / `dav1d-rs`**: Wrappers (FFI) para as bibliotecas em C. São as opções mais maduras e com melhor desempenho para leitura (decodificação) em produção.
  - **`image`**: A biblioteca padrão de imagens do ecossistema Rust possui suporte ao AVIF utilizando dependências como `rav1e` (para escrita) ou interfaces C para leitura.
  - **`zune-avif`**: Uma iniciativa recente para decodificação em Rust puro, ideal para segurança de memória, mas que ainda pode não cobrir 100% das edge-cases e animações do padrão AV1 se comparado ao `dav1d`.

## File structure
O AVIF não é um formato construído do zero; ele é, na verdade, um arquivo de vídeo disfarçado. Estruturalmente, ele utiliza o formato de contêiner **ISOBMFF (ISO Base Media File Format)**, especificamente herdando as características do padrão HEIF.

1. **Boxes (Caixas/Átomos)**: O arquivo é uma coleção de blocos lógicos.
   - **`ftyp` (File Type)**: Fica no início absoluto do arquivo. Identifica a "marca" (brand) como `avif` ou `avis` (para sequências animadas).
   - **`meta` (Metadata)**: Contém a estrutura da imagem. Aqui ficam os perfis ICC, metadados Exif/XMP (em caixas específicas `uuid` ou `Exif`) e as propriedades da imagem (`iprp`).
   - **`iloc` (Item Location)**: Funciona como um índice. Diz exatamente em quais bytes do arquivo (offset e tamanho) os dados comprimidos da imagem começam e terminam.
   - **`mdat` (Media Data)**: O grande bloco binário que guarda o "payload". No caso do AVIF, esse payload é simplesmente um *Intra-frame* (Keyframe) compactado com o codec de vídeo AV1.
2. **Separação de Canais (Alpha)**: Diferente do PNG, o AVIF frequentemente lida com a transparência (Alpha) criando um segundo "item" de imagem dentro do mesmo arquivo: um frame AV1 monocromático que funciona exclusivamente como máscara de opacidade, referenciado através do bloco `auxl` (Auxiliary Image).

## Strategy for Thumbnail Generation
O padrão ISOBMFF permite extrema eficiência na extração de miniaturas se o criador do arquivo tiver embutido uma.

1. **Busca por Miniatura Embutida (`thmb`)**:
   - Utilize um parser de ISOBMFF em Rust (como `mp4` ou extensões do `image`) para ler a árvore de caixas.
   - Verifique a caixa de referências (`iref`). Se existir uma referência do tipo `thmb` apontando para um ID de imagem secundário, significa que há um thumbnail embutido.
   - Use a caixa `iloc` para pegar os bytes exatos desse thumbnail, passe pelo decodificador AV1, e encode para WebP. Isso é incrivelmente rápido.
2. **Decodificação de Base (Fallback)**:
   - Diferente do JPEG XL (`.jxl`), o AVIF não possui um modo progressivo de decodificação espacial (onde você lê apenas o início do arquivo e tem uma miniatura). Se não houver miniatura embutida no contêiner, você será obrigado a decodificar o frame AV1 principal.
   - Utilize o `dav1d` (via FFI) por ser o decodificador AV1 mais rápido do mundo (altamente otimizado com Assembly).
   - Extraia o buffer, redimensione para as dimensões desejadas usando um algoritmo de interpolação no crate `image`, e exporte a matriz para `webp`.

## Strategy for Visualization
O AVIF foi projetado para as telas modernas, sendo mestre em **HDR (High Dynamic Range)** e **Wide Color Gamut (WCG)**.

1. **Decodificação do Payload AV1**: A interface (FFI) do `dav1d` ou `libavif` vai receber os bytes do bloco `mdat` e cuspir um buffer de pixels.
2. **Conversão de YUV para RGB**: Quase todos os AVIFs armazenam as cores não em RGB, mas em subamostragens YUV (como YUV420, YUV422 ou YUV444). Sua engine em Rust precisará ler a caixa de propriedades de cor (`colr` -> `nclx`) para saber qual matriz de conversão usar (ex: BT.709, BT.2020) e calcular os valores de RGB reais. O `libavif` possui funções embutidas que já fazem essa conversão YUV->RGB antes de entregar ao Rust.
3. **Gerenciamento HDR e 10-bit/12-bit**: 
   - O arquivo pode conter profundidade de 10-bit ou 12-bit.
   - Leia a função de transferência (Transfer Function) especificada. Se for PQ (Perceptual Quantizer - SMPTE ST 2084) ou HLG (Hybrid Log-Gamma), o arquivo é HDR.
   - Para exibição perfeita em monitores HDR no Rust, preserve o buffer em `f16` ou `f32` (floats lineares) após aplicar as curvas, envie este array para uma textura WGPU (`TextureFormat::Rgba16Float`), e avise a Swapchain do WGPU que o espaço de cor da janela é HDR10.
4. **Perfil ICC**: Se houver um perfil ICC explícito na caixa `colr`, ele sobrescreve os parâmetros `nclx`. Você deverá usar o crate `lcms2` para transpor as cores lidas para o perfil do monitor do usuário.

## Uncertain Points
- **Animações (Animated AVIF)**: AVIF suporta imagens animadas (similar a GIFs, mas milhares de vezes menores). Na leitura em Rust, é crucial inspecionar o arquivo para checar se existe um *Track* de vídeo em vez de um único *Item* de imagem. Decodificar uma animação como se fosse uma foto estática extrairá apenas o primeiro quadro (o que é ótimo para o Thumbnail, mas falhará na visualização completa).
- **O Gargalo do Pure Rust**: Implementar um player AVIF 100% nativo em Rust (sem bibliotecas C) em nível de produção é atualmente perigoso e incerto. O codec AV1 é absurdamente complexo, com centenas de ferramentas de predição espacial e filtros de loop. Você ficará dependente do FFI do `dav1d` para obter o desempenho esperado por um usuário moderno, complicando levemente o cross-compilation da sua aplicação.

## Other informations
- O AVIF alcança perfeitamente a premissa de "Lossless" (sem perdas), mas sua adoção massiva ocorreu no ambiente web devido ao seu poder em "Lossy". Em taxas de bits extremamente baixas (bitrates), onde o JPEG vira um mosaico de blocos e o WebP perde nitidez, o AVIF mantém as bordas afiadas aplicando um leve desfoque nas texturas (grazing).
- O custo de **codificar/salvar** (encode) um arquivo AVIF em alta qualidade (ex: convertendo um de seus RAWs para AVIF para salvar no disco) é elevadíssimo em termos de CPU, podendo demorar vários segundos para uma única foto em alta resolução, diferentemente da decodificação, que ocorre na casa dos milissegundos.
