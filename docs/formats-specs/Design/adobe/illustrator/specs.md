# Adobe Illustrator (.ai) File Format Technical Specification

## 1. Visão Geral do Formato
*   **Nome da Extensão:** `.ai` (Adobe Illustrator Artwork).
*   **Possível Origem:** Desenvolvido pela Adobe Inc.
*   **Categoria:** Vetorial / Container (PDF-Hybrid).
*   **Assinatura Mágica (Hexadecimal):** `25 50 44 46` (`%PDF`) para arquivos modernos (v9.0+). Versões legadas (v1-v8) usam `25 21 50 53` (`%!PS`).
*   **Tamanho Típico Observado:** 60 KB a 2 MB (dependendo da complexidade e se a compatibilidade com PDF está ativa).
*   **Variações entre Arquivos Analisados:** Todos os exemplares analisados seguem a estrutura de container PDF (PDF 1.5/1.6), agindo como "Dual Format" que contém dados PDF padrão e dados proprietários do Illustrator encapsulados.

## 2. Estrutura Binária Global
O formato funciona como um container PDF que "esconde" os dados originais do Illustrator em streams de metadados e objetos privados.

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 8 bytes | `ASCII` | **Magic Header** | `%PDF-1.x` | Identificado como PDF. |
| Variável | Variável | `Object` | **PDF Body** | Objetos PDF padrão. | Catalog, Pages, Streams. |
| Variável | Variável | `Metadata` | **XMP Block** | XML com metadados. | Contém thumbnails em Base64. |
| Variável | Variável | `Stream` | **Private Data** | Dados `%AI12_CompressedData`. | Onde reside o vetor real. |
| EOF | 5 bytes | `ASCII` | **Trailer** | `%%EOF` | Fim do arquivo PDF. |

## 3. Header Principal
*   **Estrutura detalhada:** Segue a especificação PDF. O arquivo começa com o marcador de versão.
*   **Campos identificados:** Versão do PDF (ex: `1.5`, `1.6`).
*   **Endianness:** Big-endian (padrão de rede/PDF).
*   **Marcador Especial:** Logo após o header, geralmente há um bloco binário `%âãÏÓ` para indicar que o arquivo contém dados binários (8-bit).

## 4. Estruturas Internas Identificadas

### 4.1. Bloco XMP (Extensible Metadata Platform)
*   **Offset inicial:** Variável (buscado pela tag `<x:xmpmeta>`).
*   **Função:** Armazena metadados estruturados em XML.
*   **Thumbnail:** Localizado dentro de tags `<xmpGImg:image>` em formato JPEG codificado em Base64.

### 4.2. Bloco Private Data (Illustrator Proprietary)
*   **Assinatura:** `%AI12_CompressedData` (ou similar conforme versão).
*   **Estrutura:** Um stream FlateDecode (Zlib) que contém o grafo de objetos vetoriais original do Illustrator.
*   **Função:** Permite que o Illustrator reabra o arquivo com todas as camadas e filtros editáveis, mesmo que o PDF padrão não suporte todos os recursos.

### 4.3. Bloco de Preview Legado (AI7_Thumbnail)
*   **Assinatura:** `%AI7_Thumbnail`.
*   **Estrutura:** Contém largura, altura, profundidade de bits e um stream hexadecimal (`%%BeginData`).
*   **Função:** Usado por versões antigas ou plugins de visualização rápida.

## 5. Endianness
*   **Big-endian:** Padrão herdado do PostScript e adotado pelo PDF.
*   **Evidência encontrada:** Todos os valores de comprimento de stream e identificadores de objetos no container PDF seguem a ordem big-endian.

## 6. Compressão
*   **Algoritmo:** **Zlib / FlateDecode**.
*   **Assinatura:** `78 9C` (Zlib Default Compression) frequentemente encontrada após comandos como `/Filter/FlateDecode`.
*   **Uso:** Aplicado em streams de dados privados e objetos de conteúdo da página.

## 7. Dados de Imagem (Pre-render)
*   **Dimensões:** Definidas no `/MediaBox` do PDF e no dicionário `/Page`.
*   **Bit depth:** Geralmente 8 bits por canal para previews.
*   **Reconstrução:** A visualização básica é feita renderizando o stream PDF padrão contido no arquivo.

## 8. Thumbnail / Preview Embutido
*   **Existência:** Sim, múltiplos níveis.
*   **Extração:**
    1.  **Via XMP:** Decodificar Base64 da tag `<xmpGImg:image>`.
    2.  **Via /Thumb:** Atributo PDF referenciando um objeto de imagem.
    3.  **Via AI7:** Parsing de `%AI7_Thumbnail` e conversão do hex stream.
*   **Formato:** JPEG (em XMP) ou Bitmap indexado/RGB (em AI7).

## 9. Metadados
*   **Strings encontradas:** "Adobe Illustrator", versão do criador (ex: Adobe Illustrator 24.1), data de criação, títulos de camadas.
*   **Estrutura:** XML (XMP) e dicionários `/Info` do PDF.

## 10. Engenharia Reversa Estrutural
*   **Container:** O arquivo é um híbrido. Se renomeado para `.pdf`, abre em leitores comuns.
*   **TLV:** O PDF usa uma estrutura de objetos indexados (`xref table`) que funcionam como ponteiros internos.
*   **Redundância:** O Illustrator salva o desenho duas vezes: uma vez como objetos PDF simples (para compatibilidade) e outra vez como seu formato proprietário comprimido (para edição).

## 11. Estratégia para Implementação de Parser
1.  **Validação de Header:** Checar `%PDF-`.
2.  **Varredura de Metadados:** Procurar por `xmp:Thumbnails` para extração ultra-rápida de preview sem processar o vetor.
3.  **Localização de Objeto /Thumb:** Verificar dicionário da primeira página.
4.  **Parsing Incremental:** Se necessário reconstruir o vetor original, localizar o stream `/Filter /FlateDecode` associado ao marcador `%AIXX_CompressedData`.

## 12. Pseudocódigo de Parser
```pseudo
open file
read magic ("%PDF-")
if not found, check legacy magic ("%!PS-Adobe")

find cross-reference table (xref) at end of file
locate Catalog object
locate Metadata stream

# Extract Thumbnail
search for "<xmpGImg:image>" in entire file (fast scan)
if found:
    extract Base64 content
    decode to JPEG buffer
    return thumbnail

# Fallback
search for "/Page" objects
check for "/Thumb" key
extract referenced Image object stream
return image
```

## 13. Estratégia para Geração de Thumbnail
*   **Melhor Abordagem:** Usar o preview XMP embutido. É uma imagem JPEG pré-renderizada e de fácil acesso.
*   **Complexidade:** Baixa (Regex para encontrar as tags + Decodificação Base64).
*   **Pipeline:** `Find Tag -> Extract -> Base64 Decode -> Save as .jpg`.

## 14. Estratégia para Visualização Básica
*   Utilizar bibliotecas PDF padrão (Poppler, PDF.js, MuPDF) para renderizar a página 1.
*   Não é necessário implementar o motor vetorial proprietário da Adobe para visualização simples.

## 15. Mapa Comparativo Entre Arquivos
| Arquivo | Estrutura | PDF Version | Thumbnail | Observações |
| :--- | :--- | :--- | :--- | :--- |
| `Logo.ai` | Hybrid | 1.6 | XMP/Base64 | Estrutura moderna. |
| `sample.ai` | Hybrid | 1.5 | XMP/Base64 | Segue padrão Adobe CC. |
| `Cake box...` | Hybrid | 1.5 | AI7/XMP | Contém preview legado e moderno. |

## 16. Pontos Incertos
*   **PGF (Progressive Graphics File):** Alguns arquivos citam `Adobe_Direct_PGF`. A estrutura interna deste stream binário é opaca (Confiança: 30%).
*   **Blending Modes Proprietários:** Certos efeitos de transparência do Illustrator podem não aparecer corretamente em renderizadores PDF genéricos se estiverem apenas no Private Data (Confiança: 80%).

## 17. Conclusão Técnica
O formato `.ai` contemporâneo é um exemplo clássico de encapsulamento de dados proprietários em um container aberto (PDF). A extração de thumbnails é facilitada pela redundância de metadados XMP, enquanto o parsing total do vetor exige um motor PDF completo e conhecimento das extensões privadas da Adobe para reconstrução perfeita.
