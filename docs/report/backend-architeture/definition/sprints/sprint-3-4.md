# Sprint 3.4: Extratores Especiais (RAW, Modelos 3D, SVG e ZIP)

**Status:** ✅ Concluído
**Data e hora de inicio:** 2026-03-06 20:30
**Data da conclusão:** 2026-03-08 14:35

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Consolidar a capacidade suprema do Mundam em devorar arquétipos restritos e de estúdios que geradores comuns repudiam. Converta as abstrações antigas de Pré-visualização ZIP, Fontes gráficas, Scanners e Models em Plugins Plug&Play de alta voltagem isolados contra falhas nucleares nos SDKs C++/Nativos.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. [x] **Resiliência Arquitetural (Isolamento de SDK):** Se a C-FFI ou a Binária acionada num Raw Camera Corrompido (`.CR3` / `DNG`) surtar, a falha retorna empírica um Rust `Err` formatado seguro encapsulada no `AppResult` e aborta só ela mesma. Nada morre por SegFault no banco global.
2. [x] **ZIP Preview:** Arquivos tipo `Clip Studio (.clip)` ou `CBZ/ZIP` abrem as Header em memória sem extrair pra um folder de TEMP no SSD, retornam o byte-array da miniatura XML e descartam a cópia (Stream Zip puro).
3. [x] **SVG & Fontes:** Resolução vetorizada atende perfeitamente ao pedido de Size_Hint: Pedir SVG pra escalar em Píxels 2000x2000 gera conversão impecável limpidamente sem artefatos.
4. [x] **Extração RAW em Camadas:** Sistema tenta LibRaw (Veloz), cai para Brute-Force Scanner (Resiliente) e finaliza no FFmpeg (Garantia) para garantir que nenhum RAW fique sem preview.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Provider de Raw Photography (Tiered Strategy)
- [x] Em `src-tauri/src/processing/media/raw_format.rs`, migre a lógica de `raw.rs`.
- [x] Implementar a estratégia de 3 camadas:
    1. **LibRaw** (`rsraw`): Extração de previews oficiais.
    2. **Brute-Force JPEG Scan**: Varredura binária por `FF D8 FF` nos primeiros 8MB (migrar de `raw.rs`).
    3. **FFmpeg Fallback**: Acionamento do modem de vídeo para RAWS que o Rust não entende.
- [x] Implementar `MetadataCapability` capturando ExifData (Modelos, ExposureTimes, F-Stops).

### 2. Provider de Projetos Arquivados (ZIP/CLIP)
- [x] Mapeie *Extensions*: `clip`, `zip`, `cbz`, etc...
- [x] A lógica do extractor de Thumbnail deve utilizar a crate nativa do `.zip/ArchiveType`, iterando somente pela árvore do Zip até achar `path/preview.png` no index, lendo diretamente (Zero Copies To Disk) da Stream em Memoria para entregar a Capability via `Ok(Vec)`.

### 3. Font e SVG Resolvers Vectoriais
- [x] Aportar bibliotecas Resvg/Skia/Ttf-Parsers da V1 na Capability, garantindo renderização de glifo em RAM baseado nas resoluções (`u32` limits) passadas pelo Worker limitando Memory Bloat.

### 4. Integração Definitiva no Cartório Central
- [x] Amarrar todos estes novos construtores (`RawFormatProvider`, `ArchiveFormatProvider`, `FontFormatProvider`) na lista de Bindings Globais da Inicialização App do Main logo ACIMA do `Fallback FFmpeg` que fizemos na aba anterior para que sejam roteados perante precedência de qualidade.

---

## 🛠️ Informações da Implementação

### Dificuldades Encontradas e Soluções
1. **Incompatibilidade de API (SDKs):** Crates como `ttf-parser` e `usvg/resvg` tiveram mudanças significativas em suas APIs de 2024 para 2025.
   - *Solução:* Foi necessário ajustar o uso de iteradores no `ttf-parser` (uso de loops explícitos e verificação de `name_id`) e métodos de escala no `usvg` (`scale_to` em vez de `scale_to_fit`).
2. **Visibilidade de Utilitários:** O helper `process_and_encode_webp` foi inicialmente privado ao `raw_format.rs`, impossibilitando o reuso pelo provedor de Arquivos.
   - *Solução:* Refatorado para `pub(crate)` permitindo compartilhamento seguro entre módulos de processamento sem expor para o resto da aplicação.
3. **Gerenciamento de Memória RAW:** RAWS grandes estavam falhando em parsers binários simples.
   - *Solução:* Implementado `memmap2` para garantir que o acesso aos dados binários do RAW seja performático e não estoure o stack do tokio.

### Melhorias Além do Escopo
1. **Shared WebP Encoding:** Centralização da lógica de conversão e resize no `raw_format.rs`, garantindo que todos os provedores técnicos sigam o mesmo padrão de qualidade e compressão.
2. **Documentação TSDoc/RustDoc:** Adição de comentários técnicos em todos os novos provedores seguindo as Guidelines do projeto.

---

## 📂 Arquivos Modificados
- `src-tauri/Cargo.toml` (Adição de `ttf-parser`, `rsraw`, etc)
- `src-tauri/src/processing/media/mod.rs` (Exportação de novos módulos)
- `src-tauri/src/core/formats/mod.rs` (Registro dos provedores no FormatRegistry)
- `src-tauri/src/processing/media/raw_format.rs` (Implementação RAW Tiered) [NEW]
- `src-tauri/src/processing/media/archive_format.rs` (Provedor ZIP/CLIP) [NEW]
- `src-tauri/src/processing/media/font_format.rs` (Provedor Fontes) [NEW]
- `src-tauri/src/processing/media/svg_format.rs` (Refactor de escalonamento SVG)

---

## 💡 Notas para o Desenvolvedor / Agente
> Você está transpondo códigos sensíveis que lidam com ponteiros e dependências cruzadas massivas. Não se acanhe em forçar MultiThreading Pools isoladas do `tokio::spawn_blocking` para os Renders pesados 3D ou SVG, evitando travar toda a mainloop e travar comunicação RPC (IPC) local. Um Raw de 150MB é lento para parsear; não engasgue a query pool do SQlite aguardando promessas!
