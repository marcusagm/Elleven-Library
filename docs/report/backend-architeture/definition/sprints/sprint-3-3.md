# Sprint 3.3: Fallbacks Dinâmicos (Vídeos & FFmpeg)

**Status:** Concluído
**Data e hora de inicio:** 2026-03-06T19:03:36Z
**Data da conclusão:** 2026-03-06T20:00:00Z

**Fase 3:** O Format-Kit Registry (Extração Pura)
**Objetivo:** Abordar arquivos gigantescos, contêineres de vídeo e formatos desamparados. Esta sprint integra o poderoso CLI do FFmpeg devolta ao sistema como um `FormatProvider` puro e submisso ao ecossistema abstrato.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. [x] **FFmpeg Subprocessos Seguros:** A decodificação atira Threads independentes (Shell Command) sem congelar NUNCA o Backend Rust principal.
2. [x] **Metadata Técnica Completa:** FFprobe gera JSON limpo (Codec, Bitrate, Durations) para inserção no CQRS via Provider.
3. [x] **Keyframe Visual:** Geração de um Frame Snapshot em tempo determinado encapsulado sob interface padrão devolvendo um byte array WebP ou Jpg da prévia.
4. [x] **Formatos Modernos:** Suporte total a `HEIC`, `AVIF` e `JXL` via FFmpeg transcoding.
5. [x] **Waveforms de Áudio:** Extração de dados de amplitude (waveform) para arquivos de áudio via FFmpeg `f32le` stream.
6. [x] **Segurança de Processo:** Implementação de `wait_timeout` e `child.kill()` para prevenir processos zumbis.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Adapter do Vídeo
- [x] Em `processing/media/video_format.rs` instancie e declare dezenas de extensões multimídia genéricas suportadas.
- [x] Cheque a existência/validação empírica binária se o FFmpeg existe nas pastas binárias do Sistema antes da instanciação inicial, ou desista do Capability.

### 2. Extraction Limits via FFprobe
- [x] Na `MetadataCapability`, acionar o subprocesso de CLI invisível. Usar `serde_json::from_slice` nativo sobre o "stout" para não precisar parsear regex em strings de saídas nativas e dolorosas do FFprobe.

### 3. Frame Grabbing Customizado (ThumbnailCap)
- [x] Assinar a trait garantindo que buscar um frame utilize comandos duplos de performance: `ffmpeg -v quiet -ss <time> -i <input> -vframes 1 ...` (Jamais varrer o vídeo quadro a quadro do `0:00` pra pegar snapshot no segundo `10:00`).
- [x] Garantir o tratamento de HDR/EXR tonemapping (linear -> hable -> 709) conforme implementado no legado.

### 4. Audio Waveform Capability
- [x] Implementar extração de waveform for o `AudioFormatProvider` usando o pipeline de pipes do FFmpeg (`-ar 100 -ac 1 -f f32le`).

### 5. Magic Bytes puros e Segurança
- [x] Criar no fim do Bootstrap um "GenericByteFallbackProvider" que herda tudo que errou na Hash O(1) e força varredura de magic bytes sem emitir Metadata pra evitar perdas totais nos Indexers caso se depare com Mídias bizarras ocultas do sistema local e apenas atire "AppError::NoResolutionLimit" visuais pro Front.
- [x] Implementar o wrapper `run_command_with_timeout` para garantir que nenhum processo FFmpeg fique travado se o IO do disco ou o codec surtar.

---

## 🚀 Informações da Implementação

Durante a execução desta sprint, foram realizadas melhorias significativas em relação ao plano inicial:

1.  **Infraestrutura de Transcoding Avançada**: O módulo `processing/transcoding` foi desenhado para ser agnóstico a sistema operacional, resolvendo binários tanto em diretórios de recursos do Tauri quanto no PATH global, garantindo portabilidade.
2.  **Tratamento de Erros Robusto**: Introduzimos a variante `AppError::NoResolutionLimit` para lidar especificamente com arquivos identificados via magic bytes mas que não possuem processadores de conteúdo, evitando loops de erro no indexador.
3.  **Performance de Áudio**: A extração de waveform foi otimizada para processar streams f32le brutos do FFmpeg, reduzindo o volume de dados para 500 pontos normalizados ideais para renderização no Frontend.
4.  **Dificuldades Superadas**: Enfrentamos desafios com a mudança de API do Tauri v2 (necessidade do trait `Manager` para resolução de caminhos) e gerenciamento de ownership no Rust ao executar comandos de retry em vídeos curtos ou corrompidos. Ambas as questões foram resolvidas com implementação de referências e tratamento de traits adequado.

---

## 📁 Arquivos Modificados

- [src-tauri/src/core/error/domain.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/error/domain.rs)
- [src-tauri/src/core/formats/mod.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/core/formats/mod.rs)
- [src-tauri/src/processing/mod.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/mod.rs)
- [src-tauri/src/processing/transcoding/mod.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/transcoding/mod.rs)
- [src-tauri/src/processing/media/mod.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/media/mod.rs)
- [src-tauri/src/processing/media/video_format.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/media/video_format.rs)
- [src-tauri/src/processing/media/audio_format.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/media/audio_format.rs)
- [src-tauri/src/processing/media/modern_image_format.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/media/modern_image_format.rs)
- [src-tauri/src/processing/media/fallback_format.rs](file:///Users/marcusmaia/Documents/Desenvolvimento/Mundam/src-tauri/src/processing/media/fallback_format.rs)

---

## 💡 Notas para o Desenvolvedor / Agente
> Processar clipes de vídeo em Subprocessos Rust é extremamente fácil deixar vazar o "Zumber" process se o Tokio não amarrar matar a Child Process. Garanta atrelar a morte da Pipeline a um tokio-kill na trait de Abort ao falhar/cancelar! A trait foi projetada pura propositalmente; não inclua Lógica de "Onde no disco a Thumb vai ser salva" no Codec Video. Devova só o Byte-Ray.
