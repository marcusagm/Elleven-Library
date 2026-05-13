# Sprint 11.2: Correção e Estabilização de Fontes e Vetores (Arquitetura V2)

**Status da sprint:** Planejamento
**Data e hora de início da sprint:** -
**Data e hora da conclusão estimada:** -

## Objetivo

Corrigir as regressões e completar o suporte para arquivos de Fontes e Vetores na Arquitetura V2, eliminando falhas de visualização e melhorando a extração de metadados.
1. Resolver o problema de "thumbnails brancas" em todos os formatos de fontes.
2. Restaurar a indexação e visualização de arquivos PostScript (`.ps`).
3. Corrigir a falta de extração de metadados em arquivos SVG.
4. Reposicionar o PDF como Documento e implementar geração de thumbnails reais.
5. Melhorar a fidelidade visual e transparência em previews de EPS/PS.

## Análise de Gap e Problemas Identificados

Com base na verificação manual (`task-list-end-new-backend.md`), os seguintes problemas foram detectados:

### Fontes
| Extensão                  | Problema Identificado                                 | Impacto | Causa Provável                                                                                 |
| :------------------------ | :---------------------------------------------------- | :------ | :--------------------------------------------------------------------------------------------- |
| `otf`, `ttf`, `woff`, etc | Thumbnail completamente branca.                       | 🟠 Grave | Falha na renderização do texto pelo `resvg` (mismatch de family name ou erro no template SVG). |
| `eof`                     | Thumbnail branca (regressão da V1 que não suportava). | 🟡 Médio | Falta de suporte a glifos no extractor nativo.                                                 |

### Vetores
| Extensão | Problema Identificado                                             | Impacto      | Causa Provável                                                                    |
| :------- | :---------------------------------------------------------------- | :----------- | :-------------------------------------------------------------------------------- |
| `ps`     | Arquivos não são indexados na V2.                                 | 🔴 Bloqueante | Extensão ausente na lista `supported_extensions` do `AiFormatProvider`.           |
| `eps`    | V2 não gera nenhuma thumbnail.                                    | 🟠 Grave      | O extrator retorna PDF, mas o `ThumbnailCapability` não converte PDF para imagem. |
| `svg`    | Erro `Application error... does not support metadata extraction`. | 🟠 Grave      | `SvgFormatProvider` não implementa o trait `MetadataCapability`.                  |
| `pdf`    | Classificado como Vector; sem thumbnail (apenas ícone).           | 🟡 Médio      | `MediaType` incorreto e falta de implementação de renderização de página.         |

## Referências Técnicas

### V1 (Legacy Context)
- `src-tauri/src/thumbnails/font.rs`: Template SVG e decodificação WOFF via `wuff`.
- `src-tauri/src/thumbnails/svg.rs`: Renderização estável via `resvg`.
- `src-tauri/src/thumbnails/extractors/eps.rs`: Conversão via `pstopdf` (macOS) ou Ghostscript.

### V2 (Current Implementation)
- `src-tauri/src/processing/media/font_format.rs`: Implementação atual que gera thumbnails brancas.
- `src-tauri/src/processing/media/svg_format.rs`: Falta metadados.
- `src-tauri/src/processing/media/pdf_format.rs`: Stub básico.
- `src-tauri/src/processing/media/ai_format.rs`: Provedor central para AI/EPS/PS.

---

## Tarefas do Plano de Ação

### 1. Estabilização de Fontes (`FontFormatProvider`)
- [ ] **Fix Thumbnail**: Investigar o mismatch entre o nome da família extraído pelo `ttf-parser` e o usado no template SVG.
- [ ] **Background Contrast**: Ajustar a cor de fundo do template SVG para garantir visibilidade mesmo se a fonte falhar.
- [ ] **Metadados Estendidos**: Extrair versão da fonte, copyright e quantidade de glifos.
- [ ] **WOFF/WOFF2**: Validar se a descompressão está funcionando corretamente antes de carregar no `fontdb`.

### 2. Correção de Vetores (`SvgFormatProvider` & `AiFormatProvider`)
- [ ] **SVG Metadata**: Implementar `MetadataCapability` para SVG (extrair ViewBox, width, height e título se disponível).
- [ ] **PS Indexing**: Adicionar `"ps"` ao `supported_extensions` do `AiFormatProvider`.
- [ ] **EPS/PS Thumbnails**: Modificar `AiFormatProvider::generate` para que, se o extrator retornar um PDF (Mime `application/pdf`), ele renderize a primeira página como imagem (PNG/WebP) em vez de retornar os bytes brutos do PDF.
- [ ] **Transparency Fix**: Ajustar o fundo do preview de EPS para evitar o "fundo preto" relatado na verificação manual.

### 3. Reestruturação de PDF (`PdfFormatProvider`)
- [ ] **Classificação**: Alterar `MediaType` de `Vector` para `Document`.
- [ ] **Metadados de Documento**: Implementar extração de:
    - Número de páginas.
    - Autor, Título, Data de Criação (via metadados PDF).
    - Dimensões da primeira página.
- [ ] **Thumbnail Real**: Implementar geração de thumbnail da primeira página (usando FFmpeg como fallback ou biblioteca PDF nativa se disponível).

---

## Critérios de Aceitação

- [ ] Fontes exibem amostras de texto ("Aa") e nome da família corretamente no Grid.
- [ ] Arquivos `.ps` são indexados e exibem thumbnail.
- [ ] Arquivos `.eps` exibem thumbnail (mesmo os que dependem de conversão para PDF).
- [ ] Clicar em um SVG não resulta em erro de "metadata extraction".
- [ ] PDFs aparecem com a categoria "Document" no ItemInspector.
- [ ] Metadados de PDF (autor, páginas) são visíveis no Advanced Data.

---

## Riscos e Mitigações

| Risco                      | Impacto                                                       | Mitigação                                                                                                             |
| :------------------------- | :------------------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------- |
| **Dependências Externas**  | EPS/PS dependem de `pstopdf` ou `gs` no sistema.              | Manter o fallback gracioso para thumbnails XMP/Binary integrados quando as ferramentas de OS não estiverem presentes. |
| **Performance de PDF**     | Renderizar PDFs pesados para thumbnails pode travar o worker. | Limitar a renderização apenas à primeira página e usar limites de timeout estritos.                                   |
| **Renderização de Fontes** | Algumas fontes complexas podem não carregar via `usvg`.       | Implementar um fallback visual (ícone genérico com nome da fonte) se a renderização falhar.                           |
