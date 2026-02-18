# Technical Analysis: Affinity Publisher (.afpub)

## 1. Visão Geral do Formato

*   **Extensão:** `.afpub`
*   **Origem:** Serif Affinity Publisher.
*   **Categoria:** Documento de Editoração Eletrônica (DTP) / Container.
*   **Assinatura Mágica (Hexadecimal):** `00 FF 4B 41` (Little-Endian: `0x414BFF00`).
*   **Tamanho Típico:** Varia de 150 KB a centenas de MB, dependendo das imagens vinculadas ou embutidas.
*   **Variações:** Compartilha a mesma estrutura de base (Affinity Common Format) que `.afdesign` e `.afphoto`.

## 2. Estrutura Binária Global

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Magic** | `00 FF 4B 41` | Identificador do formato Affinity. |
| `0x04` | 4 bytes | `u32` | **Version/Flags** | Versão do esquema ou flags. | Ex: `0xB` (11) ou `0x8000B`. |
| `0x08` | 8 bytes | `ASCII` | **Persona ID** | `nsrP#Inf` | "Persona Info" em Little-Endian (`Prsn#Inf`). |
| `0x10` | 8 bytes | `u64` | **Content Ptr** | Ponteiro de Conteúdo | Endereço absoluto do bloco de dados principal. |
| `0x18` | 8 bytes | `u64` | **Thumb Ptr** | Ponteiro de Thumbnail | Endereço absoluto do bloco de miniatura. |
| `0x20` | ... | `u64` | **Other Ptrs** | Outros Ponteiros | Sequência de endereços para blocos adicionais. |

## 3. Header Principal

*   **Estrutura:** Bloco inicial de 64 bytes contendo as assinaturas e a tabela de endereçamento base.
*   **Endianness:** **Little-Endian** em todos os campos numéricos.
*   **Campos Críticos:** O ponteiro em `0x18` é o mais relevante para extração de visualização rápida.

## 4. Estruturas Internas Identificadas

As seções internas são organizadas em blocos com um cabeçalho padrão de 8 bytes:
*   `0xFFFFFFFF` (4 bytes)
*   Signature (4 bytes, ex: `Thmb`, `Doc `, `Prop`)

### Bloco de Thumbnail (`Thmb`)
*   **Offset:** Definido no header em `0x18`.
*   **Estrutura:**
    *   `+00`: `FF FF FF FF` (Marcador de Bloco)
    *   `+04`: `Thmb` (Assinatura)
    *   `+08`: Version (u32, geralmente `1`)
    *   `+12`: Total Block Size (u32)
    *   `+16`: Header Length (u32, fixo em `29` ou `0x1D`)
    *   `+20`: Zero (u32)
    *   `+24`: Payload Size (u32 - tamanho do PNG)
    *   `+28`: Flag (1 byte, ex: `0x01`)
    *   `+29`: **PNG Data** (Inicia com `89 50 4E 47`)

## 5. Endianness

*   **Little-Endian.**
*   **Evidência:** Os ponteiros de offset lidos como `u64` Little-Endian apontam corretamente para os blocos de dados no final do arquivo, enquanto a leitura Big-Endian resultaria em endereços fora dos limites do arquivo.

## 6. Compressão

*   **Estrutura Interna:** Os dados do documento em si são comprimidos (Zlib provável) dentro dos blocos de conteúdo.
*   **Miniatura:** Utiliza compressão **PNG** padrão (Deflate/Zlib), facilitando a extração sem necessidade de bibliotecas proprietárias.

## 7. Dados de Imagem

*   O arquivo `.afpub` não armazena uma imagem crua única (como um RAW), mas sim um layout de páginas. No entanto, ele embuti uma visualização (thumbnail) da primeira página ou do spread atual.

## 8. Thumbnail / Preview Embutido

*   **Existe preview?** Sim.
*   **Format:** Standard **PNG**.
*   **Detecção Automática:**
    1.  Ler 8 bytes em `0x18` (Offset `T`).
    2.  Seek para `T`.
    3.  Confirmar `FFFFFFFF` + `Thmb`.
    4.  Extrair stream a partir de `T + 29`.

## 9. Metadados

*   Contém referências a arquivos externos (Linked assets) e fontes.
*   Strings identificadas no header sugerem o uso de um "Object Store" interno onde as propriedades do documento são serializadas.

## 10. Engenharia Reversa Estrutural

*   **Container de Blocos:** O formato é essencialmente um diretório de blocos binários acessados por uma tabela de ponteiros no início do arquivo.
*   **Resiliência:** O uso de ponteiros em vez de offsets fixos permite que o software anexe dados ao final do arquivo sem reescrever todo o conteúdo.

## 11. Estratégia para Implementação de Parser

1.  Validar Magic `00 FF 4B 41`.
2.  Ler ponteiro de miniatura em `0x18`.
3.  Saltar para o offset lido.
4.  Validar cabeçalho do bloco `Thmb`.
5.  Ler o tamanho do PNG em `Offset + 24`.
6.  Extrair o buffer e salvar com extensão `.png`.

## 12. Pseudocódigo de Parser

```pseudo
open file
read magic (4 bytes)
if magic != 0x414BFF00: fail

seek to 0x18
thumb_ptr = read_u64_le()

seek to thumb_ptr
block_magic = read_u32()
block_sig = read_string(4)
if block_sig != "Thmb": fail

seek relative +16
png_size = read_u32_le()

seek relative +1
png_data = read(png_size)
save png_data as preview.png
```

## 13. Estratégia para Geração de Thumbnail

*   **Abordagem:** Extração direta do bloco `Thmb`.
*   **Complexidade:** O(1) - requer apenas dois saltos (seeks) no arquivo, independente do tamanho total.
*   **Pipeline:** `Header Read -> PTR Seek -> Block Validate -> Stream Copy`.

## 14. Estratégia para Visualização Básica

*   Renderizar extraindo o PNG embutido. Devido à natureza DTP do arquivo, renderizar o conteúdo total exigiria reconstruir todo o motor de layout, o que não é viável sem o software original. A miniatura embutida é a representação fiel pretendida.

## 15. Mapa Comparativo Entre Arquivos

| Arquivo | Versão Header | Ptr Thumbnail | Tamanho Thumb | Observações |
| :--- | :--- | :--- | :--- | :--- |
| `handbook.afpub` | 524299 | `0x17BF55` | 53.8 KB | Mock-up de manual. |
| `Flyer German.afpub`| 524299 | `0x19BCE` | 45.4 KB | Documento simples. |
| `evermore.afpub` | 11 | `0x18FEBAA`| 8.3 KB | Documento grande, thumb pequena (ícone). |

## 16. Pontos Incertos

*   **Campo Version (0x04):** O valor varia significativamente entre revisões do software (ex: `11` vs `524299`). Pode incluir flags de compatibilidade (Confiança: 70%).
*   **Estrutura de Conteúdo:** O bloco `Prop` (Properties) contém a árvore de objetos serializada, mas o formato dessa serialização é opaco e proprietário (Confiança: 90%).

## 17. Conclusão Técnica

O formato `.afpub` é altamente estruturado e eficiente para operações de leitura aleatória. A extração de miniaturas é simples e segue um padrão industrial sólido, permitindo interoperabilidade com assistentes e exploradores de arquivos sem risco de corrupção ou necessidade de dependências pesadas.
