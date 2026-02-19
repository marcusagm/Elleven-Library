# Implementação de Suporte a Arquivos `.penpot` (Penpot)

**Data:** 19 de Fevereiro de 2026  
**Autor:** Antigravity (Assistente de IA)

---

## 📌 Contexto
O objetivo desta tarefa foi adicionar suporte para extração de miniaturas (thumbnails) e visualizações rápidas de arquivos do software de design open-source, **Penpot** (`.penpot`). O desenvolvimento foi centralizado em uma estratégia de backend robusta operada inteiramente em Rust, seguindo as diretrizes do `backend-rust.md`.

Arquivos Penpot atualmente existem em dois formatos fundamentalmente distintos de armazenamento:
1. **V1 (Legacy/Standard):** Um contêiner padrão `ZIP` onde os recortes/miniaturas das `artboards` são exportados fisicamente na camada de `objects/` do arquivo em formato `.png`.
2. **V2 (Modern/Optimized):** Uma única _stream_ ou "blob" serializado usando tecnologias derivadas do Clojure (Transit-MP / Nigiri) e comprimidos com o algoritmo **Zstandard (zstd)** visando ganhos absurdos de espaço para *UI Kits* complexos.

---

## 🚧 Passo a Passo da Implementação

### 1. Pesquisa e Arquitetura do Extrator
Ao iniciar o trabalho, avaliamos três estratégias diferentes (Extrator Nativo Total, Extrator Nativo Parcial ou Renderização Dinâmica via WebView). Optamos pela criação de um módulo 100% autônomo e nativo no backend Rust.
- Criamos o arquivo isolado `src-tauri/src/thumbnails/extractors/penpot.rs`.

### 2. Integração da Dependência `zstd`
Para permitir que o ecossistema backend em Rust suportasse nativamente a descompressão do formato Penpot V2, implementamos a crate oficial da compressão:
- Comando de instalação da biblioteca `cargo add zstd`.

### 3. Escrevendo o Código de Extração
No módulo principal `penpot.rs`:
- Lemos e inferimos os 4-bytes contendo o [*Magic Byte Signature*](https://en.wikipedia.org/wiki/List_of_file_signatures) para diferenciar um ZIP normal ($50 4B 03 04$) do Cabeçalho Penpot de Zstd ($01 0B 1A 86$).
- **V1 (ZIP):** Utilizamos iteração do pacote `zip` por todos os items e aplicamos a lógica de seleção buscando a maior de todas as imagens em tamanho real dentro do caminho interno `objects/`. É extremamente eficaz e resgatou a Thumb certinha no exemplo de teste `Eisenhower Matrix` e `minimalist-wireframing-kit`.
- **V2 (Zstd):** Deslocamos a leitura (um `seek`) ignorando os 17 bytes de cabeçalho do arquivo em disco (onde ficam a assinatura e versão), alocando a stream comprimida pelo Decoder dinâmico do Zstd e isolando em um *buffer* num limite seguro de memória RAM (50MB) para prevenir problemas com estouros em painéis Penpot colossais da web.
- Finalmente, programamos uma rotina manual na função `extract_largest_png_from_buffer` encarregada de iterar por todos os blocos _(Chunks)_ e capturar diretamente bytes com assinaturas de cabeçalho `\x89PNG` e fim `IEND`.

### 4. Configuração do Modulo Global Tauri
* Em `formats/definitions.rs`: Adicionamos o `Penpot Project` com extensões `["penpot"]` apontando o `ThumbnailStrategy::NativeExtractor`.
* Em `thumbnails/extractors/mod.rs`: Mapeamos o nome `penpot` encaminhando para nossa nova função `extract_penpot_preview` e injetando as rotinas seguras que convertem tudo para `"image/png"`.
* Em `thumbnails/mod.rs`: Desabilitamos as tentativas desnecessárias e custosas do FFmpeg para a extensão `.penpot`, acelerando muito os processamentos.

---

## 🛑 Obstáculos Encontrados

Durante os testes automatizados via `cargo test` para validarmos os Extratores criados, percebemos que **arquivos formatados no V2 ("Cartas Creativas") estavam retornando a falta de Imagens no payload**. 
- Criamos e rodamos rotinas provisórias dinâmicas em `Python` sob ponteiro binário, caçando por todos os *magic bytes* conhecidos pela indústria global (`PNG`, `JPEG`, e marcações Base64 Data-URI do próprio `SVG/HTML`) dentro do código Zstd descomprimido em _buffer_ total de memória.
- **Veredito Técnico:** O exportador da plataforma web Penpot salva *apenas vetores serializados via Lisp* nesses novos arquivos Otimizados (V2), visando performance. **Não há representações de arte realística ou miniaturas rasterizadas (como JPG/PNG) armazenados fisicamente de maneira alguma no arquivo**.
- Isso tornaria virtualmente impossível construir o *raster* das miniaturas por leitura binária básica, apenas pelo recálculo da lógica de interpretação React/Clojure completa.

### ✓ Solução do Obstáculo
Mapeamos o comportamento do Extrator `extract_v2_zstd_preview()` para retornar um Erro limpo sempre que a rotina otimista não achasse imagens base. Esse err aciona o gatilho principal de **Fallback do Mundam (Icon Strategy)** que passa automaticamente a mostrar o Ícone global SVG predefinido com o nome "Penpot Project" na galeria, prevenindo que o sistema engasgue.

---

## 🚀 Melhorias Futuras

1. **Proxy Webview Bridge (Renderização Headless):** Caso o suporte ao V2 da documentação Penpot evolua ou a comunidade decida priorizar as thumbs perfeitas nele, pode-se orquestrar uma passagem do Rust ativando ocultamente o WebView Nativo e passando o Stream completo para uma lib JS portando a renderização Lisp do Penpot, usando o motor Chromium local e enviando um DataBlob de volta para o Rust.
2. **Engatilhar Parsing Oficial de Metadados:** Agora que decodificamos Zstd e conhecemos o padrão de Chunks com arrays e dicionários, desenvolver um módulo serializador capaz de mapear a "hierarquia estrutural do Clojure Transit MP" diretamente para o banco sqlite do app Mundam (nomes das Artboards, autores etc).
3. **Mapeamento de Objetos Base64 Livres:** Apesar do formato primário para arquivos menores/padrão atualmente no V2 ser estritamente descritivo, as rotinas varredouras criadas podem ser ampliadas e re-exploradas periodicamente caso os contribuidores do ecossistema do Penpot integrem um bloco Base64 opcional com thumbnail do workspace futuro.
