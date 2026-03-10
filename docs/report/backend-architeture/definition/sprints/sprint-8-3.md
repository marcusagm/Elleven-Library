# Sprint 8.3: Auditoria de FormatProviders e Cobertura de Extratores Especializados

**Status:** Pendente
**Data e hora de inicio:** -  
**Data da conclusão:** -

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
- [ ] Abrir `Mundam-main/src-tauri/src/formats/definitions.rs` e listar todas as extensões do `SUPPORTED_FORMATS`.
- [ ] Abrir `Mundam-main/src-tauri/src/thumbnails/extractors/mod.rs` e listar todos os extractors registrados.

### 2. Mapear FormatProviders V2 existentes
- [ ] Listar todos os arquivos em `Mundam/src-tauri/src/processing/media/`:
  - `image_format.rs` → Quais extensões cobre?
  - `video_format.rs` → Quais extensões cobre?
  - `psd_format.rs` → PSD, PSB
  - `svg_format.rs` → SVG, SVGZ
  - `font_format.rs` → TTF, OTF, WOFF, WOFF2
  - `icon_format.rs` → ICO, ICNS
  - `raw_format.rs` → CR2, NEF, ARW, DNG, etc.
  - `pdf_format.rs` → PDF
  - `model3d_format.rs` → OBJ, FBX, GLTF, GLB, STL, etc.
  - `affinity_format.rs` → AFDESIGN, AFPHOTO, AFPUB
  - `archive_format.rs` → ZIP, CBZ, CBR
  - `ai_format.rs` → AI
  - `aseprite_format.rs` → ASEPRITE, ASE
  - `audio_format.rs` → MP3, WAV, FLAC, OGG, etc.
  - `modern_image_format.rs` → HEIC, AVIF, JXL
  - `exr_format.rs` → EXR
  - `usd_format.rs` → USD, USDA, USDC, USDZ
  - `cad_format.rs` → DWG, DXF
  - `xmind_format.rs` → XMIND
  - `binary_design_formats.rs` → ???
  - `project_zip_formats.rs` → ???
  - `fallback_format.rs` → catchall

### 3. Auditar `binary_design_formats.rs` e `project_zip_formats.rs`
- [ ] Ler o conteúdo de ambos e listar exatamente quais formatos cobrem.
- [ ] Cross-reference com V1 extractors:
  - `sai.rs` → SAI (PaintTool SAI)
  - `sai2.rs` → SAI2 (PaintTool SAI 2)
  - `rebelle.rs` → REB (Rebelle)
  - `corel_painter.rs` → RIFF (Corel Painter)
  - `coreldraw.rs` → CDR (CorelDRAW)
  - `sketch.rs` → SKETCH (Bohemian Sketch)
  - `penpot.rs` → PENPOT
  - `mdp.rs` → MDP (MediBang Paint)
  - `eps.rs` → EPS (Encapsulated PostScript)
  - `xcf.rs` → XCF (GIMP)
  - `clip.rs` → CLIP (CLIP Studio Paint)
  - `binary_jpeg.rs` → Extractor genérico para formatos com JPEG embutido

### 4. Implementar FormatProviders faltantes
- [ ] Para cada formato V1 **NÃO** coberto no V2:
  1. Criar novo FormatProvider OU adicionar extensão a um provider existente.
  2. Implementar `ThumbnailCapability` usando a mesma lógica do extractor V1 correspondente.
  3. Registrar no `core/formats/mod.rs` → `build_format_registry()`.
- [ ] **Referência V1:** Cada arquivo em `Mundam-main/src-tauri/src/thumbnails/extractors/*.rs` contém a lógica de extração de thumbnail para seu formato.

### 5. Verificar `build_format_registry()`
- [ ] Em `Mundam/src-tauri/src/core/formats/mod.rs`, confirmar que TODOS os providers estão registrados.
- [ ] A ordem de registro importa: providers específicos PRIMEIRO, fallbacks DEPOIS.

### 6. Testes de Resolução
- [ ] Para cada formato, simular `registry.resolve(Path::new("test.<ext>"), &[])` e confirmar que o provider correto é retornado.
- [ ] Testar fallback para extensões desconhecidas.

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
- 

### Melhorias Realizadas
- 

### 📄 Arquivos Criados ou Modificados
- 
