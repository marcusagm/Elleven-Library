# Sprint 3.2: Mídias Nativas Primárias (Imagens e Documentos)

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Dar vida à Interface abstrata criada na Sprint 3.1. Migramos aqui as conversões canônicas mais leves: Imagens convencionais e extrações puramente gráficas (como extração nativa de ícones do SO ou decodificação de PNG/JPG via CPU livre).

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Providers Isolados Encaixados:** Um `.JPG` escaneado passa pelo roteador `O(1)` e é mapeado para seu novo provedor físico `ImageFormatProvider` com pleno sucesso.
2. **Metadata Operacional:** O Adaptador processou corretamente dimensões e perfil ICC usando a biblioteca `image` do Rust para entregar o DTO Semântico.
3. **Thumb Resized in-memory:** Um PNG gordo de 12MB forneceu sua miniatura reduzindo estritamente as resoluções num `Vec<u8>` formatado como WebP limpo para as Traits.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Provider de Imagens Rasterizadas
- [ ] Em `src-tauri/src/processing/media/image_format.rs`, defina `ImageFormatProvider`. Identifique ele estritamente por suas dezenas de extensões comuns (`jpg`, `jpeg`, `png`, `webp`, `gif`, `bmp`).
- [ ] Assine a `MetadataCapability`: Leia largura, altura, modo de cor através dos readers seguros de baixo perfil (como *image-rs* decoders). Evitar carregamento de Pixels brutos no RAW.

### 2. Thumbnails Dinâmicas Otimizadas
- [ ] Assine a `ThumbnailCapability`: Mova as estratégias velhas do `native::generate_thumbnail_fast` emulando perfeitamente a passagem no novo contrato abstrato.
- [ ] Respeite ativamente os `size_hint` mandados como parâmetros via App request, entregando bytes compactos transcodificados para `.webp`.

### 3. Arquitetura de Fiação
- [ ] Abstrair todas as saídas pra `AppResult`. Capturar conversões maliciosas de formato (Magic Byte Incompatível com Extensão) interceptando antes ou durante a rotina do Decoder para atirar Error Code polido pro front.
- [ ] Adicionar um Modulo Extra apenas para PDFs (`PdfFormatProvider` ou `DocumentProvider` se o preview nativamente exportar páginas vetorizadas do OS).

---

## 💡 Notas para o Desenvolvedor / Agente
> Em `extract_technical()` (Metadados), leia sempre os HEADERS. Nunca ordene o decode bruto ou a decodificação da imagem na RAM só para puxar a Width e a Height, senão o OS congelará em varreduras massivas. O Decode inteiro e Filter de Resampling Lanczos3 operam APENAS na trait de Thumbnails (sob pena de travamento grave se violado).
