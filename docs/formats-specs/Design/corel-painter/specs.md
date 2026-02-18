# Technical Specification: Corel Painter (.rif)

## 1. Visão Geral do Formato
*   **Nome da Extensão:** `.rif` (Raster Image File)
*   **Possível Origem:** Procreate (Fractal Design), MetaCreation, Corel Corporation (Corel Painter).
*   **Categoria:** Documento de Gráficos Raster Multicamada / Digital Art.
*   **Assinatura Mágica (Hexadecimal):** `00 02` (Version 2) ou `52 49 46 46` (Legacy RIFF variant).
*   **Tamanho Típico Observado:** 80 KB (minimalista) a 50+ MB.
*   **Variações entre Arquivos Analisados:** Todos os arquivos modernos analisados utilizam a assinatura `00 02` e seguem uma estrutura de cabeçalho fixa de 8 bytes seguida por blocos binários.

## 2. Estrutura Binária Global

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| ------ | ------- | ---- | ------------- | --------- | ----------- |
| `0x00` | 8 bytes | `Struct` | **Global Header** | Identificação e dimensões do canvas. | Big-Endian. |
| `0x08` | Variável | `Binary` | **Raster Data** | Dados comprimidos do canvas (pixel layers). | Frequentemente o maior bloco. |
| `EOF-Var`| Variável | `List` | **Metadata Segment**| Segmento contendo miniatura e metadados. | Localizado ao fim do arquivo. |

## 3. Header Principal

*   **Estrutura Detalhada (8 bytes):**
    *   `0x00`: `u16` Version (Sempre `0x0002` em arquivos modernos).
    *   `0x02`: `u16` Flags (ex: `0x2000` para arquivos complexos, `0x0000` para simples).
    *   `0x04`: `u16` Width (BE).
    *   `0x06`: `u16` Height (BE).
*   **Endianness:** Big-Endian.

## 4. Estruturas Internas Identificadas

### 4.1. Metadata Blocks (Tagged Pairs)
Os blocos de metadados seguem um padrão de identificação via tags:
*   **Header do Bloco:**
    *   `u32 BE TotalSize`: Tamanho total do bloco (Tag + Payload).
    *   `4-char Tag`: Identificador ASCII (ex: `PCOL`).
    *   `u32 BE PayloadSize`: (Opcional em alguns blocos) Tamanho dos dados reais.
*   **Tags Comuns:**
    *   `PCOL`: Paper Color (Geralmente 34 bytes).
    *   `FSKT`: Friskets (Máscaras de proteção).
    *   `ANNO`: Annotations (Anotações do usuário).
    *   `NOTE`: Note Text (Pode incluir metadados de dimensões da miniatura).
    *   `ICCP`: ICC Profile (Perfil de cor embutido).
    *   `BUMB`: Bump map/Texture (Dados de superfície).

## 5. Endianness
*   **Principal:** Big-Endian.
*   **Evidência:** Dimensões como `01 04` (260) e `01 F4` (500) coincidem com a largura e altura reais quando interpretadas em Big-Endian.

## 6. Compressão
*   **Indícios:** A proporção entre o tamanho do arquivo e o número de pixels ($Width \times Height$) indica uma compressão eficiente.
*   **Algoritmos Estimados:** Provavelmente utiliza uma variação de RLE ou compressão bitstream proprietária para os dados de pincel e camadas.
*   **Miniaturas:** Utilizam compressão standard **JPEG**.

## 7. Dados de Imagem (Raster)
*   **Início:** Offset `0x08`.
*   **Estrutura:** Fluxo binário proprietário. O Corel Painter armazena não apenas pixels, mas também propriedades de simulação física (umidade, pigmento).
*   **Diferenciação:** O chunk `LAYR` pode ser usado para separar dados de camadas individuais.

## 8. Thumbnail / Preview Embutido
*   **Existe preview?** Sim, na maioria das versões modernas.
*   **Localização:** Geralmente próximo ao fim do arquivo, antes dos blocos de metadados.
*   **Formato:** **JPEG** (standard JFIF).
*   **Detecção Automática:** Pesquisar pela assinatura `FF D8 FF E0` (JPEG Start of Image).
*   **Extração:** O bloco JPEG termina com a marca `FF D9`.

## 9. Metadados
*   **ICC Profiles:** Frequentemente embutidos ao final do arquivo, seguindo o padrão da International Color Consortium.
*   **Strings de Texto:** Encontradas nos blocos `NOTE` ou `ANNO` em formato ASCII ou UTF-16.

## 10. Engenharia Reversa Estrutural
*   **Record Chaining:** O segmento de metadados é uma sequência de registros `[Size][Tag][Data]`.
*   **Container:** Funciona como um container linear simples, onde os dados principais ocupam o início e os metadados são anexados ao fim.

## 11. Estratégia para Implementação de Parser
1.  **Validar Header:** Ler os primeiros 8 bytes e validar `Version == 2`.
2.  **Identificar Miniatura:** Realizar um scan binário por `FF D8 FF E0` para extração imediata do preview.
3.  **Mapear Blocos:** Iniciar leitura sequencial a partir do offset encontrado após o raster data até o EOF.
4.  **Tratamento de Erros:** Ignorar blocos com tags desconhecidas ou tamanhos que excedam o limite do arquivo.

## 12. Pseudocódigo de Parser
```pseudo
open file
header = read(8)
if header.ver != 2: raise Error("Legacy or Invalid Format")

canvas_w = header.w
canvas_h = header.h

# Thumbnail extraction
pos = find_sequence(FF D8 FF E0)
if pos != -1:
    end_pos = find_sequence(FF D9 from pos)
    thumb_data = read(pos to end_pos)
    save_as_jpeg(thumb_data)

# Metadata parsing
seek(end_of_raster_data)
while not EOF:
    block_size = read_u32_be()
    block_tag = read_string(4)
    block_data = read(block_size - 4)
    process_metadata(block_tag, block_data)
```

## 13. Estratégia para Geração de Thumbnail
*   **Abordagem Recomendada:** Extração do JPEG embutido. É a forma mais rápida e precisa, pois reflete o estado salvo do documento sem processar a simulação de pintura.
*   **Fallback:** Se não houver JPEG, a decodificação do raster principal é desencorajada devido à complexidade do motor de renderização proprietário.

## 14. Estratégia para Visualização Básica
*   Exibir a miniatura JPEG extraída.
*   Para visualização do canvas real, seria necessário um motor capaz de interpretar os pacotes de dados proprietários pós-header (O(N) de alta complexidade).

## 15. Mapa Comparativo Entre Arquivos

| Arquivo | Versão | Resolução | Thumbnail | Blocos Extras |
| ------- | ------ | --------- | --------- | ------------- |
| `splat.rif` | 2 | 260x500 | N/A | NOTE, ANNO |
| `Line Sketches1.rif` | 2 | 826x1169| JPEG | FSPG, PCOL |
| `env.rif` | 2 | 826x1169| JPEG | ICCP, BUMB |

## 16. Pontos Incertos
*   **Algoritmo de Compressão Raster (Confiança: 10%):** O formato dos dados entre `0x08` e o Thumbnail é altamente opaco e proprietário.
*   **Flags do Header (Confiança: 40%):** O bit `0x2000` parece indicar a presença de camadas complexas ou simulações físicas ativas.
*   **Tag BUMB (Confiança: 90%):** Relacionada à simulação de "Bump" (relevo da tinta) característica do software.

## 17. Conclusão Técnica
O formato `.rif` (Painter 2.x) é um container binário otimizado para salvar o estado de simulação artística. Para sistemas de catalogação externa (como o Mundam), a extração de miniaturas é trivial via scan de blocos JPEG, mas a reconstrução total do documento sem o software original é um desafio de engenharia reversa de alta complexidade.
