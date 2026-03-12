# Sprint 8.3: Auditoria de FormatProviders e Cobertura de Extratores Especializados

**Status:** Concluído
**Data e hora de inicio:** 2026-03-11 23:00  
**Data da conclusão:** 2026-03-12 01:10

**Fase 8:** Paridade IPC — Mídia, Manutenção e Utilidades
**Objetivo:** Garantir que TODOS os formatos de arquivo suportados pelo V1 estejam cobertos por FormatProviders no V2 com capabilities corretas. Auditar os extractors especializados (SAI, SAI2, Rebelle, CorelDRAW, CorelPainter, Sketch, Penpot, MDP, EPS, XCF, CLIP) e confirmar que cada um possui `ThumbnailCapability` funcional no V2.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. Todo formato listado em `Mundam-main/src-tauri/src/formats/definitions.rs` tem um FormatProvider correspondente no V2.
2. Todo extractor em `Mundam-main/src-tauri/src/thumbnails/extractors/` tem lógica equivalente em algum FormatProvider V2.
3. O FormatRegistry V2 resolve corretamente extensões de TODOS os formatos do V1.
4. Nenhum formato "cai no fallback" do V2 que era suportado explicitamente no V1.
5. `cargo build` compila sem warnings.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Listar TODOS os formatos V1
- [x] Abrir `Mundam-main/src-tauri/src/formats/definitions.rs` e listar todas as extensões do `SUPPORTED_FORMATS`.
- [x] Abrir `Mundam-main/src-tauri/src/thumbnails/extractors/mod.rs` e listar todos os extractors registrados.

### 2. Mapear FormatProviders V2 existentes
- [x] Listar todos os arquivos em `Mundam/src-tauri/src/processing/media/`:
  - `image_format.rs` → Quais extensões cobre? (JPG, PNG, WebP, GIF, BMP, ICO, TIFF, HDR, DDS, PBM, PGM, PPM, PNM, PAM)
  - `video_format.rs` → Quais extensões cobre? (MP4, MKV, AVI, MOV, WMV, WEBM, FLV, M4V, MTS)
  - `psd_format.rs` → PSD, PSB
  - `svg_format.rs` → SVG, SVGZ
  - `font_format.rs` → TTF, OTF, WOFF, WOFF2
  - `icon_format.rs` → ICO, ICNS
  - `raw_format.rs` → CR2, NEF, ARW, DNG, etc.
  - `pdf_format.rs` → PDF
  - `model3d_format.rs` → OBJ, FBX, GLTF, GLB, STL, etc.
  - `affinity_format.rs` → AFDESIGN, AFPHOTO, AFPUB
  - `archive_format.rs` → ZIP, CBZ, CBR, CLIP
  - `ai_format.rs` → AI, EPS
  - `aseprite_format.rs` → ASEPRITE, ASE
  - `audio_format.rs` → MP3, WAV, FLAC, OGG, AAC, M4A, WMA
  - `modern_image_format.rs` → HEIC, AVIF, JXL
  - `exr_format.rs` → EXR
  - `usd_format.rs` → USD, USDA, USDC, USDZ
  - `cad_format.rs` → DWG, DXF
  - `xmind_format.rs` → XMIND
  - `binary_design_formats.rs` → SAI, SAI2, XCF, RIFF (Painter), MDP, CorelDRAW
  - `project_zip_formats.rs` → Sketch, Rebelle, Penpot
  - `fallback_format.rs` → catchall

### 3. Auditar `binary_design_formats.rs` e `project_zip_formats.rs`
- [x] Ler o conteúdo de ambos e listar exatamente quais formatos cobrem.
- [x] Cross-reference com V1 extractors:
  - [x] `sai.rs` → SAI (PaintTool SAI)
  - [x] `sai2.rs` → SAI2 (PaintTool SAI 2)
  - [x] `rebelle.rs` → REB (Rebelle)
  - [x] `corel_painter.rs` → RIFF (Corel Painter)
  - [x] `coreldraw.rs` → CDR (CorelDRAW)
  - [x] `sketch.rs` → SKETCH (Bohemian Sketch)
  - [x] `penpot.rs` → PENPOT
  - [x] `mdp.rs` → MDP (MediBang Paint)
  - [x] `eps.rs` → EPS (Encapsulated PostScript)
  - [x] `xcf.rs` → XCF (GIMP)
  - [x] `clip.rs` → CLIP (CLIP Studio Paint)
  - [x] `binary_jpeg.rs` → Extractor genérico para formatos com JPEG embutido

### 4. Implementar FormatProviders faltantes
- [x] Para cada formato V1 **NÃO** coberto no V2:
  1. Criar novo FormatProvider OU adicionar extensão a um provider existente.
  2. Implementar `ThumbnailCapability` usando a mesma lógica do extractor V1 correspondente.
  3. Registrar no `core/formats/mod.rs` → `build_format_registry()`.
- [x] **Referência V1:** Cada arquivo em `Mundam-main/src-tauri/src/thumbnails/extractors/*.rs` contém a lógica de extração de thumbnail para seu formato.

### 5. Verificar `build_format_registry()`
- [x] Em `Mundam/src-tauri/src/core/formats/mod.rs`, confirmar que TODOS os providers estão registrados.
- [x] A ordem de registro importa: providers específicos PRIMEIRO, fallbacks DEPOIS.

### 6. Testes de Resolução
- [x] Para cada formato, simular `registry.resolve(Path::new("test.<ext>"), &[])` e confirmar que the provider correto é retornado.
- [x] Testar fallback para extensões desconhecidas.

---

## 📁 Arquivos de Referência V1

| Extractor V1    | Arquivo V1 (Mundam-main)                               | Formato | Técnica                      |
| --------------- | ------------------------------------------------------ | ------- | ---------------------------- |
| SAI             | `src-tauri/src/thumbnails/extractors/sai.rs`           | .sai    | Binary offset extraction     |
| SAI2            | `src-tauri/src/thumbnails/extractors/sai2.rs`          | .sai2   | SQLite embedded thumbnail    |
| Rebelle         | `src-tauri/src/thumbnails/extractors/rebelle.rs`       | .reb    | ZIP archive extraction       |
| CorelPainter    | `src-tauri/src/thumbnails/extractors/corel_painter.rs` | .riff   | RIFF header parsing          |
| CorelDRAW       | `src-tauri/src/thumbnails/extractors/coreldraw.rs`     | .cdr    | Binary header                |
| Sketch          | `src-tauri/src/thumbnails/extractors/sketch.rs`        | .sketch | ZIP archive preview          |
| Penpot          | `src-tauri/src/thumbnails/extractors/penpot.rs`        | .penpot | ZIP archive                  |
| MDP             | `src-tauri/src/thumbnails/extractors/mdp.rs`           | .mdp    | Binary JPEG extraction       |
| EPS             | `src-tauri/src/thumbnails/extractors/eps.rs`           | .eps    | Ghostscript/embedded preview |
| XCF             | `src-tauri/src/thumbnails/extractors/xcf.rs`           | .xcf    | GIMP native parsing          |
| CLIP            | `src-tauri/src/thumbnails/extractors/clip.rs`          | .clip   | SQLite embedded thumb        |
| BinaryJPEG      | `src-tauri/src/thumbnails/extractors/binary_jpeg.rs`   | Generic | Magic byte JPEG search       |
| All definitions | `src-tauri/src/formats/definitions.rs`                 | All     | Static FORMAT array          |

## 📁 Arquivos a Modificar no V2

| Arquivo V2 (Mundam)                                         | Ação                      |
| ----------------------------------------------------------- | ------------------------- |
| `src-tauri/src/processing/media/binary_design_formats.rs`   | Auditar cobertura         |
| `src-tauri/src/processing/media/project_zip_formats.rs`     | Auditar cobertura         |
| `src-tauri/src/processing/media/*.rs` (novos se necessário) | Novos FormatProviders     |
| `src-tauri/src/core/formats/mod.rs`                         | Registrar novos providers |

---

## 💡 Notas para o Desenvolvedor / Agente
> **CRITÉRIO DE COMPLETUDE:** Esta sprint só está "Done" quando CADA extensão listada em `definitions.rs` do V1 resolve para um FormatProvider no V2 via `registry.resolve()`. Use o `definitions.rs` do V1 como checklist definitiva.

> No V2, formatos com técnicas similares são agrupados em `binary_design_formats.rs` (formatos que usam busca de JPEG binário embutido) e `project_zip_formats.rs` (formatos baseados em ZIP com preview dentro). Isso é SUPERIOR ao V1 onde cada um tinha seu arquivo separado. MAS verifique que a lógica específica de cada formato está preservada.

> **NÃO REMOVA o `fallback_format.rs`** — ele é necessário para formatos desconhecidos que aparecem no futuro.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- **Refatoração Granular**: O maior desafio foi a transição de providers genéricos (ex: `RASTER_IMAGE_PROVIDER`) para definições granulares (`JPEG Image`, `PNG Image`). Isso exigiu a refatoração do trait `FormatProvider` e da lógica do `FormatRegistry`.
- **Borrow Checker**: Encontramos conflitos de borrow mútuo em funções recursivas (especialmente `walk_riff` no `coreldraw.rs`), resolvidos com a extração de estados intermediários.
- **Imports e Dependências**: A migração de extractores V1 para V2 revelou dependências ausentes e tipos de retorno incompatíveis que exigiram ajustes finos em tempo de compilação.
- **Paridade Semântica vs. Melhoria**: A introdução de novos tipos como `MediaType::Vector` no V2 causou um "desvio de paridade" com o frontend, que esperava `MediaType::Image` para formatos como SVG e PDF (seguindo o padrão V1). Foi necessário reverter essas classificações para garantir compatibilidade imediata, priorizando a estabilidade da migração sobre a taxonomia ideal.

### Melhorias Realizadas
- **Arquitetura de Metadados Ricos**: Integração de `MediaType`, `PreviewStrategy` e `PlaybackStrategy` diretamente na estrutura `SupportedFormat`, permitindo que o frontend tome decisões inteligentes de UI (ex: exibir player de vídeo vs. galeria de imagens) sem lógica duplicada no cliente.
- **Arquitetura de Formatos**: Implementado o método `supported_formats()` no trait `FormatProvider`, permitindo que o backend saiba exatamente quais formatos lógicos cada provider gerencia, melhorando a precisão do comando `get_library_supported_formats`.
- **Limpeza de Código**: Toda a lógica de extração foi centralizada em `processing/media/extractors/`, seguindo padrões de clean-code e eliminando dead-code.
- **Detecção Robusta**: Melhorada a validação de magic bytes para formatos complexos como Aseprite e CorelDRAW.

### 📄 Arquivos Criados ou Modificados

#### Core / Formats
- `src-tauri/src/core/formats/provider.rs` (Refatoração do trait)
- `src-tauri/src/core/formats/registry.rs` (Refatoração da resolução)
- `src-tauri/src/core/formats/mod.rs` (Registro de extractores)
- `src-tauri/src/core/formats/types.rs` (Enums de estratégia e tipos de mídia - Paridade V1)
- `src-tauri/src/core/error.rs` (Novos tipos de erro de extração)

#### Extractors (Portados/Refinados)
- `src-tauri/src/processing/media/extractors/mod.rs`
- `src-tauri/src/processing/media/extractors/ai.rs`
- `src-tauri/src/processing/media/extractors/aseprite.rs`
- `src-tauri/src/processing/media/extractors/binary_jpeg.rs`
- `src-tauri/src/processing/media/extractors/clip.rs`
- `src-tauri/src/processing/media/extractors/corel_painter.rs`
- `src-tauri/src/processing/media/extractors/coreldraw.rs`
- `src-tauri/src/processing/media/extractors/eps.rs`
- `src-tauri/src/processing/media/extractors/mdp.rs`
- `src-tauri/src/processing/media/extractors/penpot.rs`
- `src-tauri/src/processing/media/extractors/rebelle.rs`
- `src-tauri/src/processing/media/extractors/sai.rs`
- `src-tauri/src/processing/media/extractors/sai2.rs`
- `src-tauri/src/processing/media/extractors/sketch.rs`
- `src-tauri/src/processing/media/extractors/xcf.rs`

#### Providers (Implementação Granular e Metadados)
- `src-tauri/src/processing/media/ai_format.rs` (Ajustado para paridade V1)
- `src-tauri/src/processing/media/aseprite_format.rs`
- `src-tauri/src/processing/media/binary_design_formats.rs`
- `src-tauri/src/processing/media/project_zip_formats.rs`
- `src-tauri/src/processing/media/audio_format.rs`
- `src-tauri/src/processing/media/video_format.rs`
- `src-tauri/src/processing/media/image_format.rs`
- `src-tauri/src/processing/media/psd_format.rs`
- `src-tauri/src/processing/media/pdf_format.rs` (Ajustado para paridade V1)
- `src-tauri/src/processing/media/raw_format.rs`
- `src-tauri/src/processing/media/font_format.rs`
- `src-tauri/src/processing/media/svg_format.rs` (Ajustado para paridade V1)
- `src-tauri/src/processing/media/xmind_format.rs`
- `src-tauri/src/processing/media/usd_format.rs` (Corrigido para Model3D)
- `src-tauri/src/processing/media/exr_format.rs`
- `src-tauri/src/processing/media/cad_format.rs` (Corrigido para Model3D)
- `src-tauri/src/processing/media/model3d_format.rs` (Corrigido para Model3D)
- `src-tauri/src/processing/media/affinity_format.rs`
- `src-tauri/src/processing/media/archive_format.rs`
- `src-tauri/src/processing/media/icon_format.rs` (Auditoria de metadados)
