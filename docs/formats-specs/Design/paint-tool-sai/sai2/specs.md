# Technical Analysis: PaintTool SAI v2 (.sai2)

## 1. Visão Geral do Formato
*   **Nome da Extensão:** `.sai2`
*   **Possível Origem:** SYSTEMAX PaintTool SAI (Versão 2).
*   **Categoria:** Documento de Gráficos Raster Multicamada.
*   **Assinatura Mágica (Hexadecimal):** `53 41 49 2D 43 41 4E 56 41 53` (`SAI-CANVAS`).
*   **Tamanho Típico Observado:** 1.6 MB a 200 MB (em camadas e alta resolução).
*   **Variações entre Arquivos Analisados:** O cabeçalho pode variar entre versões (ex: `SAI-CANVAS-TYPE0`), e o campo de contagem de chunks pode ser zero em versões recentes, exigindo escaneamento de tags.

## 2. Estrutura Binária Global

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 64 bytes | `Header` | **File Header** | Identificação e metadados globais. | Tamanho fixo. |
| `0x40` | N * 16 bytes | `List` | **Chunk List** | Tabela de descritores de blocos. | N pode ser variável. |
| Variável | Variável | `Block` | **Chunk Data** | Dados brutos dos chunks sequenciados. | Alinhamento variável. |

## 3. Header Principal
*   **Estrutura Detalhada:**
    *   `0x00`: Magic (10 bytes) - `SAI-CANVAS`.
    *   `0x0A`: Type Suffix (6 bytes) - Ex: `-TYPE0`.
    *   `0x10`: Unknown (4 bytes) - Frequentemente `0x00004000`.
    *   `0x14`: Chunk Count Alt (4 bytes) - Pode conter o número real de chunks se o campo oficial for zero.
    *   `0x20`: Canvas Width (4 bytes) - Largura em pixels (u32 LE).
    *   `0x24`: Canvas Height (4 bytes) - Altura em pixels (u32 LE).
    *   `0x28`: Chunk Count (4 bytes) - Campo oficial para número de chunks (u32 LE).
*   **Endianness:** Little-Endian.

## 4. Estruturas Internas Identificadas

### 4.1. Chunk Descriptor (16 bytes)
*   **Tag (4 bytes):** Identificador ASCII (ex: `thum`, `view`, `layr`).
*   **ID/Flags (4 bytes):** Identificador único do bloco ou flags de estado.
*   **Size (8 bytes):** Tamanho dos dados do chunk em bytes (u64 LE).

### 4.2. Chunk Data (Canvas Entries)
Alguns chunks (`thum`, `layr`) utilizam uma estrutura interna de entradas:
*   **Type (4 bytes):** Tipo da entrada (ex: `0x11` para Thumbnail Losssy).
*   **Size (4 bytes):** Tamanho da entrada.
*   **Data (Variável):** Conteúdo.

## 5. Endianness
*   **Little-Endian:** Verificado nos campos de largura, altura e tamanhos de chunks.
*   **Evidência:** O valor `87 00 00 00` interpretado como `135` reflete as dimensões reais de canvas observadas em ferramentas de inspeção.

## 6. Compressão
*   **Zlib:** Indícios em alguns blocos de dados.
*   **DPCM (Differential Pulse Code Modulation):** Utilizado para thumbnails lossless e dados de pixels de camada (`lpix`). Requer reconstrução: `Pixel[n] = Pixel[n-1] + Delta[n]`.
*   **JPEG:** O chunk `view` e `thum` pode conter JPEGs encapsulados em contêineres `JSSF`.

## 7. Dados de Imagem (Raster)
*   **Tiles:** O SAI2 armazena pixels em blocos (tiles), comumente de 256x256 pixels.
*   **Canais:** Armazenamento em formato BGRA (Blue, Green, Red, Alpha).
*   **Reconstrução:** Exige o processamento de múltiplos chunks `layr` e `lpix` associados.

## 8. Thumbnail / Preview Embutido
*   **Existe preview?** Sim, altamente comum.
*   **Chunk Tags:** `thum` (pequena miniatura) e `view` (visualização de maior qualidade).
*   **Formato:**
    *   **Lossy:** Stream JPEG dentro de um cabeçalho `JSSF`.
    *   **Lossless:** Dados DPCM crus.
*   **Extração:** Localizar o chunk `view` na lista de descritores, buscar pela assinatura `JSSF` nos dados e extrair o JPEG sequencial.

## 9. Metadados
*   **Histórico:** Chunk `hist` (ou `normhist`) contém strings UTF-16 com datas de salvamento e modificação.
*   **Nomes de Camada:** Armazenados nos chunks `layr`.

## 10. Engenharia Reversa Estrutural
*   **Container de Blocos:** O formato é extensível via tags de 4 letras.
*   **Escaneamento Resiliente:** Devido à inconsistência do campo `Chunk Count` entre versões, a estratégia de escanear a Tabela de Chunks por strings conhecidas é obrigatória para parsers modernos.

## 11. Estratégia para Implementação de Parser
1.  **Validar Header:** Checar `SAI-CANVAS`.
2.  **Determinar Chunk Count:** Testar offset `0x28`. Se zero, checar `0x14` ou realizar scan linear de tags ASCII começando em `0x40`.
3.  **Mapear Offsets:** Data Offset de um chunk `i` é `(Header + ListSize) + sum(Sizes i-1)`.
4.  **Priorizar Visualização:** Buscar chunk `view`. Se não existir, usar `thum`.

## 12. Pseudocódigo de Parser
```pseudo
open file
read magic -> "SAI-CANVAS"
width = read_u32_le(0x20)
height = read_u32_le(0x24)

# Chunk list parsing
chunks = []
seek(0x40)
while current_pos < filesize:
    tag = read_string(4)
    if not is_ascii(tag): break
    id = read_u32_le()
    size = read_u64_le()
    chunks.append({tag, size})

# Offset calculation
data_start = current_pos
running_offset = data_start
for chunk in chunks:
    if chunk.tag == "view":
        seek(running_offset)
        extract_jssf_jpeg(chunk.size)
        return
    running_offset += chunk.size
```

## 13. Estratégia para Geração de Thumbnail
*   **Alta Fidelidade:** Extrair do chunk `view`.
*   **Compatibilidade:** Implementar o decodificador DPCM para arquivos salvos em modo sem perdas.
*   **Pipeline:** `List Scan -> Offset Resolve -> JSSF Detect -> JPEG Decode`.

## 14. Estratégia para Visualização Básica
*   Extração do JPEG embutido é a única forma prática sem implementar o motor de renderização de tiles e camadas proprietário.

## 15. Mapa Comparativo Entre Arquivos
| Arquivo | Versão Header | Chunks Detectados | Resolução | Observações |
| :--- | :--- | :--- | :--- | :--- |
| `elfinha4.sai2` | TYPE0 | 135 | 135x276 | Contagem oficial em 0x28 é 0. |

## 16. Pontos Incertos
*   **Campo 0x14 (Confiança: 80%):** Parece ser um contador alternativo ou flag de offsets, visto o valor 2490 em um arquivo de 135 chunks.
*   **Encriptação por Máquina (Confiança: 60%):** O software possui opção de salvar arquivos que só abrem no PC original; esses arquivos provavelmente usam o mesmo container mas com os dados dos chunks encriptados via AES ou similar (baseado em hardware ID).

## 17. Conclusão Técnica
O `.sai2` é um formato de blocos bem estruturado mas com variações de cabeçalho que desafiam parsers estáticos. A extração de thumbnails via chunks `view`/`thum` é viável e performática, desde que o parser utilize escaneamento de tags para localizar os dados corretamente.
