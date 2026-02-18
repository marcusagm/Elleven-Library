# CorelDRAW (.cdr) File Format Technical Specification

## 1. Visão Geral do Formato
*   **Nome da Extensão:** `.cdr`
*   **Possível Origem:** Corel Corporation.
*   **Categoria:** Documento de Gráficos Vetoriais.
*   **Assinatura Mágica (Hexadecimal):**
    *   **Modern (X4+):** `50 4B 03 04` (ZIP container).
    *   **Legacy (X3 and below):** `52 49 46 46` (RIFF container).
    *   **Ultra Legacy (v5-):** `57 4C 6D 00` (`WLm.` proprietary header).
*   **Tamanho Típico Observado:** 3 KB (minimalistas) a 3 MB nos exemplares.
*   **Variações entre Arquivos Analisados:** Observou-se uma transição clara de containers RIFF para containers ZIP. Arquivos muito antigos utilizam um formato binário direto sem container padrão.

## 2. Estrutura Binária Global

### 2.1. Modern Structure (ZIP-based)
| Offset | Tamanho | Tipo | Nome do Campo | Descrição |
| ------ | ------- | ---- | ------------- | --------- |
| `0x00` | 4 bytes | `u32` | **ZIP Magic** | `50 4B 03 04`. |
| Var    | Var     | `File`| **mimetype**  | `application/x-vnd.corel.zcreate`. |
| Var    | Var     | `Dir` | **previews/** | Pasta contendo imagens PNG/BMP. |
| Var    | Var     | `Dir` | **content/**  | Dados vetoriais em formato XML ou binário proprietário. |

### 2.2. Legacy Structure (RIFF-based)
| Offset | Tamanho | Tipo | Nome do Campo | Descrição |
| ------ | ------- | ---- | ------------- | --------- |
| `0x00` | 4 bytes | `ASCII`| **RIFF Magic**| `RIFF`. |
| `0x04` | 4 bytes | `u32`  | **FileSize**  | Tamanho total do arquivo - 8. |
| `0x08` | 4 bytes | `ASCII`| **CDR Signature**| `CDR ` ou `CDRB`. |
| `0x0C` | Var     | `Chunk`| **Chunks**    | Sequência de blocos sub-RIFF. |

## 3. Header Principal

### 3.1. Modern (X4+)
*   **Estrutura:** Segue o padrão PKZIP.
*   **Endianness:** Little-endian.
*   **Campos:** Local File Headers, Central Directory, End of Central Directory.

### 3.2. Legacy (RIFF)
*   **Estrutura:**
    *   `0x08`: Identificador `CDR ` (CorelDRAW) ou `CDRB` (Versões compressas).
    *   **Versão:** Frequentemente encontrada no sub-chunk `vrsn`.
*   **Endianness:** Little-endian.

## 4. Estruturas Internas Identificadas

### 4.1. Chunks RIFF (Legacy)
*   **vrsn:** Contém 2 bytes indicando a versão do software (ex: `02 00` para v2, `0D 00` para X3).
*   **DISP:** (Display) Bloco que contém a visualização para o Windows Explorer (geralmente WMF ou Bitmap).
*   **icp0:** Chunk que armazena ícone/thumbnail para algumas versões.

### 4.2. ZIP Paths (Modern)
*   `previews/thumbnail.png`: Miniatura padrão do documento (PNG).
*   `content/data/page1.dat`: Dados binários da primeira página.
*   `color/color.xml`: Definições de perfil de cor.

## 5. Endianness
*   **Little-endian:** Verificado nos tamanhos de chunks RIFF e nos cabeçalhos ZIP.
*   **Evidência:** Arquivo `ABCNEWS.CDR` (Legacy) mostra offsets e tamanhos em ordem crescente de significância.

## 6. Compressão
*   **Modern:** Compressão **Deflate** padrão do ZIP.
*   **Legacy (CDRB):** Utiliza algoritmos de compressão LZW ou RLE customizados dentro dos chunks de dados vetoriais para reduzir o tamanho do RIFF.

## 7. Dados de Imagem
*   **Vetorial:** O núcleo do formato descreve curvas de Bézier, preenchimentos gradientes e estilos.
*   **Bitmaps embutidos:** Armazenados como chunks binários ou arquivos separados no ZIP (ex: `content/data/Bitmaps.dat`).

## 8. Thumbnail / Preview Embutido
*   **Modern:** Arquivo `previews/thumbnail.png` dentro do ZIP.
*   **Legacy:** Chunk `DISP` ou `icp0` no container RIFF.
*   **Formato do Preview:**
    *   Modern: **PNG**.
    *   Legacy: **BMP** ou **WMF** (Windows Metafile).
*   **Detecção:**
    *   Extrair arquivo do ZIP.
    *   Pesquisar chunk ID `DISP` no fluxo binário RIFF.

## 9. Metadados
*   **Modern:** Localizados em `META-INF/metadata.xml`.
*   **Legacy:** Localizados no chunk `LIST` do tipo `INFO`. Contém campos como `INAM` (Nome), `ICOP` (Copyright).

## 10. Engenharia Reversa Estrutural
*   **Container Switch:** O CorelDRAW abandonou o RIFF binário opaco em favor do ZIP (XML/Binário) na versão X4 para melhorar a extensibilidade e compatibilidade.
*   **Pointer System:** O container ZIP utiliza o Central Directory no final do arquivo para localizar membros. O container RIFF utiliza offsets sequenciais baseados em tamanhos de blocos.

## 11. Estratégia para Implementação de Parser
1.  **Diferenciação:** Checar bytes `0x00-0x03`.
2.  **Se ZIP:** Usar biblioteca padrão de descompressão e buscar o diretório `previews/`.
3.  **Se RIFF:** Implementar um "Chunk Walker" que lê ID e Size, saltando os blocos não reconhecidos.
4.  **Se WLm.:** Tratar como formato binário legado (complexidade alta, recomenda-se fallback para bibliotecas específicas como `libcdr`).

## 12. Pseudocódigo de Parser
```pseudo
open file
header = read(4)

if header == "PK\x03\x04":
    # Modern ZIP
    zip = open_as_zip(file)
    if "previews/thumbnail.png" exists:
        return extract_file("previews/thumbnail.png")
    else if "previews/page1.png" exists:
        return extract_file("previews/page1.png")

else if header == "RIFF":
    # Legacy RIFF
    skip(4) # skip size
    type = read(4)
    while not EOF:
        chunk_id = read(4)
        chunk_size = read_u32_le()
        if chunk_id == "DISP":
            skip(4) # skip type/flags
            return read_image_data(chunk_size - 4)
        skip(chunk_size + alignment)
```

## 13. Estratégia para Geração de Thumbnail
*   **Modern:** Extração direta do ZIP (O(1) complexity).
*   **Legacy:** Parsing de RIFF chunks. Se o `DISP` contiver WMF, pode exigir renderização extra. Priorizar extração de BMP se disponível no chunk `icp0`.

## 14. Estratégia para Visualização Básica
*   Exibir o thumbnail PNG extraído.
*   Renderização vetorial completa: Exige parsing de múltiplos arquivos `.dat` ou `.xml` e implementação de um motor de renderização vetorial compatível com as especificações da Corel (extremamente complexo).

## 15. Mapa Comparativo Entre Arquivos
| Arquivo | Estrutura | Versão Estimada | Thumbnail | Observações |
| ------- | --------- | --------------- | --------- | ----------- |
| `example.cdr`| ZIP | X4+ | PNG (previews/) | Estrutura moderna completa. |
| `ABCNEWS.CDR`| WLm. | v5 ou inferior | N/A | Formato binário proprietário cru. |
| `01-Receipt...`| RIFF | X3 ou inferior | DISP Chunk | Documento financeiro legado. |

## 16. Pontos Incertos
*   **Formato WLm.:** Praticamente não documentado. Baseia-se em dumps diretos de memória das estruturas do software das décadas de 80/90 (Confiança: 20%).
*   **Custom RIFF Chunks:** Usuários podem estender o formato com chunks privados não padronizados pela Corel (Confiança: 90%).

## 17. Conclusão Técnica
O `.cdr` é um formato que evoluiu de uma estrutura proprietária binária para um container standard (ZIP). A facilidade de parsing para fins de miniatura é alta em versões modernas, mas cai drasticamente para arquivos legados, onde o suporte tende a depender de bibliotecas de engenharia reversa como a `libcdr` do projeto LibreOffice.
