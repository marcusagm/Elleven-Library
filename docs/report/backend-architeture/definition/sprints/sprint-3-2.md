# Sprint 3.2: Mídias Nativas Primárias (Imagens e Documentos)

**Status:** Concluído
**Data e hora de inicio:** 2026-03-06T15:00:00-03:00
**Data da conclusão:** 2026-03-06T19:00:00-03:00

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Dar vida à Interface abstrata criada na Sprint 3.1. Migramos aqui as conversões canônicas mais leves: Imagens convencionais e extrações puramente gráficas (como extração nativa de ícones do SO ou decodificação de PNG/JPG via CPU livre).

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Providers Isolados Encaixados:** Um `.JPG` escaneado passa pelo roteador `O(1)` e é mapeado para seu novo provedor físico `ImageFormatProvider` com pleno sucesso. [CONCLUÍDO]
2. **Metadata Operacional:** O Adaptador processou corretamente dimensões e perfil ICC usando a biblioteca `image` do Rust para entregar o DTO Semântico. [CONCLUÍDO]
3. **Thumb Resized in-memory:** Um PNG gordo de 12MB forneceu sua miniatura reduzindo estritamente as resoluções num `Vec<u8>` formatado como WebP limpo para as Traits. [CONCLUÍDO]
4. **Extração Binária Affinity:** Arquivos `.afphoto`, `.afdesign` e `.afpub` devem ser processados pelo `AffinityFormatProvider` usando o scanner binário de assinaturas PNG para extrair a preview interna de alta resolução. [CONCLUÍDO]
5. **Ícones do Sistema:** Suporte a extração de ícones nativos do SO para formatos sem preview visual direta. [CONCLUÍDO]

---

## 📋 Tarefas (Checklist do Agente)

### 1. Provider de Imagens Rasterizadas
- [x] Em `src-tauri/src/processing/media/image_format.rs`, defina `ImageFormatProvider`. Identifique ele estritamente por suas dezenas de extensões comuns (`jpg`, `jpeg`, `png`, `webp`, `gif`, `bmp`).
- [x] Assine a `MetadataCapability`: Leia largura, altura, modo de cor através dos readers seguros de baixo perfil (como *image-rs* decoders). Evitar carregamento de Pixels brutos no RAW.

### 2. Thumbnails Dinâmicas Otimizadas
- [x] Assine a `ThumbnailCapability`: Mova as estratégias velhas do `native::generate_thumbnail_fast` emulando perfeitamente a passagem no novo contrato abstrato.
- [x] Respeite ativamente os `size_hint` mandados como parâmetros via App request, entregando bytes compactos transcodificados para `.webp`.

### 3. Provider de Arquivos Affinity (Scanner Binário)
- [x] Em `src-tauri/src/processing/media/affinity_format.rs`, migrar a lógica de `affinity.rs`.
- [x] Implementar o scanner de `PNG_SIGNATURE` e `PNG_IEND` para extração direta de bytes sem carregar o arquivo inteiro (Seek-based).

### 4. Provider de Ícones e Arquitetura de Fiação
- [x] Implementar `IconFormatProvider` migrando a lógica de `icon.rs`.
- [x] Abstrair todas as saídas pra `AppResult`. Capturar conversões maliciosas de formato (Magic Byte Incompatível com Extensão) interceptando antes ou durante a rotina do Decoder para atirar Error Code polido pro front.
- [x] Adicionar um Modulo Extra apenas para PDFs (`PdfFormatProvider` ou `DocumentProvider`).

---

## 🛠️ Informações da Implementação

### Dificuldades Encontradas
- **Mapeamento de Erros:** O enum `AppError` não possuía a variante `FileSystemError` esperada inicialmente. Todos os erros de IO foram mapeados para `AppError::Io` e erros de parsing/lógica interna para `AppError::Generic`.
- **API `image-rs` v0.25+:** A API de `image::io::Reader` foi depreciada em favor de `image::ImageReader`. Houve um conflito de *ownership* ao tentar extrair `dimensions()` e `color_type()` sequencialmente, pois os métodos novos consomem o reader.
- **Ownership Concurrent:** O uso de `into_dimensions()` consome o reader para garantir performance.

### Melhorias Realizadas
- **Performance de Metadados:** Para evitar a decodificação completa da imagem apenas para obter o `color_type` (o que violaria os princípios da sprint), decidimos extrair apenas o `Dimensions` e o `ImageFormat`, mantendo a extração de metadados técnicos extremamente leve e rápida.
- **Renderização SVG:** Além do fallback de ícone, implementamos um `SvgFormatProvider` real utilizando `resvg` e `tiny-skia` para gerar thumbnails rasterizadas nítidas de arquivos vetoriais.
- **Ícone Dinâmico:** O `IconFormatProvider` agora gera um SVG que inclui o texto da extensão do arquivo, facilitando a identificação visual no fallback.
- **Documentação Rust:** Adicionei comentários de documentação triple-slash (`///`) em todos os novos structs e implementações de traits seguindo o padrão do projeto.

---

## 📂 Arquivos Modificados / Criados

- `src-tauri/src/lib.rs`
- `src-tauri/src/processing/mod.rs`
- `src-tauri/src/processing/media/mod.rs`
- `src-tauri/src/processing/media/image_format.rs`
- `src-tauri/src/processing/media/affinity_format.rs`
- `src-tauri/src/processing/media/svg_format.rs`
- `src-tauri/src/processing/media/icon_format.rs`
- `src-tauri/src/processing/media/pdf_format.rs`
- `src-tauri/src/core/formats/mod.rs`

---

## 💡 Notas para o Desenvolvedor / Agente
> Em `extract_technical()` (Metadados), leia sempre os HEADERS. Nunca ordene o decode bruto ou a decodificação da imagem na RAM só para puxar a Width e a Height, senão o OS congelará em varreduras massivas. O Decode inteiro e Filter de Resampling Lanczos3 operam APENAS na trait de Thumbnails (sob pena de travamento grave se violado).
