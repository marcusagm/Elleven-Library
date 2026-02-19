# Refatoração da Extração de Miniaturas e Previews para Adobe Illustrator (.ai)

**Data:** 2026-02-19
**Autor:** Antigravity Agent
**Objetivo:** Melhorar a qualidade do preview e a robustez da geração de thumbnails para arquivos `.ai`, resolvendo problemas de falha na decodificação de JPEG e baixa resolução.

## 1. O Problema Identificado

Durante a implementação do suporte para arquivos Adobe Illustrator, dois problemas principais foram identificados:

1.  **Baixa Qualidade do Preview:** Os arquivos carregavam previews pixelados (extraídos de thumbnails XMP) em vez de renders vetoriais de alta qualidade, mesmo quando o arquivo era compatível com PDF.
2.  **Falha na Geração de Thumbnails (Fallback para Ícone):** Alguns arquivos falhavam na geração de thumbnails com o erro `Format error decoding Jpeg: "Found a marker with invalid length:1"`. Isso ocorria porque a extração binária ingênua do XMP não tratava corretamente entidades XML (como `&#xA;`) e bits de padding, resultando em dados corrompidos passados para o decodificador JPEG.
3.  **Erro de "Formato Indeterminado":** Arquivos que falhavam no render do PDF passavam os bytes crus do PDF para o decodificador de imagem, que falhava ao não reconhecer o formato.

## 2. A Solução Implementada

### 2.1 Refatoração da Estratégia de Preview (`ai.rs`)

Alteramos a prioridade das estratégias de extração em `src-tauri/src/thumbnails/extractors/ai.rs`:

*   **Antes:**
    1.  Tentar XMP Thumbnail (Rápido, Baixa Qualidade)
    2.  Tentar PDF Stream (Lento, Alta Qualidade)

*   **Depois:**
    1.  **Tentar PDF Stream (Alta Qualidade):** Prioridade máxima. Se o arquivo for compatível com PDF (padrão moderno), extraímos o stream PDF. Isso permite que o frontend/webview utilize seu renderizador nativo de PDF, garantindo zoom infinito e qualidade vetorial.
    2.  **Tentar XMP Thumbnail (Fallback):** Se não houver compatibilidade PDF, usamos a thumbnail embutida.

### 2.2 Robustez na Extração XMP (`ai.rs`)

O extrator de XMP (`extract_xmp_thumbnail_safe`) foi completamente reescrito para ser "binary-safe" e tolerante a falhas:

1.  **Busca Binária:** Localiza os marcadores `<xmpGImg:image>` e `</xmpGImg:image>` no arquivo bruto.
2.  **Conversão Lossy para String:** Converte os bytes encontrados para String usando `String::from_utf8_lossy`, permitindo manipulação de texto mesmo em arquivos binários mistos.
3.  **Sanitização de Entidades XML:** Remove explicitamente entidades como `&#xA;` e `&#xD;` que estavam quebrando o decodificador Base64.
4.  **Filtro de Caracteres:** Mantém apenas caracteres alfanuméricos e simbólicos válidos de Base64 (`+`, `/`, `=`), descartando qualquer lixo ou whitespace.

```rust
// Exemplo da lógica de sanitização implementada
let clean_str: String = raw_str
    .replace("&#xA;", "")
    .replace("&#xD;", "")
    .chars()
    .filter(|c| {
        c.is_alphanumeric() || *c == '+' || *c == '/' || *c == '='
    })
    .collect();
```

### 2.3 Melhoria no Fallback da Geração de Thumbnails (`mod.rs`)

A função `generate_thumbnail_extracted` em `src-tauri/src/thumbnails/extractors/mod.rs` foi atualizada para lidar melhor com falhas no render do PDF:

*   **Lógica Anterior:** Se o render do PDF falhasse, caía para um scanner binário genérico (`binary_jpeg`) que muitas vezes falhava nos mesmos marcadores inválidos.
*   **Nova Lógica:**
    1.  Tenta Renderizar o PDF (PDFium).
    2.  **Se falhar:** Tenta explicitamente `ai::extract_xmp_thumbnail_safe`. Isso garante que usaremos a nossa nova extração robusta de XMP para obter uma thumbnail válida (JPEG) se o render do PDF não for possível (ex: biblioteca PDFium ausente ou erro de parsing).
    3.  Se falhar: Tenta Scanner Genérico.
    4.  Se falhar: Tenta FFmpeg.

## 3. Resultados

*   **Preview:** Arquivos `.ai` modernos agora são exibidos como PDFs vetoriais de alta qualidade.
*   **Thumbnails:** Arquivos que antes exibiam ícones genéricos devido a erros de decodificação agora exibem corretamente as thumbnails extraídas do XMP.
*   **Código:** A função `extract_xmp_thumbnail_safe` foi tornada pública para ser reutilizada como fallback robusto em outras partes do sistema.

## 4. Arquivos Modificados

*   `src-tauri/src/thumbnails/extractors/ai.rs`
*   `src-tauri/src/thumbnails/extractors/mod.rs`
