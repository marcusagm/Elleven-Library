# Hasselblad RAW (`.3fr`) File Format Technical Specification

## Format Overview

* **Extension Name**: `.3fr`
* **Possible Origin**: Hasselblad (Desenvolvido para suas costas digitais e câmeras de médio formato).
* **Category**: Camera RAW Image.
* **LibRaw Support**: Suportado. O LibRaw consegue ler os metadados de cor base e realizar o debayering das pesadas matrizes geradas pelos sensores Hasselblad.
* **FFMPEG Support**: Extremamente limitado e não confiável. O FFmpeg falhará em decodificar os dados reais do sensor devido ao peso e à compressão proprietária, podendo no máximo extrair a miniatura caso consiga interpretar o contêiner superficialmente.
* **Rust alternative converters**: `libraw-rs` para decodificação total do RAW. Crates de parsing puro como `kamadak-exif` ou `tiff` para navegar pela estrutura e extrair as miniaturas embarcadas sem invocar dependências C/C++.

## File structure

* **Contêiner TIFF:** O formato `.3fr` é estritamente baseado no padrão TIFF (Tagged Image File Format). O arquivo inicia com a assinatura de ordem de bytes e aponta diretamente para o mapeamento de diretórios (IFDs).
* **Múltiplas IFDs (Image File Directories):** A estrutura organiza os dados da imagem em diferentes pastas internas. A primeira IFD geralmente detém os metadados fotográficos em formato EXIF e parâmetros de disparo.
* **Miniaturas Embutidas:** Sub-IFDs específicas são utilizadas para armazenar JPEGs já processados pela câmera, garantindo que o fotógrafo possa pré-visualizar a imagem no display LCD ou em gerenciadores de arquivos sem realizar o cálculo matemático do RAW.
* **Dados Brutos (Raw Sensor Data):** Uma IFD designada guarda a matriz de filtro de cores (Bayer) do sensor de médio formato. Esses dados frequentemente recebem compressão sem perdas (lossless) para otimizar o gargalo de gravação no cartão de memória, já que lidam com resoluções de 50 a 100+ megapixels.
* **MakerNotes e Perfis:** A área proprietária armazena os dados cruciais de calibração de cor, calibração do sensor e as variáveis necessárias para corrigir características da lente Hasselblad no momento da captura.

## Strategy for Thumbnail Generation

* **Parsing Estrutural Rápido:** Utilize o crate `tiff` ou `kamadak-exif` para realizar a leitura apenas do cabeçalho do arquivo e navegar diretamente pelos diretórios IFD, ignorando a leitura dos volumosos dados RAW.
* **Localização do Preview:** Escaneie as IFDs em busca do diretório cuja tag de compressão (Compression Tag) possua o identificador de JPEG (geralmente o valor `6`).
* **Mapeamento e Extração de Bytes:** Capture as tags `StripOffsets` e `StripByteCounts` dessa IFD para descobrir a exata posição e tamanho do JPEG embutido. Recorte esse slice de bytes diretamente para a memória.
* **Decodificação e Re-encoding:** Repasse este slice de bytes para a função de carregamento em memória do crate `image`. Redimensione a matriz de pixels conforme o padrão do seu sistema e exporte diretamente para o formato `webp`. Esta operação completa contorna processamentos pesados e executa quase instantaneamente.

## Strategy for Visualization

* **Uso Obrigatório de FFI:** Evite tentar reconstruir a imagem do zero em Rust puro. A decodificação da compressão `.3fr` e o gerenciamento de cor exigem o repasse do processamento inicial para o `libraw-rs`.
* **Demosaicing Intensivo:** Como as câmeras Hasselblad possuem sensores de médio formato (maiores que Full Frame), as matrizes RAW geradas têm dimensões colossais. A etapa de interpolação (debayering) alocará bastante memória RAM. A aplicação em Rust precisa garantir que a memória seja despachada e liberada adequadamente após a renderização.
* **Gerenciamento de Matrizes de Cor:** O LibRaw deverá extrair os multiplicadores de balanço de branco do arquivo para corrigir o ponto de neutralidade. O pipeline de exibição precisará aplicar a matriz de calibração da câmera para transpor os valores lineares lidos do sensor para o espaço de cor exigido pela interface de usuário (como sRGB).

## Uncertain Points

* **Hasselblad Natural Colour Solution (HNCS):** O maior diferencial dos arquivos `.3fr` não é a estrutura do arquivo em si, mas a interpretação de cores da marca. Ferramentas open-source aplicarão uma conversão padrão, o que gera uma imagem plana e tecnicamente correta. Ela quase nunca refletirá o contraste perfeito e as transições suaves de cor alcançadas ao processar a mesma imagem pelo software oficial e fechado da marca (Hasselblad Phocus).
* **Correções Ópticas Proprietárias:** O arquivo armazena instruções detalhadas sobre vinheta e correção de aberrações da lente utilizada. O decodificador open-source pode não aplicar esses vetores de correção matemática automaticamente no arquivo aberto.

## Other informations

* **Formato de Transição:** Para usuários do ecossistema Hasselblad, o `.3fr` é conhecido puramente como o formato de gravação *in-camera*. Quando transferido para a estação de trabalho, o software oficial geralmente desembalará e converterá este arquivo em um arquivo `.fff` (Flexible File Format). O `.fff` retém os mesmos dados RAW do sensor, porém é um contêiner otimizado que suporta a injeção de edições não-destrutivas e histórico de processamento. Ao implementar um leitor para seu software, se suportar o `.3fr`, muito provavelmente a mesma base estrutural TIFF funcionará integralmente para ler também os arquivos `.fff`.
