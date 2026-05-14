# Sprint 11.1: Correção e Estabilização do Suporte de Imagens (Arquitetura V2)

**Status da sprint:** Planejamento
**Data e hora de início da sprint:** -
**Data e hora da conclusão estimada:** -

## Objetivo

Resolver as regressões identificadas na transição para a Arquitetura V2 no suporte a formatos de imagem, com foco em:
1. Restaurar paridade de indexação para formatos legados (`cur`, `tga`, `icns`).
2. Implementar extração completa de metadados EXIF/Camera para todos os formatos RAW.
3. Integrar o extrator binário multicamadas (Tier 3) no `RawFormatProvider`.
4. Corrigir falhas de visualização (previews) em formatos modernos e profissionais.
5. Melhorar a qualidade visual das thumbnails e organizar a exibição de metadados no ItemInspector.

## Análise de Gap e Problemas Identificados

Com base na verificação manual (`task-list-end-new-backend.md`), foram identificados os seguintes gaps entre a V1 e a V2:

| Categoria        | Problema Identificado                                           | Impacto      | Causa Provável                                                   |
| :--------------- | :-------------------------------------------------------------- | :----------- | :--------------------------------------------------------------- |
| **Indexação**    | `cur`, `tga`, `icns` não são indexados na V2.                   | 🔴 Bloqueante | Extensões ausentes na `supported_extensions` dos providers.      |
| **Metadados**    | Falta de dados EXIF/Camera (ISO, Abertura, etc) em RAWs.        | 🟠 Grave      | `RawFormatProvider::extract_technical` limitado a width/height.  |
| **RAW Previews** | Regressão de preview em quase todos os formatos RAW.            | 🟠 Grave      | Tier 3 (Binary Scanner) não integrado; falta de fallback FFmpeg. |
| **Modernos**     | `jxl` sem thumbnail; `heic`/`avif` com erro de timeout/preview. | 🟠 Grave      | Estratégia `Icon` no JXL; timeout de 10s no FFmpeg insuficiente. |
| **Qualidade**    | Qualidade inferior em thumbnails PNG e NEF.                     | 🟡 Médio      | Uso de filtro `Bilinear` em vez de `Lanczos3`.                   |
| **Interface**    | Metadados exibidos como `[object Object]` no ItemInspector.     | 🟡 Médio      | Problema de serialização JSON do mapa de EXIF para o frontend.   |
| **Dados**        | Informações redundantes de largura/altura no "Advanced Data".   | 🔵 Baixo      | Duplicação entre metadados core e técnicos.                      |

## Referências Técnicas

### V1 (Gold Standard)
- `src-tauri/src/thumbnails/raw.rs`: Estratégia de 3 camadas (LibRaw -> BruteForce -> BinaryScanner).
- `src-tauri/src/thumbnails/extractors/binary_jpeg.rs`: Scanner robusto para JPEG/PNG/TIFF/XMP embutidos.
- `src-tauri/src/formats/definitions.rs`: Registro mestre com estratégias específicas.

### V2 (Current State)
- `src-tauri/src/processing/media/raw_format.rs`: Implementação incompleta (Tier 1 e 2 apenas).
- `src-tauri/src/processing/media/image_format.rs`: Provedor principal com suporte a EXIF via `rexif`.
- `src-tauri/src/processing/media/extractors/binary_jpeg.rs`: Portado da V1, mas não utilizado.

---

## Tarefas do Plano de Ação

### 1. Restauração de Formatos (Indexação)
- [ ] Adicionar extensões `cur` e `tga` ao `ImageFormatProvider`.
- [ ] Adicionar extensão `icns` ao `IconFormatProvider` ou `ImageFormatProvider` com estratégia de extração de ícone.
- [ ] Verificar `supports_magic_bytes` para esses formatos para garantir detecção profunda.

### 2. Upgrade do Provedor RAW (`RawFormatProvider`)
- [ ] **Extração de Metadados**: Implementar extração de EXIF completa usando `rexif` ou delegando para utilitário unificado.
- [ ] **Integração Tier 3**: Chamar `extractors::binary_jpeg::extract_any_embedded` como último recurso síncrono antes do erro.
- [ ] **Qualidade de Preview**: Aumentar a qualidade de saída na conversão WebP (atualmente 80.0) para 90.0 em previews.

### 3. Correção de Formatos Modernos (`ModernImageFormatProvider`)
- [ ] **JPEG XL (JXL)**: Mudar estratégia de `Icon` para `Ffmpeg` ou implementar via `zune-jxl`.
- [ ] **Timeout & Performance**: Aumentar timeout do FFmpeg de 10s para 30s em arquivos RAW/Modernos pesados.
- [ ] **HEIC/AVIF**: Revisar parâmetros de escala do FFmpeg para evitar falhas em imagens com dimensões não-padrão.

### 4. Estabilização de Formatos Específicos
- [ ] **DDS**: Corrigir erro de "dimensions too small or too large" investigando o `ImageReader` ou implementando extractor específico para headers DDS.
- [ ] **HDR**: Implementar suporte a Radiance HDR no `ImageFormatProvider` (atualmente delegado mas não tratado).
- [ ] **PNG**: Verificar por que a thumbnail V2 é inferior à V1 (possível diferença no tratamento de transparência/alfa).

### 5. Refino Visual e de Dados (Qualidade Mundam Premium)
- [ ] **Filtro de Resample**: Substituir `fr::FilterType::Bilinear` por `fr::FilterType::Lanczos3` em `image_utils.rs` e `image_format.rs`.
- [ ] **Serialização de EXIF**: Ajustar a estrutura do retorno JSON para que o frontend receba uma lista de chave-valor em vez de um objeto nested que causa o erro `[object Object]`.
- [ ] **Deduplicação**: Remover campos de largura/altura da extração técnica se eles já existirem no core do `Asset`.

### 6. Icones genericos de Fallback
- [ ] Todos os formatos que utilizam ThumbnailStrategy::Icon deve apenas registrar no banco de dados que não possuem thumbnail, pois o frontend agora que é reponsável por gerar icones genericos para quando um formato não possui thumbnail ou ocorreu algum erro na geração. O arquivo do frontend que gera esses icones é o `src/components/features/viewport/assets/FileIcon.tsx` utilizado por arquivos como `src/components/features/viewport/assets/Thumbnail.tsx` para garantir que os assets que precisam de icones, os mostrem. Talvez criar um `ThumbnailStrategy::Fallback` ou `ThumbnailStrategy::None` para que o `Thumbnail` do frontend saiba que deve gerar um icone generico?
- [ ] ThumbnailStrategy::Icon deve passar a trabalhar exclusivamente formatos de icones como `icns`, `ico` e `cur` e `ani`.

---

## Critérios de Aceitação

- [ ] Arquivos `.cur`, `.tga` e `.icns` aparecem na biblioteca após scan.
- [ ] ItemInspector exibe dados de Câmera (ISO, F-Stop) para arquivos `.nef`, `.cr2`, `.arw`.
- [ ] Imagens que falhavam no preview (ex: `3fr`, `dng`) agora exibem imagem em tela cheia.
- [ ] Erro `[object Object]` no ItemInspector resolvido.
- [ ] Thumbnails de alta fidelidade (Lanczos3) visíveis no Grid.
- [ ] JXL exibe thumbnail corretamente.

---

## Riscos e Mitigações

| Risco               | Impacto                                                       | Mitigação                                                                                       |
| :------------------ | :------------------------------------------------------------ | :---------------------------------------------------------------------------------------------- |
| **Performance**     | O uso de Lanczos3 e extração profunda pode lentificar o scan. | Utilizar `spawn_blocking` e garantir que o Tier 1 (LibRaw) resolva 90% dos casos rapidamente.   |
| **Memória**         | Imagens RAW gigantes podem causar OOM no worker.              | Manter o uso de `memmap2` e evitar carregar o arquivo inteiro em `Vec<u8>` sempre que possível. |
| **Compatibilidade** | FFmpeg pode não ter suporte a JXL em todas as máquinas.       | Adicionar verificação de capacidade dinâmica ou fallback para `zune-jxl`.                       |
