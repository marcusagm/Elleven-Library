# Panasonic Lumix RAW (`.rw2`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.rw2`
* **Possible Origin**: Panasonic Corporation (Substituiu o antigo formato `.raw` em toda a linha Lumix, especialmente associado ao consórcio Micro Quatro Terços e câmeras Full-Frame da linha S).
* **Category**: Camera RAW Image.
* **LibRaw Support**: Suportado. O formato é de amplo conhecimento e possui excelente integração no LibRaw, que decodifica a compressão de 12-bit/14-bit e extrai os complexos parâmetros de lente da Panasonic.
* **FFMPEG Support**: Limitado a nulo. O FFmpeg falhará em realizar o demosaicing (debayering) da matriz proprietária, não sendo capaz de extrair o frame RAW fotográfico com fidelidade de cores e geometria adequadas.
* **Rust alternative converters**:
* **`libraw-rs`**: Solução via FFI estritamente necessária para a reconstrução dos dados do sensor e a interpretação correta do *black level* e metadados de lente.
* **`kamadak-exif` / `tiff**`: Crates nativos em Rust perfeitos para ler o cabeçalho e extrair o arquivo JPEG embutido sem invocar rotinas pesadas em C/C++.



## File structure

O formato `.rw2` segue a espinha dorsal de um arquivo **TIFF (Tagged Image File Format)**, mas introduz blocos de dados proprietários que quebram parsers TIFF extremamente rígidos se não forem tratados com flexibilidade.

1. **Header TIFF**: O arquivo inicia com a ordem de bytes (geralmente `II` para Little Endian) seguido de uma variação da assinatura mágica (`0x0055` em vez do tradicional `0x002A` em muitas versões do RW2, o que é um ponto de atenção crucial para o seu parser em Rust).
2. **IFD0 (Metadados Básicos)**: O primeiro diretório contém as tags Exif padrão (Exposição, ISO, Modelo da Câmera).
3. **Panasonic MakerNotes**: Diferente de outras fabricantes que aninham metadados simples, a Panasonic utiliza a seção de MakerNotes de forma intensiva. É aqui que residem os opcodes de correção de lente, perfil de cor da câmera e, em muitos modelos, o próprio ponteiro para a miniatura embutida.
4. **Preview Block (Thumbnail/JpgFromRaw)**: O arquivo embute um JPEG processado pela câmera. Dependendo do modelo, ele pode estar em uma sub-IFD padrão ou diretamente mapeado por uma tag proprietária da Panasonic (como a tag `0x002E`).
5. **Raw Sensor Data**: A malha de pixels do sensor (Bayer). Nos modelos Lumix mais recentes, estes dados são armazenados em 12-bit ou 14-bit e frequentemente utilizam uma compressão sem perdas (lossless) proprietária baseada em algoritmos DPCM/Huffman.

## Strategy for Thumbnail Generation

O `.rw2` foi projetado para que o JPEG embutido seja acessado rapidamente. A extração em Rust deve focar em ler os offsets corretos sem tocar no RAW comprimido.

1. **Parser Flexível (Cuidado com o Header)**: Se você utilizar o crate `tiff`, ele pode falhar logo no primeiro byte ao não encontrar o magic number padrão. Pode ser necessário escrever um pequeno código em Rust puro que leia o cabeçalho, pule a checagem do magic number (se for `0x0055`) e instancie o leitor de IFDs manualmente, ou usar o `kamadak-exif` configurado para tolerar falhas de padrão.
2. **Busca pelo Offset do JPEG**: Em vez de procurar apenas pelo padrão `Compression == 6`, o seu parser deve procurar pela tag proprietária da Panasonic responsável pelo JPEG (frequentemente `JpgFromRaw` na MakerNote ou a sub-IFD correta com tag de offset).
3. **Extração de Memória e Cópia**:
* Faça a leitura do tamanho (Length) e da posição de início (Offset) deste bloco JPEG.
* Construa um slice numérico `&[u8]` diretamente do arquivo em disco (`mmap` é excelente aqui).
* Passe este slice para `image::load_from_memory`.
* Redimensione para as necessidades da sua aplicação e exporte a imagem rapidamente utilizando o encoder `webp`.



## Strategy for Visualization

Para a visualização do negativo digital real (RAW), o ecossistema Micro Quatro Terços (e a linha Lumix Full-Frame) possui uma filosofia de "correção por software" que sua aplicação precisa tratar via LibRaw.

1. **Delegação Completa (FFI)**: O carregamento do arquivo e a descompressão do DPCM da Panasonic devem ser realizados exclusivamente pelo `libraw-rs`.
2. **Cálculo de Black Level**: A Panasonic tem um comportamento muito específico com o nível de preto (Black Level) do sensor, que pode variar dependendo do ISO e do modelo (especialmente sensores dual-gain da linha GH). O LibRaw subtrairá esse nível de preto matematicamente para que as sombras não fiquem arroxeadas ou leitosas.
3. **Demosaicing e Color Matrix**: O LibRaw vai interpolar a matriz Bayer para RGB e aplicar a matriz de calibração de cores para transpor os tons lineares do sensor para um espaço de cor padronizado como o sRGB. Transfira esse buffer linear de ponto flutuante (`f32`) de volta para o Rust.
4. **WGPU e Renderização**: Lance esse buffer para uma textura gráfica em Rust para exibir na tela do usuário, aplicando a curva de Gamma necessária.

## Uncertain Points

* **Correção Geométrica de Lente Oculta**: As lentes Micro Quatro Terços da Panasonic (e Olympus) são desenhadas com forte distorção de barril (barrel distortion) ótica. Essa distorção é corrigida 100% via software pelo motor da câmera para o JPEG. As instruções (Opcodes) para corrigir o RAW estão dentro das MakerNotes do `.rw2`. O LibRaw nem sempre aplica essa deformação matemática (warp) automaticamente na matriz decodificada. Isso significa que, em seu software, ao visualizar a imagem RAW a 100%, o usuário poderá ver a imagem com as bordas arredondadas e distorcidas (Fisheye leve), o que difere drasticamente do Thumbnail perfeitamente plano extraído antes.
* **Relações de Aspecto Multi-Aspect**: Câmeras como a Lumix LX100 e a GH5S possuem sensores "multi-aspecto" (o sensor é maior que a área efetiva da lente para manter a resolução em 4:3, 3:2 e 16:9). A leitura incorreta das tags de área de corte (Crop Area) no `.rw2` pode resultar na renderização de pixels mortos e bordas pretas, já que o sensor físico realocará dimensões inesperadas se o crop não for respeitado.

## Other informations

* O formato `.rw2` é altamente associado ao mercado híbrido (fotógrafos e videomakers). Devido ao foco massivo da Panasonic em vídeo, o controle de ruído de cor (Chroma Noise) e o tratamento de perfis V-Log que podem influenciar a forma como os RAWs fotográficos são expostos tornam esse formato desafiador em termos de reprodução de perfil exato comparado ao Adobe Lightroom.
* Como a Panasonic (através da Lumix) tem um acordo formal com a Leica (L-Mount Alliance), muitos dos algoritmos, estruturas de metadados e abordagens de empacotamento no `.rw2` são quase idênticos aos dos formatos RAW proprietários recentes da própria Leica. Caso você venha a dar suporte à atual linha Leica (SL, Q), perceberá que grande parte da sua implementação para o `.rw2` funcionará imediatamente.
