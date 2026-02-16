# Plano de Implementação: Suporte Avançado a SAI2 no Mundam

Este documento detalha as etapas necessárias para corrigir os problemas de geração de thumbnails e visualizações para arquivos do PaintTool SAI v2 (.sai2), incluindo o suporte ao formato DPCM (lossless).

---

## 1. Diagnóstico dos Problemas Atuais

1.  **Falha na Localização de Chunks:** Versões recentes do SAI2 podem ter alterado a ordem ou a estrutura dos chunks, fazendo com que o parser atual não encontre a tag `"thum"`.
2.  **Falta de Suporte DPCM:** Arquivos configurados para salvar thumbnails "lossless" utilizam compressão DPCM em vez de JPEG, o que resulta em erro no extrator atual.
3.  **Qualidade da Visualização:** O sistema atual extrai apenas a thumbnail embutida (geralmente 256x256), o que é insuficiente para visualizações em tela cheia.

---

## 2. Etapas de Implementação

### Fase 1: Robustez do Parser de Chunks
*   **Ação:** Refatorar a função `parse_chunk_list` em `sai2.rs` para ser mais resiliente a variações de alinhamento.
*   **Melhoria:** Implementar uma busca baseada em assinaturas (magic bytes) caso a tabela de chunks inicial falhe ou pareça inconsistente.
*   **Validação:** Adicionar logs detalhados para mapear todos os tipos de chunks encontrados (`layr`, `lpix`, `thum`, `view`).

### Fase 2: Implementação do Decodificador DPCM
O formato DPCM do SAI2 armazena a diferença entre pixels adjacentes para compressão sem perdas.
*   **Algoritmo:**
    1.  Ler dados brutos do chunk `CANVAS_TYPE_THUMBNAIL_LOSSLESS`.
    2.  Reconstruir os canais BGRA: `P[i] = P[i-1] + Delta[i]`.
    3.  Processar em blocos de 256x256 (tamanho padrão das tiles do SAI2).
*   **Código Rust:** Criar um módulo `dpcm.rs` utilitário para realizar essa reconstrução de forma performática.

### Fase 3: Suporte a Visualizações de Alta Qualidade
Para arquivos sem uma imagem composta de alta resolução embutida:
*   **Estratégia:** Tentar localizar o chunk `view` (se disponível) que contém uma versão de visualização maior que a thumbnail padrão.
*   **Fallback Inteligente:** Se apenas a thumbnail de 256px estiver disponível, aplicar um filtro de upscale (Lanczos ou similar) no frontend para suavizar a visualização inicial enquanto o usuário não solicita a abertura no editor nativo.

---

## 3. Mudanças no Código (Resumo)

### `src-tauri/src/thumbnails/extractors/sai2.rs`
*   Adicionar constante `CANVAS_TYPE_VIEW: u32 = 0x10;` (verificar offset exato).
*   Implementar `decode_dpcm_tile(data: &[u8], width: u32, height: u32) -> Vec<u8>`.
*   Atualizar `extract_sai2_preview` para tentar:
    1.  Chunk `view` (Alta qualidade).
    2.  Chunk `thum` -> `ThumbnailLossy` (JPEG).
    3.  Chunk `thum` -> `ThumbnailLossless` (DPCM).

### `src/components/features/viewport/ReferenceImage.tsx`
*   Adicionar suporte a metadados de "qualidade da fonte" para indicar se a imagem exibida é uma thumbnail upscaled ou uma visualização real.

---

## 4. Cronograma Estimado

| Atividade | Esforço | Prioridade |
| :--- | :--- | :--- |
| Correção do Parser de Chunks | 1 dia | Crítica |
| Implementação do Decodificador DPCM | 2 dias | Alta |
| Integração de Visualizações de Alta Qualidade | 2 dias | Média |
| Testes de Regressão com Versões Antigas/Novas | 1 dia | Alta |

---

## 5. Referências Técnicas
*   [Wunkolo/libsai](https://github.com/Wunkolo/libsai) - Referência principal para a estrutura de arquivos.
*   [SaiThumbs](https://github.com/Wunkolo/SaiThumbs) - Implementação de referência para extensões de shell do Windows.
