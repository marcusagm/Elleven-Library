# Implementação de Suporte a Corel Painter (.rif)

**Data:** 19 de Fevereiro de 2026
**Status:** Concluído
**Autor:** Antigravity (Assistant)

---

## 1. Objetivo
Adicionar suporte nativo para geração de thumbnails e previews de arquivos do **Corel Painter** (extensões `.rif`, `.riff`) no application Mundam, garantindo alta performance e qualidade visual.

## 2. Análise Prévia
A análise técnica dos arquivos de exemplo (`Line Sketches1.rif`, `env.rif`, etc.) revelou:
*   **Formato:** RIFF (Raster Image File) proprietário do Painter.
*   **Header:** Assinatura de versão moderna (`0x00 02`) nos primeiros bytes.
*   **Preview:** A maioria dos arquivos salvos em versões modernas contém uma thumbnail **JPEG** de alta qualidade embutida no final do arquivo.
*   **Estratégia Escolhida:** Extração direta do fluxo de bytes JPEG (Binary Scan), evitando a complexidade de parsear a estrutura proprietária de camadas e simulação física de tinta.

## 3. Implementação Detalhada

### Passo 1: Criação do Extrator
Criamos um novo módulo `src-tauri/src/thumbnails/extractors/corel_painter.rs`.

A lógica implementada:
1.  **Validação de Header:**
    *   Lê os primeiros 8 bytes.
    *   Verifica se bytes `0-1` são `0x00 02` (Versão 2).
    *   Implementa fallback de verificação para assinatura "RIFF" (embora arquivos modernos usem a assinatura binária).
2.  **Binary Scan (Varredura):**
    *   Lê o arquivo para um buffer.
    *   Busca pela assinatura de início de JPEG: `FF D8 FF E0` (SOI + APP0).
    *   Busca pela assinatura de fim de JPEG: `FF D9` (EOI).
    *   Extrai o slice de bytes entre esses marcadores.

```rust
// Trecho simplificado da lógica de scan
if let Some(start_offset) = find_sequence(&buffer, &jpeg_start_sig) {
    if let Some(end_relative) = find_sequence(&buffer[start_offset..], &jpeg_end_sig) {
        // ... extrai dados
    }
}
```

### Passo 2: Registro do Formato
Atualizamos o registro central em `src-tauri/src/formats/definitions.rs` para incluir o novo tipo de arquivo.

```rust
FileFormat {
    name: "Corel Painter Image",
    extensions: &["rif", "riff"],
    mime_types: &["application/x-painter-rif", "image/x-rif"],
    type_category: MediaType::Project, // Categoria Project pois é arquivo de trabalho
    strategy: ThumbnailStrategy::NativeExtractor,
    preview_strategy: PreviewStrategy::NativeExtractor,
    playback: PlaybackStrategy::None,
},
```

### Passo 3: Roteamento de Extração
Atualizamos `src-tauri/src/thumbnails/extractors/mod.rs` para conectar as extensões à nova função.

1.  Adicionado `pub mod corel_painter;`.
2.  Adicionado o case no `match` de extensões:
    ```rust
    "rif" | "riff" => {
        corel_painter::extract_corel_painter_preview(path)
    },
    ```

### Passo 4: Testes
Adicionamos um teste unitário (`#[test]`) no próprio arquivo do extrator.
*   **Alvo:** `file-samples/Imagens/Design/Corel Painter/Line Sketches1.rif`.
*   **Validação:** Verifica se o retorno é `Ok`, se o MIME é `image/jpeg` e se os dados começam com o header JPEG correto.
*   **Resultado:** O teste passou com sucesso (`test result: ok. 1 passed`).

## 4. Conclusão
O suporte foi implementado seguindo estritamente as diretrizes de Rust do projeto (segurança, tratamento de erros centralizado via `Box<dyn Error>` para conversores rápidos, e performance). A solução é robusta para arquivos modernos do Corel Painter e ignora dados proprietários complexos, focando apenas na experiência de visualização do usuário.
