# Adobe Photoshop (.psd) File Format Technical Specification

## 1. Visão Geral do Formato
*   **Nome da Extensão:** `.psd` (Photoshop Document).
*   **Possível Origem:** Desenvolvido pela Adobe Systems Inc.
*   **Categoria:** Documento de Imagem Raster Multicamada.
*   **Assinatura Mágica (Hexadecimal):** `38 42 50 53` (`8BPS`).
*   **Tamanho Típico Observado:** 1.6 MB a 120 MB nos exemplares (pode atingir gigabytes no formato `.psb`).
*   **Variações entre Arquivos Analisados:** Todos os arquivos analisados (exceto os samples base de baixa resolução) contêm blocos de recursos complexos incluindo thumbnails JPEG e metadados XMP.

## 2. Estrutura Binária Global

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 26 bytes | `Header` | **File Header** | Metadados básicos do documento. | Tamanho fixo. |
| Variável | 4 + N | `Block`  | **Color Mode Data** | Tabela de cores indexadas ou dados duotone. | Geralmente 0 para RGB/CMYK. |
| Variável | 4 + N | `Block`  | **Image Resources** | Metadados, previews, caminhos, etc. | Estrutura iterativa baseada em IDs. |
| Variável | 4 + N | `Block`  | **Layer & Mask Info** | Dados de todas as camadas e máscaras. | Frequentemente a maior seção. |
| Variável | 2 + N | `Data`   | **Image Data** | Imagem mesclada final (composite). | Ponto de visualização imediata. |

## 3. Header Principal
*   **Estrutura Detalhada:**
    *   `0x00`: Signature (4 bytes) - `8BPS`.
    *   `0x04`: Version (2 bytes) - `1` para PSD, `2` para PSB.
    *   `0x06`: Reserved (6 bytes) - Deve ser zero.
    *   `0x0C`: Channels (2 bytes) - Número de canais de cor (1-56).
    *   `0x0E`: Height (4 bytes) - Altura em pixels.
    *   `0x12`: Width (4 bytes) - Largura em pixels.
    *   `0x16`: Depth (2 bytes) - Bits por canal (1, 8, 16, 32).
    *   `0x18`: Color Mode (2 bytes) - Modo de cor (3 = RGB, 4 = CMYK, etc).
*   **Endianness:** Big-Endian.
*   **Flags/Checksums:** Não há checksums globais no header básico.

## 4. Estruturas Internas Identificadas

### 4.1. Image Resource Block (8BIM)
*   **Offset inicial:** Após a seção Color Mode Data.
*   **Tamanho:** Variável (definido no início da seção).
*   **Estrutura Interna:**
    *   Signature (4 bytes): `8BIM`.
    *   ID (2 bytes): Identificador do recurso (ex: 1036 para Thumbnail).
    *   Name (Pascal String): Nome do recurso (alinhado a 2 bytes).
    *   Size (4 bytes): Comprimento dos dados do recurso.
    *   Data (Variável): Payload do recurso (alinhado a 2 bytes).
*   **Função:** Repetido N vezes para armazenar thumbnails, perfis ICC, metadados XMP e guias.

## 5. Endianness
*   **Big-Endian:** Absolutamente todos os campos numéricos (inteiros de 16, 32 e 64 bits) seguem o formato de byte mais significativo primeiro.
*   **Evidência Encontrada:** O campo de versão `00 01` e resoluções observadas nos dados hexadecimais confirmam a ordem Adobe Standard (Big-Endian).

## 6. Compressão
*   **Indícios:** O campo inicial da seção *Image Data* indica o método.
*   **Algoritmos:**
    *   `0`: Raw (Sem compressão).
    *   `1`: RLE (PackBits).
    *   `2`: Zip sem predição.
    *   `3`: Zip com predição.
*   **Estratégia:** Para RLE, cada canal/linha deve ser descompactado sequencialmente de acordo com a tabela de comprimentos de linha.

## 7. Dados de Imagem (Merged Composite)
*   **Offset:** Localizado na seção final do arquivo.
*   **Format:** Planar (Canais separados). Se for RGB, armazena todos os pixels do canal R, seguidos por G, depois B.
*   **Bit Depth:** 8-bit é o mais comum, mas 16-bit e 32-bit (visto em HDR) são suportados.
*   **Reconstrução:** Intercalar os dados planares em um buffer RGBA/RGB para exibição.

## 8. Thumbnail / Preview Embutido
*   **Existe Preview?** Sim, altamente comum.
*   **Offset:** Dentro da seção Image Resources.
*   **ID do Recurso:** `1036` (ou `1033`).
*   **Formato:** JPEG encapsulado (KJpegRGB).
*   **Extração:** Localizar o recurso 1036, pular os 28 bytes de header fixo do thumbnail (dimensões e metadados internos) e extrair o stream JPEG que começa logo em seguida.

## 9. Metadados
*   **XMP Metadata:** Encontrado no Resource ID `1060`. XML em texto puro formatado pela Adobe.
*   **EXIF:** Frequentemente embutido nos metadados XMP ou em blocos de recursos específicos.
*   **Strings:** Encontrados nomes de camadas em UTF-8 (dentro da seção de camadas) e nomes de recursos em Pascal Strings.

## 10. Engenharia Reversa Estrutural
*   **Padrões Recorrentes:** Blocos com assinatura `8BIM` seguidos por comprimentos `Size`.
*   **TLV (Type-Length-Value):** Toda a arquitetura interna de recursos e camadas é baseada em TLV.
*   **Alinhamento:** Preenchimento de bytes (padding) é necessário para garantir que cada bloco comece em um offset par.

## 11. Estratégia para Implementação de Parser
1.  **Ordem:** Header -> Skip ColorMode -> Iterate Resources (Target 1036) -> Layer Metadata.
2.  **Validações:** Verificar se a assinatura do recurso é `8BIM`. Se falhar, o parser perdeu o alinhamento.
3.  **Tratamento de Erros:** Usar o comprimento total da seção para evitar leitura além dos limites em arquivos mal-formados.

## 12. Pseudocódigo de Parser
```pseudo
open file
read magic (4 bytes) -> must be "8BPS"
read version (2 bytes) -> 1=PSD, 2=PSB
skip reserved(6)
width, height, depth = read_header_dims()

skip color_mode_data_len

resource_section_len = read_u32()
end_resource_offset = current_pos + resource_section_len

while current_pos < end_resource_offset:
    sig = read(4) # Expect "8BIM"
    id = read_u16()
    name = read_pascal_string_aligned() 
    data_size = read_u32()
    
    if id == 1036:
        # Extrair Thumbnail
        skip(28) # Header do thumb
        jpeg_buffer = read(data_size - 28)
        save jpeg_buffer as "preview.jpg"
        break
    
    skip(data_size + padding)
```

## 13. Estratégia para Geração de Thumbnail
*   **Melhor Abordagem:** Usar o Resource 1036 (JPEG). É performático e reflete exatamente a intenção do artista ao salvar.
*   **Fallback:** Decodificar a imagem mesclada na seção final. Requer implementação de descompressão RLE ou Zip e recomposição de planos de canais.

## 14. Estratégia para Visualização Básica
*   Se o documento estiver em modo RGB, o thumbnail 1036 é um arquivo JPEG pronto.
*   Para visualização em alta fidelidade, renderizar a seção *Image Data* aplicando o modo de cor (RGB para RGB, CMYK para RGB via perfil ICC básico).

## 15. Mapa Comparativo Entre Arquivos
| Arquivo | Versão | Resolução | Canais | Recursos | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `SPC_8187.psd` | 1 | 4912x7360 | 4 | XMP + Thumb | Arquivo profissional pesado. |
| `sample.psd` | 1 | 758x960 | 4 | XMP + Thumb | Exemplo padrão. |
| `sample_640x426.psd`| 1 | 640x426 | 3 | ResolutionInfo | Sem thumbnail embutido. |

## 16. Pontos Incertos
*   **Alinhamento de Pascal Strings:** Alguns softwares de terceiros podem não alinhar corretamente o nome do recurso a 2 bytes (Confiança: 90%).
*   **Zlib Predictor:** O algoritmo de predição em modo Zip (tipo 3) pode variar levemente entre versões do Photoshop (Confiança: 85%).

## 17. Conclusão Técnica
O formato PSD é robusto e extensível, utilizando uma arquitetura modular de recursos. A extração de thumbnails é simples devido ao encapsulamento de streams JPEG padrão dentro de blocos identificáveis por IDs fixos, permitindo que ferramentas externas gerem visualizações rápidas sem processar toda a árvore de camadas.
