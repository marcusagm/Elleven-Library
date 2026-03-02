# Sprint 5: Special Formats Deep-Dive & Advanced Rendering

**Data:** 2026-03-02
**Status:** Planejado
**Data e hora da conclusão:** -

## 📌 Objetivo
Complementar a Fase 2 do Roadmap garantindo tratamento cirúrgico a arquivos vetoriais essenciais no universo de bibliotecas criativas (DPI ilimitado para web nativa e suporte retroativo a streams PDF em softwares veteranos), prevenindo rastros *pixelados* no DAM.

## 🛠 Tarefas de Implementação

### 1. Vector Engine SVGs & High DPI Scaling
- **Escopo:** Suprir a limitação das bibliotecas genéricas rasters garantindo "Zoom In" infindável na galeria (`GridView.tsx`, `VirtualListView.tsx`) a instâncias SVGs.
- **Ações (Rust & Solid):**
  - **Rust:** Evitar que a indexação gere caches PNG limitados/emborrachados via ffmpeg estrito. Em arquivos detectados com formato vetorial web (SVG nativo), reportar à `FileFormat::detect` que a resposta primordial enviada no Grid/Streaming deve preservar o mimetype e string SVG em cru.
  - **Solid UI:** Atualização da ponte de UI (`AssetCard.tsx`) utilizando `img` restritas, `object-fit: contain` e calculadoras base para resgate dinâmico do viewBox SVG adaptando a altura responsiva do Grid conforme os painéis laterais retraem/expandem, garantindo nitidez High DPI.
- **Validação:** Ao realizar pinches no touchpad, zooms altos não geram artefatos ou quadriculados rasters nos formatos suportados, mantendo eficiência (pois é render nativo WebView).

### 2. Priorização Híbrida em Formatos Nativos (EPS, AI - Abobe)
- **Escopo:** Mitigar as quebras genéricas observadas de arquivos locais .ai e .eps em provedores de pré-visualização.
- **Ações:**
  - Extração profunda em Rust alterando os limites de cabeçalhos de extração bruta (`formats/definitions.rs` e `formats/mod.rs`). Implementar *Parsers* otimizados (como invocações de leituras passivas a trechos iniciados pro comentários nativos PDF dentro de .ai `.ai`/`.eps`), identificando os *embedded streams* (ex: versão PDF salva nativa 1.4+ do Illustrator) contidas no arquivo pai.
  - Caso detectado o container com PDF viável, enviar de forma imediata ao interpretador sem tentar rasterizá-lo grosseiramente ou recusar. Falhas em containers legados ou sem embed (arquivos sem preferência PDF salva no Photoshop/Illustrator) farão o *Tracing estruturado* cair educadamente pro erro sem crash em produção, provendo ícone de _"Thumbnail Indisponível"_.
- **Validação:** A plataforma torna-se apta a enxergar previews detalhadas de pranchas Adobe legadas diretamente do disco caso contenham sub-matrizes PDF em sua base. O pipeline log de erros do Tracing e do OTLP mapeará falhas crônicas para debug local.
