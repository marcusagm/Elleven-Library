# Sprint 10.5: Migração Completa — Extractor CorelDRAW (.cdr)

**Status da sprint:** ✅ Concluída
**Data e hora de inicio da sprint:** 2026-05-05T17:00:00-03:00
**Data e hora da conclusão da sprint:** 2026-05-05T20:20:00-03:00

## Objetivo

Garantir paridade completa do extractor CorelDRAW (`.cdr`) na V2, incluindo thumbnail embarcado, metadados técnicos e preview em alta resolução, superando a fidelidade da V1 através de parsing estrutural profundo.

## Estado Atual

### V1 — `mundam-main/src-tauri/src/thumbnails/extractors/coreldraw.rs`
- **Tamanho:** 18,425 bytes
- Parse completo do formato RIFF de arquivos CDR
- Extração do bloco `RIFF → DISP` ou `RIFF → bmDt` para thumbnail
- Suporte a CDR v16+ (mini-bitmap embutido) e versões antigas
- Extração de metadados via bloco `INFO`

### V2 — `src-tauri/src/processing/media/extractors/coreldraw.rs`
- **Status:** Implementação de alta fidelidade concluída
- **Capacidades:** Upscale Lanczos3, metadados reais (mm), suporte a ZIP/RIFF/WL completo.

## Análise de Gap Final

| Funcionalidade | V1 | V2 (Final) | Melhoria |
|---|---|---|---|
| Parse RIFF container | ✅ | ✅ | Implementação robusta com walk recursivo |
| Extract bloco `DISP` (thumbnail) | ✅ | ✅ | Priorização por densidade de pixels |
| Suporte multi-versão CDR | ✅ | ✅ | v3 até v18+ (X8+) |
| Dimensões do documento | Pixels | ✅ mm | Agora extrai dimensões REAIS da página via `mcfg` |
| Preview Alta Resolução | ❌ | ✅ | Upscale Lanczos3 para 1024px |
| Suporte ZIP Interno | Parcial | ✅ | Parsing completo do `riffData.cdr` interno |

## Tarefas Realizadas

### 1. Auditoria e Portabilidade V1 vs V2
**Status:** ✅ Concluído
Identificados os gaps de parsing RIFF e suporte a versões legadas. A lógica de extração foi totalmente reescrita para focar em precisão e não apenas em heurísticas de bitmap.

### 2. Implementação de Parsing RIFF e ZIP Moderno
**Status:** ✅ Concluído
Implementado suporte para arquivos ZIP (`.cdr` modernos), com extração e parsing do arquivo de dados interno (`content/riffData.cdr`), permitindo acesso aos chunks de metadados reais mesmo em arquivos empacotados.

### 3. Integração com BinaryDesignFormatProvider
**Status:** ✅ Concluído
Adicionado suporte nativo a `.cdr` no provider, garantindo que o backend reconheça e processe o formato automaticamente.

### 4. Metadados Reais e Preview High-Res
**Status:** ✅ Concluído
Implementada a extração do chunk `mcfg`, convertendo unidades internas do CorelDRAW para milímetros, e pipeline de preview com upscale Lanczos3 para máxima qualidade visual no modal.

## Detalhes da Implementação V2 (CorelDRAW)

Nesta sprint, elevamos o suporte ao CorelDRAW para um nível de fidelidade superior à arquitetura anterior:

### 🚀 Preview de Alta Fidelidade
- **Upscaling Inteligente**: Implementamos o algoritmo **Lanczos3** para elevar os previews embutidos (geralmente de 256px) para **1024px**, mantendo a nitidez necessária para visualização em telas modernas.
- **Heurística de Seleção**: O motor agora analisa a contagem de pixels de múltiplos candidatos (thumbnail vs page1.png) para escolher sempre a fonte de maior densidade.

### 📏 Metadados de Precisão (Dimensionamento Real)
- **Chunk `mcfg`**: Migramos da extração de dimensões baseada em imagem (que reportava o tamanho do thumbnail) para a leitura do chunk `mcfg`.
- **Unit Awareness**: Implementamos a conversão das "Corel Units" (1/10000 mm) para milímetros reais, permitindo que o usuário veja o tamanho exato da página do documento no Inspector.
- **Version Offsets**: O parser lida com os diferentes offsets do chunk `mcfg` entre versões (v600, v900, v1300), garantindo compatibilidade histórica.

### 📦 Arquitetura ZIP e RIFF Unificada
- **Deep ZIP Inspection**: Para arquivos modernos, o sistema agora "mergulha" no pacote ZIP, localiza o RIFF de dados e aplica o mesmo parser estrutural usado em arquivos legados, unificando a extração de metadados.
- **Fallback Robusto**: Em caso de falha no parsing estrutural, mantivemos um scanner binário de emergência para localizar bitmaps brutos em arquivos corrompidos ou malformados.

## Arquivos Modificados

- `src-tauri/src/processing/media/extractors/coreldraw.rs` — Implementação core do extractor.
- `src-tauri/src/processing/media/extractors/mod.rs` — Exportação de utilitários de versão.
- `src-tauri/src/processing/media/binary_design_formats.rs` — Enriquecimento de metadados JSON.

## Critérios de Aceitação

- [x] Arquivo `.cdr` gera thumbnail correto.
- [x] Suporte a múltiplas versões do CDR (v3 até X8+).
- [x] Inspector mostra dimensões reais em **mm**.
- [x] Preview modal mostra imagem nítida (Upscale 1024px).
- [x] Metadados incluem versão exata do CorelDRAW (ex: "RIFF v12.0").

## Referência Final
- Documentação técnica baseada nos specs do Kaitai Struct e UniConvertor para máxima conformidade com o formato proprietário.

