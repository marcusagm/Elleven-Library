# Technical Analysis: Encapsulated PostScript (.eps) File Format

## 1. Visão Geral do Formato
*   **Nome da Extensão:** `.eps` (Encapsulated PostScript).
*   **Possível Origem:** Desenvolvido pela Adobe Systems em 1987.
*   **Categoria:** Documento de Gráficos Vetoriais / Container.
*   **Assinatura Mágica (Hexadecimal):**
    *   **Binary EPS:** `C5 D0 D3 C6` (Little-Endian: `0xC6D3D0C5`).
    *   **ASCII EPS:** `25 21 50 53` (`%!PS`).
*   **Tamanho Típico Observado:** 3 KB a 10 MB (dependendo da complexidade vetorial e da presença de previews TIFF).
*   **Variações entre Arquivos Analisados:** Observou-se tanto arquivos puramente textuais (PostScript Puro) quanto arquivos binários (Adobe Generic Header) que embutem o PostScript ASCII junto com miniaturas binárias.

## 2. Estrutura Binária Global

### 2.1. Binary EPS (Adobe Generic)
Arquivos que utilizam o header binário facilitam a visualização rápida sem necessidade de um interpretador PostScript completo.

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `C5 D0 D3 C6`. | Little-endian order. |
| `0x04` | 4 bytes | `u32` | **PS Offset** | Início do código PostScript. | Geralmente 30 (Logo após o header). |
| `0x08` | 4 bytes | `u32` | **PS Size** | Tamanho do código PostScript. | |
| `0x0C` | 4 bytes | `u32` | **WMF Offset** | Início do preview WMF. | 0 se não presente. |
| `0x10` | 4 bytes | `u32` | **WMF Size** | Tamanho do preview WMF. | |
| `0x14` | 4 bytes | `u32` | **TIFF Offset** | Início do preview TIFF. | Comumente localizado após o PostScript. |
| `0x18` | 4 bytes | `u32` | **TIFF Size** | Tamanho do preview TIFF. | |
| `0x1C` | 2 bytes | `u16` | **Checksum** | Header checksum. | Frequentemente `0xFFFF`. |

### 2.2. ASCII EPS (Raw PostScript)
Segue as convenções de estruturação de documentos (DSC - Document Structuring Conventions).

| Offset | Tamanho | Tipo | Descrição |
| :--- | :--- | :--- | :--- |
| `0x00` | Var | `ASCII` | Inicia com `%!PS-Adobe-3.0 EPSF-3.0`. |
| Var    | Var | `Comentário`| Comentários DSC (ex: `%%Title`, `%%BoundingBox`). |
| Var    | Var | `Metadata` | Bloco XMP (XML) embutido em comentários. |
| Var    | Var | `Code` | Operadores PostScript (ex: `moveto`, `lineto`). |

## 3. Header Principal
### 3.1. Binary Header
*   **Estrutura:** 30 bytes fixos.
*   **Campos:** Ponteiros absolutos para os três segmentos possíveis (PostScript, Windows Metafile, TIFF).
*   **Endianness:** Little-endian.

### 3.2. ASCII Header
*   **Estrutura:** Texto livre seguindo convenções proprietárias.
*   **Campos:** Versão do EPSF, Criador, Data, Bounding Box.
*   **Endianness:** N/A (Textual).

## 4. Estruturas Internas Identificadas

### 4.1. Bloco PostScript (Obrigatório)
*   Contém a descrição vetorial real.
*   Em arquivos binários, o offset aponta para este bloco.
*   Termina com o operador `showpage` ou `%%EOF`.

### 4.2. Bloco TIFF Preview (Opcional)
*   **Assinatura:** `49 49 2A 00` (II) ou `4D 4D 00 2A` (MM).
*   **Função:** Uma imagem raster em baixa resolução para exibição rápida em softwares de design que não renderizam PostScript em tempo real.

### 4.3. Bloco XMP Metadata (Moderno)
*   **Localização:** Frequentemente dentro da seção de PostScript como um bloco XML.
*   **Thumbnail:** Tags `<xmpGImg:image>` contêm uma string Base64 que representa um JPEG.

## 5. Endianness
*   **Binary Header:** **Little-Endian**.
*   **Embedded TIFF:** Pode ser **Little-Endian (II)** ou **Big-Endian (MM)**.
*   **PostScript:** O código em si é textual.

## 6. Compressão
*   **Indícios:** O PostScript pode usar filtros como `/FlateDecode` (Zlib) ou `/ASCII85Decode`.
*   **Previews:** O preview TIFF pode estar comprimido com PackBits ou LZW.
*   **Thumbnails XMP:** São JPEGs padrão codificados em Base64.

## 7. Dados de Imagem
*   **Vetorial:** Representado por coordenadas e comandos PS.
*   **Raster embutido:** Pode existir via comando `image` no PostScript ou via preview binário.

## 8. Thumbnail / Preview Embutido
*   **Como detectar automaticamente:**
    1.  Verificar magic `C5 D0 D3 C6`. Se sim, ler offset em `0x14`.
    2.  Se ASCII, buscar pela string `<xmpGImg:image>` para miniaturas modernas.
    3.  Se ASCII legado, buscar por `%%BeginPreview`.

## 9. Metadados
*   **DSC:** `%%Title`, `%%Creator`, `%%CreationDate`.
*   **XMP:** Bloco XML embutido com informações ricas de autoria e histórico de edição (Adobe Creative Cloud).

## 10. Engenharia Reversa Estrutural
*   **Container Híbrido:** O EPS é um dos raros formatos que mistura cabeçalhos binários de comprimento fixo com corpos de dados textuais de comprimento variável.
*   **Pointer System:** O header binário utiliza um sistema de tabelas de offsets absoluto, permitindo pular o PostScript e ir direto para o preview.

## 11. Estratégia para Implementação de Parser
1.  **Diferenciação:** Ler os primeiros 4 bytes.
2.  **Caso Binário:** Extrair os 30 bytes do header, validar o offset do TIFF e extrair o sub-arquivo.
3.  **Caso ASCII:**
    - Scan por marcadores de metadados (`%%BeginMetadata`, `<x:xmpmeta>`).
    - Decodificar XMP Thumbnail se presente (Base64 -> JPEG).
4.  **Tratamento de Erros:** Validar se os offsets lidos no header binário não ultrapassam o tamanho total do arquivo.

## 12. Pseudocódigo de Parser
```pseudo
open file
read magic(4)
if magic == 0xC6D3D0C5:
    header = read(30)
    tiff_offset = header.get_u32(20)
    tiff_size = header.get_u32(24)
    if tiff_size > 0:
        seek(tiff_offset)
        return extract_tiff(tiff_size)

else if (magic == "%!PS"):
    content = read_all()
    if find("<xmpGImg:image>"):
        b64_data = extract_between_tags("<xmpGImg:image>", "</xmpGImg:image>")
        return decode_base64_to_jpeg(b64_data)
    else if find("%%BeginPreview"):
        # Legacy hex preview parsing
        return parse_hex_preview()
```

## 13. Estratégia para Geração de Thumbnail
*   **Alta Velocidade:** Priorizar o preview TIFF da estrutura binária ou o JPEG do XMP.
*   **Complexidade:**
    - TIFF (Binário): Baixa.
    - XMP (ASCII): Média (requer decodificação Base64).
    - PostScript Puro (Sem Preview): Alta (requer renderizador vetorial como Ghostscript).

## 14. Estratégia para Visualização Básica
*   Ao encontrar o TIFF binário, exibi-lo como uma imagem comum.
*   Caso contrário, renderizar apenas o metadado visual se disponível.

## 15. Mapa Comparativo Entre Arquivos
| Arquivo | Estrutura | Preview | Observações |
| :--- | :--- | :--- | :--- |
| `Quran 27-40...` | Binary | TIFF (66 KB) | Preview de alta fidelidade presente. |
| `i18k_e46y...` | ASCII | XMP/JPEG | Formato moderno Adobe. |
| `knightstour.eps` | ASCII | Nenhum | PostScript puro, vetorial simples. |

## 16. Pontos Incertos
*   **WMF Compatibility:** O preview WMF (recurso do Windows) caiu em desuso, mas arquivos antigos ainda podem contê-lo (Confiança: 100% da presença, 40% da utilidade moderna).
*   **Checksum Calculation:** O campo checksum de 2 bytes é raramente validado por softwares modernos, que confiam apenas nos offsets (Confiança: 80%).

## 17. Conclusão Técnica
O `.eps` é um formato de transição bem documentado, mas que exige tratamento duplo para lidar com suas variantes binárias e textuais. A extração de miniaturas é extremamente eficiente em arquivos gerados por softwares profissionais (Adobe/Corel), mas arquivos minimalistas gerados manualmente exigem renderização completa do código PostScript.
