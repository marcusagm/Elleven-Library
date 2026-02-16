
# Especificação Técnica: Rebelle Project (.reb)

## 1. Visão Geral do Formato

*   **Nome da Extensão:** `.reb`
*   **Origem:** [Escape Motions](https://www.escapemotions.com/), software **Rebelle**.
*   **Categoria:** Projeto de Arte Digital / Container.
*   **Assinatura Mágica:** `50 4B 03 04` (ZIP Local File Header) ou `50 4B 05 06` (ZIP End of Central Directory). O arquivo é um **container ZIP padrão**.
*   **Tamanho Típico:** Dezenas a centenas de Megabytes (depende da resolução e número de camadas).
*   **Variações:** A estrutura interna de arquivos (nomes e XML) parece consistente entre a versão 5 e versões recentes.

---

## 2. Estrutura Binária Global

O arquivo segue estritamente o formato ZIP (PKWARE).

| Offset | Tamanho | Tipo | Nome do Campo | Descrição | Observações |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0x00` | 4 bytes | `u32` | **Local File Header Signature** | Assinatura ZIP (`0x04034B50`). | Início do primeiro arquivo no arquivo. |
| `...` | Variável | - | **File Data** | Dados comprimidos (Deflate) ou armazenados. | Conteúdo dos arquivos XML, PNG, DAT. |
| `...` | Variável | - | **Central Directory** | Índice de todos os arquivos no ZIP. | Localizado no final do arquivo. |
| `EOF-22` | 22 bytes | - | **EOCD Record** | End of Central Directory Record. | Marcador final do ZIP. |

**Nota:** Como é um ZIP, não existe um "Header Global" do formato `.reb` proprietário no offset 0. A identificação deve ser feita pela presença de **arquivos específicos** dentro do container (ex: `artwork.xml`, `canvas.png`).

---

## 3. Header Principal

Não aplicável (ver seção 2). O "Header" lógico do projeto está contido no arquivo `artwork.xml` dentro do arquivo.

*   **Arquivo de Identificação:** `artwork.xml`
*   **Root Tag:** `<aquarelle_artwork>`
*   **Atributos Importantes:**
    *   `version`: Inteiro representando a versão (ex: `511` para 5.1.1).
    *   `file_format_version`: Versão estrutural do arquivo (ex: `5`).

---

## 4. Estruturas Internas Identificadas

Dentro do ZIP, os seguintes arquivos são padrões:

| Arquivo (Pattern) | Função | Formato Interno | Observações |
| :--- | :--- | :--- | :--- |
| `artwork.xml` | Metadados do Projeto | XML (UTF-8) | Contém dimensões, lista de camadas, histórico de cores. |
| `canvas.png` | **Preview Composto** | PNG (Standard) | A imagem final renderizada ("merged"). Essencial para thumbnail. |
| `paper.png` | Textura do Papel | PNG (Standard) | Textura de fundo usada na simulação. |
| `layer{N}.png` | Camada de Imagem | PNG (Standard) | Dados de cor (RGB+A) da camada N. |
| `layer{N}_flow.dat` | Mapa de Fluido | Binário Proprietário | Dados de simulação (umidade, pigmento). Assinatura `BBOX`. |
| `layer{N}_structure.dat` | Mapa de Estrutura | Binário Proprietário | Dados de altura/impasto da tinta. |
| `profile.icc` | Perfil de Cor | ICC Profile | Gerenciamento de cores (opcional). |

---

## 5. Endianness

*   **ZIP Container:** Little-endian (padrão PKWARE).
*   **Arquivos Internos dat:**
    *   Análise de `layer0_flow.dat`:
    *   Assinatura `BBOX` (4 bytes).
    *   Valores seguintes parecem ser inteiros de 32 bits Little-endian.
    *   Exemplo: `00 00 01 59` -> 345 (quebra de endianness visual, mas coerente com width/height).
    *   **Conclusão:** Predominantemente **Little-endian** para metadados binários.

---

## 6. Compressão

*   **Container:** ZIP (Deflate).
*   **Imagens:** PNG (Deflate).
*   **Dados Binários (.dat):** Aparentam ter cabeçalhos não comprimidos (`BBOX`, `UCHA`), seguidos por dados que podem estar comprimidos (zlib) ou ser raw arrays de floats/inteiros. A alta entropia sugere compressão ou dados densos de ponto flutuante.

---

## 7. Dados de Imagem

A imagem final e as camadas são armazenadas como **PNGs padrão**.

*   **Dimensões:** Definidas em `artwork.xml` (`<canvas width='...' height='...'/>`) e conferem com os headers IHDR dos PNGs.
*   **Bit Depth:** Tipicamente 8 ou 16 bits por canal (dependendo da configuração do projeto).
*   **Color Type:** RGB ou RGBA (Alpha channel é comum para camadas transparentes).
*   **Reconstrução:**
    *   Para visualização rápida: Usar `canvas.png`.
    *   Para reconstrução fiel editável: Empilhar `layer{N}.png` respeitando a ordem e modos de mesclagem (`blending_mode`) definidos em `artwork.xml` (`<layer ... blending_mode='NORMAL' .../>`).

---

## 8. Thumbnail / Preview Embutido

O formato **possui** um preview de alta qualidade pronto para uso.

*   **Arquivo Alvo:** `canvas.png`
*   **Localização:** Raiz do ZIP.
*   **Formato:** PNG.
*   **Extração:**
    1.  Abrir ZIP.
    2.  Localizar entry `canvas.png`.
    3.  Descomprimir stream.
*   **Detecção Automática:** Verificar existência de entry `canvas.png` ou `preview.png` (em versões mais antigas ou futuras).

---

## 9. Metadados

O arquivo `artwork.xml` é a fonte primária.

**Principais Tags:**
*   `<canvas width='1654.000000' height='2339.000000' .../>`: Dimensões físicas.
*   `<paper name='HP01 Hot Pressed' .../>`: Tipo de papel utilizado.
*   `<layer ... name='Layer 1' type='FLUID' opacity='1' blending_mode='NORMAL' .../>`: Definição de camadas.
*   `<reference_colors_history>`: Paleta de cores usadas recentemente (Hex codes).
*   `<speedpaint_recording .../>`: Se há gravação de timelapse configurada.

---

## 10. Engenharia Reversa Estrutural (Arquivos .dat)

Análise preliminar dos arquivos `_flow.dat`:

*   **Header:** 16 bytes.
    *   Magic: `42 42 4F 58` ("BBOX" - ASCII).
    *   Unknown (4 bytes): `00 00 01 59` (Provável Coordenada ou Dimensão?).
    *   Unknown (4 bytes): `00 00 00 B9`.
    *   Unknown (4 bytes): `00 00 09 B3`.
    *   *Hipótese:* Coordenadas de uma Bounding Box (X1, Y1, X2, Y2) para otimizar processamento apenas na área pintada.
*   **Bloco Seguinte:**
    *   Magic: `55 43 48 41` ("UCHA" - ASCII).
    *   Tamanho/Length: 4 bytes subsequentes (`00 6C 96 A5` no exemplo analizado).
    *   Dados: Conteúdo binário denso (provavelmente array de floats para simulação de fluido).

---

## 11. Estratégia para Implementação de Parser

Para fins de catalogação (Mundam), não é necessário parsear os arquivos `.dat`.

**Pipeline Sugerido:**

1.  **Validação Rápida:** Checar Magic Bytes do arquivo (`PK\x03\x04`).
2.  **Scan de Central Directory:** Listar arquivos internos.
3.  **Identificação:** Procurar por `artwork.xml` e `canvas.png`. Se ausentes, não é um arquivo Rebelle válido (ou é uma versão desconhecida).
4.  **Extração de Preview:** Extrair `canvas.png`.
5.  **Extração de Metadados (Opcional):** Parsear `artwork.xml` (API SAX ou DOM simples) para obter dimensões exatas e versão.

---

## 12. Pseudocódigo de Parser

```python
def parse_rebelle(filepath):
    if not is_zip_file(filepath):
        raise InvalidFormatException("Not a ZIP container")

    with ZipFile(filepath) as zf:
        file_list = zf.namelist()
        
        # Validação do formato
        if "artwork.xml" not in file_list:
            raise InvalidFormatException("Missing artwork.xml")
            
        # Extração de Metadados Básicos
        with zf.open("artwork.xml") as meta_file:
            xml_tree = parse_xml(meta_file)
            width = xml_tree.find("canvas").attr("width")
            height = xml_tree.find("canvas").attr("height")
            version = xml_tree.root.attr("version_str")

        # Extração de Imagem
        if "canvas.png" in file_list:
            preview_data = zf.read("canvas.png")
            return {
                "metadata": {
                    "width": width, 
                    "height": height, 
                    "software": f"Rebelle {version}"
                },
                "preview_blob": preview_data
            }
        else:
            # Fallback (improvável em arquivos válidos)
            raise MissingPreviewException()
```

---

## 13. Estratégia para Geração de Thumbnail

A melhor abordagem é **sempre usar o preview interno (`canvas.png`)**.

*   **Motivo:** Renderizar o arquivo raw exigiria re-implementar o motor de simulação de fluidos da Escape Motions (impossível sem engenharia reversa profunda e acesso a algoritmos proprietários).
*   **Complexidade:**
    *   **Extração:** O(1) (acesso direto ao stream ZIP).
    *   **Decode:** O(N) (onde N é o tamanho de canvas.png).
    *   **Resize:** O(M) (onde M são os pixels da imagem).
*   **Performance:** Muito alta. `canvas.png` geralmente tem alguns MBs, enquanto processar `layer_flow.dat` seria inviável.

---

## 14. Estratégia para Visualização Básica

Não é necessário conversão de RAW. O `canvas.png` já está no espaço de cor sRGB (geralmente, verificar `profile.icc` se precisão de cor absoluta for crítica).

*   **Pipeline:**
    `ZIP Extract -> PNG Decode -> (Optional ICC Transform) -> Display`

---

## 15. Mapa Comparativo Entre Arquivos

| Arquivo Analisado | Versão (XML) | Tamanho | Campos Extras | Observações |
| :--- | :--- | :--- | :--- | :--- |
| `Gordin.reb` | 5.1.1 | 37MB | `struct_flow.dat` | Padrão versão 5. |
| `portrait.reb` | 5.1.1 | 34MB | `struct_flow.dat` | Estrutura idêntica ao anterior. |

Aparentemente, a estrutura é estável na série 5.x.

---

## 16. Pontos Incertos

1.  **Significado exato de .dat:** Os arquivos `_flow.dat` e `_structure.dat` contêm a "mágica" do Rebelle (wetness, ink spreading). Sua estrutura exata não foi revertida completamenta aqui (apenas headers BBOX/UCHA identificados). *Confiança de que não são necessários para thumbnail: 100%.*
2.  **Versões Antigas:** Não foram analisados arquivos da versão 3 ou 4. Pode haver diferenças no nome do arquivo de preview (ex: `merged.png` ao invés de `canvas.png`), mas o padrão ZIP deve se manter.

---

## 17. Conclusão Técnica

O formato `.reb` é **amigável para integração**.
Sua natureza baseada em ZIP + XML + PNG remove a necessidade de parsing binário complexo para tarefas de arquivamento e visualização. A presença garantida de `canvas.png` torna a geração de thumbnails trivial e performática.

*   **Complexidade de Parsing:** Baixa.
*   **Riscos de Implementação:** Baixo (dependências padrão ZIP/XML/PNG).
*   **Recomendação:** Implementar via extração direta de `canvas.png` ignorando os dados de simulação proprietários.
