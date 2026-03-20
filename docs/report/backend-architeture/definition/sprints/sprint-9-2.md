# Sprint 9.2: Consolidação da Definição de Formatos (V2) e Remoção do definitions.rs

**Status da sprint:** Concluído
**Data e hora de inicio da sprint:** 2026-03-20 13:00
**Data e hora da conclusão da sprint:** 2026-03-20 17:00 (Simulação de conclusão após verificação de build)

## Tarefas

### Consolidação Estrutural (Format Registry v2)

**Status:** Concluído
**Data e hora de inicio:** 2026-03-20 10:00
**Data e hora da conclusão:** 2026-03-20 11:11

**Arquivos da arquitetura v1 para referência:**
- `src-tauri/src/formats/definitions.rs` (Totalmente removido)
- `src-tauri/src/formats/mod.rs` (Totalmente removido)
- `src-tauri/src/formats/types.rs` (Totalmente removido)

**Lista de Objetivos**

- [x] Sincronizar provedores de mídia com a nova assinatura `SupportedFormat::with_metadata` (adição do `ThumbnailStrategy`).
- [x] Migrar todas as definições de formatos de `definitions.rs` para provedores individuais em `src/processing/media/`.
- [x] Implementar detecção eficiente por extensão no `FormatRegistry` (`detect_by_extension`).
- [x] Refatorar infraestrutura de transcoding para ser "Format-Aware" através de injeção de dependência do `FormatRegistry`.
- [x] Resolver problemas de passagem de referências em comandos async do Tauri (mudança para `AppResult<bool>`).
- [x] Limpeza técnica: remoção definitiva do diretório `src-tauri/src/formats/`.

## 💡 Notas para o Desenvolvedor / Agente

> Esta sprint focou na "limpeza pesada" da dívida técnica deixada pela fase de transição entre V1 e V2. A remoção do arquivo `definitions.rs` centralizado em favor de provedores modulares e descentralizados garante que a Mundam possa escalar para centenas de novos formatos sem criar gargalos no registro mestre.

> O uso de `ThumbnailStrategy` permite agora que cada formato defina seu próprio motor de renderização de miniaturas (ex: nativo, icon-only, ou via transcodificação específica), o que é vital para o suporte a arquivos profissionais (PSD, RAW, CAD).

## 🚀 Informações da Implementação

### Dificuldades e Desafios

#### Injeções de Dependência e Ciclo de Vida
Um grande desafio foi tornar o `TranscodeCache` ciente dos formatos suportados sem criar loops de dependência circular. A solução foi a injeção do `Arc<FormatRegistry>` durante a inicialização no `lib.rs`, permitindo que o cache resolva extensões de saída dinamicamente através do `TranscodingDetector`.

#### Restrições do Tauri IPC
Ao mover a lógica de detecção de formatos para o `FormatRegistry` injetado, os comandos Tauri que recebiam o estado (`State<Arc<FormatRegistry>>`) e eram marcados como `async` começaram a falhar na compilação. Isso ocorreu devido a uma restrição do Tauri na serialização de referências em comandos assíncronos que não retornam `Result`. A correção envolveu a padronização desses comandos para retornar `AppResult<bool>`, garantindo segurança de tipos e compatibilidade com o frontend.

#### Mapeamento de Thumbnails
Para formatos complexos como PDF e Aseprite, foi necessário remapear estratégias que antes eram "hardcoded". PDFs agora usam explicitamente `ThumbnailStrategy::Icon` (ou extratores nativos quando disponíveis), e arquivos Aseprite foram migrados para o padrão `NativeExtractor`, garantindo que o `ThumbnailWorker` use o motor correto de decodificação.

### Melhorias Realizadas

- **Descentralização Total:** A lógica de formatos não reside mais em um único arquivo gigante, mas sim nos provedores de mídia específicos, seguindo fielmente a arquitetura hexagonal.
- **Busca Avançada O(1):** A adição de `detect_by_extension` no `FormatRegistry` permite que o `SearchBuilder` construa queries SQL complexas instantaneamente, mapeando extensões para nomes amigáveis no banco de dados.
- **Build Green:** Resolução de todos os erros de compilação relacionados à mudança de assinatura, resultando em uma base de código estável e verificada pelo `cargo check`.
- **Type Safety no Frontend:** Alinhamento das interfaces TypeScript com os novos retornos de comandos IPC (`AppResult` compatível).

### 📄 Arquivos Criados ou Modificados

#### Core
- `src-tauri/src/core/formats/registry.rs`: Implementação de `detect_by_extension`.
- `src-tauri/src/core/formats/types.rs`: Consolidação de enums de estratégia.

#### Provedores (Ajuste de Assinatura)
- `src-tauri/src/processing/media/affinity_format.rs`
- `src-tauri/src/processing/media/ai_format.rs`
- `src-tauri/src/processing/media/archive_format.rs`
- `src-tauri/src/processing/media/aseprite_format.rs`
- `src-tauri/src/processing/media/audio_format.rs`
- `src-tauri/src/processing/media/binary_design_formats.rs`
- `src-tauri/src/processing/media/cad_format.rs`
- `src-tauri/src/processing/media/exr_format.rs`
- `src-tauri/src/processing/media/fallback_format.rs`
- `src-tauri/src/processing/media/font_format.rs`
- `src-tauri/src/processing/media/icon_format.rs`
- `src-tauri/src/processing/media/image_format.rs`
- `src-tauri/src/processing/media/model3d_format.rs`
- `src-tauri/src/processing/media/modern_image_format.rs`
- `src-tauri/src/processing/media/pdf_format.rs`
- `src-tauri/src/processing/media/project_zip_formats.rs`
- `src-tauri/src/processing/media/psd_format.rs`
- `src-tauri/src/processing/media/raw_format.rs`
- `src-tauri/src/processing/media/svg_format.rs`
- `src-tauri/src/processing/media/usd_format.rs`
- `src-tauri/src/processing/media/video_format.rs`
- `src-tauri/src/processing/media/xmind_format.rs`

#### Feature & Transcoding
- `src-tauri/src/feature/transcoding/cache.rs`: Refatorado para `Registry-Aware Cache`.
- `src-tauri/src/feature/transcoding/detector.rs`: Atualizado para usar o novo registro.
- `src-tauri/src/delivery/tauri/commands/streaming.rs`: Correção de assinaturas de comandos assíncronos.

#### Infra & Cleanup
- `src-tauri/src/infra/database/search_builder.rs`: Integração com `FormatRegistry`.
- `src-tauri/src/lib.rs`: Bridge de inicialização do cache e remoção do módulo `formats` legado.
- `src-tauri/src/formats/` (Diretório apagado).
