# Correção de Cores CMYK em Miniaturas Adobe (AI/EPS)

**Data:** 2026-03-03
**Autor:** Antigravity Agent
**Objetivo:** Solucionar o defeito de inversão de cores (matrizes "azuladas" ou "negativas") resultante de perfis CMYK corrompidos nos metadados XMP embutidos pela Adobe em arquivos `.ai` e `.eps`.

## 1. O Problema Identificado

Durante as validações visuais das thumbnails na galeria do Mundam, foi detectado que os arquivos Adobe Illustrator (`.ai`) e os seus derivados vetorizados (`.eps`) renderizavam a imagem perfeitamente quando chamados pela Webview (utilizando o fallback de extração de PDF stream de alta qualidade). Contudo, a geração de miniaturas (thumbnails de pré-visualização rápida) apresentava **diferenças e inversões de cores**, tornando imagens verdes em azuladas ou alaranjadas em tons celestes. 

A investigação concluiu que isso ocorria porque o Illustrator salva a miniatura escondida nos blocos `<xmp>` no espaço de cor **CMYK** (otimizado para impressão comercial) ou em variações YCCK (JPEG). A biblioteca base `image::load_from_memory` nativa do Rust interpretava ingenuamente o JPEG como dependente de um espaço RGB (Red-Green-Blue) em algumas das vezes, causando o vazamento cromático na visualização.

## 2. A Solução Implementada

Para resolver a inversão de cores causada pelo `APP14` Adobe color-swap e manter o sistema leve e enxuto:

1. **Injeção do Decoder `zune-jpeg`**: O crate utilitário flexível `zune-jpeg` foi acoplado no topo da cascata de compressão, como ele suporta decodificação YCCK/CMYK com fallback à formatação nativa RGB sem perder as métricas de pixel.
2. **Refatoração no Extrator XMP (`extract_xmp_thumbnail_safe`)**:
   - Atualizamos a função no código `src-tauri/src/thumbnails/extractors/ai.rs`. Ao invés de lermos as entranhas base64 e retornar como "qualquer imagem", injetamos o decoder `zune_jpeg::JpegDecoder`.
   - Caso o decoder ateste com sucesso a captura da imagem (`decode_headers() == Ok`), prosseguimos extraindo todos os pixels RGB cruzes do Buffer.
   - Em vez de devolver RAW JPEGs duvidosos oriundos do ecossistema Adobe, forçamos o Crate local `image` a converter irreversivelmente o novo Buffer retificado com cores corretas em uma imagem do tipo **`image/png`** na hora de retornar.

### Implementações Realizadas:
- Inclusão do header `image::ImageEncoder`.
- Correção explícita de `mime_type` para `"image/png"` no roteamento dos arquivos em `eps.rs` e `ai.rs`.

## 3. Obstáculos Encontrados

Durante a abordagem de retificação de cores, sofremos alguns entraves arquiteturais:
* **Over-engineering com Manual Swap:** Em certo momento, tentamos realizar uma engenharia reversa para adivinhar a orientação de Cores (Trocando manualmente os bytes Azul pelo Vermelho (`chunk.swap(0, 2)`). Contudo, percebemos que *nem toda imagem* do pacote Adobe vinha defeituosa. Forçar a barra causou uma "Inversão Invertida", quebrando a cor de absolutamente **todas as imagens** de vez na base de testes.
* **Resolução Final:** Revertemos as trocas manuais (`chunk.swap(0, 2)`) e deixamos estritamente a pipeline abstrata do `zune-jpeg` fluir, convertendo de bytes puros para o buffer nativo e transacionando para PNG, o que delegou ao parser interno do zune decifrar se o YCCK/CMYK se desfez adequadamente ou não, eliminando as falsas colorações de vez.

## 4. Próximos Passos e Melhorias Futuras

Mesmo com a estabilidade de cores agora providenciada e testada, ainda há margens para otimização da cadeia vetorial:

- **Esvaziamento Contínuo:** Como as lógicas do decoder exigem varredura de binários espessos, poderíamos atrelar em atualizações futuras varreduras parciais limitando buffers para não escalar ram, embora o cap de ~10MB no read stream atual do XMP já previna isso de forma muito saudável.
- **Fallbacks Estáticos:** Investigar se os EPS da era 1990 (v3 antigos) ainda podem gerar "negativas" de cores com parsers isolados e, caso ocorra, adicionar detecção por cabeçalho mágico customizado na nossa pipeline `thumbnails/extractors`.
