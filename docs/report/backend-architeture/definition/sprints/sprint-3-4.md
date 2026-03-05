# Sprint 3.4: Extratores Especiais (RAW, Modelos 3D, SVG e ZIP)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Consolidar a capacidade suprema do Mundam em devorar arquétipos restritos e de estúdios que geradores comuns repudiam. Converta as abstrações antigas de Pré-visualização ZIP, Fontes gráficas, Scanners e Models em Plugins Plug&Play de alta voltagem isolados contra falhas nucleares nos SDKs C++/Nativos.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Resiliência Arquitetural (Isolamento de SDK):** Se a C-FFI ou a Binária acionada num Raw Camera Corrompido (`.CR3` / `DNG`) surtar, a falha retorna empírica um Rust `Err` formatado seguro encapsulada no `AppResult` e aborta só ela mesma. Nada morre por SegFault no banco global.
2. **ZIP Preview:** Arquivos tipo `Clip Studio (.clip)` ou `CBZ/ZIP` abrem as Header em memória sem extrair pra um folder de TEMP no SSD, retornam o byte-array da miniatura XML e descartam a cópia (Stream Zip puro).
3. **SVG & Fontes:** Resolução vetorizada atende perfeitamente ao pedido de Size_Hint: Pedir SVG pra escalar em Píxels 2000x2000 gera conversão impecável limpidamente sem artefatos.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Provider de Raw Photography
- [ ] Em `processing/media/raw_camera_format.rs`, migre a chamada a bibliotecas/CLI nativas delegadas exclusivamente para as file-extensions da pesada (`.cr2`, `.nef`, `.arw`).
- [ ] Lide ativamente no Metadata_Extract pegando ExifData (Modelos, ExposureTimes, F-Stops) formatando de volta num Struct coerente.

### 2. Provider de Projetos Arquivados (ZIP/CLIP)
- [ ] Mapeie *Extensions*: `clip`, `zip`, `cbz`, etc...
- [ ] A lógica do extractor de Thumbnail deve utilizar a crate nativa do `.zip/ArchiveType`, iterando somente pela árvore do Zip até achar `path/preview.png` no index, lendo diretamente (Zero Copies To Disk) da Stream em Memoria para entregar a Capability via `Ok(Vec)`.

### 3. Font e SVG Resolvers Vectoriais
- [ ] Aportar bibliotecas Resvg/Skia/Ttf-Parsers da V1 na Capability, garantindo renderização de glifo em RAM baseado nas resoluções (`u32` limits) passadas pelo Worker limitando Memory Bloat.

### 4. Integração Definitiva no Cartório Central
- [ ] Amarrar todos estes novos construtores (`PsdFormatProvider`, `FontFormatProvider`) na lista de Bindings Globais da Inicialização App do Main logo ACIMA do `Fallback FFmpeg` que fizemos na aba anterior para que sejam roteados perante precedência de qualidade.

---

## 💡 Notas para o Desenvolvedor / Agente
> Você está transpondo códigos sensíveis que lidam com ponteiros e dependências cruzadas massivas. Não se acanhe em forçar MultiThreading Pools isoladas do `tokio::spawn_blocking` para os Renders pesados 3D ou SVG, evitando travar toda a mainloop e travar comunicação RPC (IPC) local. Um Raw de 150MB é lento para parsear; não engasgue a query pool do SQlite aguardando promessas!
