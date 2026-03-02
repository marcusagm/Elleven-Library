# Sprint 2: Core Refactoring - Watcher Pipeline & Streaming

**Data:** 2026-03-02
**Status:** Planejado
**Data e hora da conclusão:** -

## 📌 Objetivo
Possuindo a malha de *tracing* unificada (Frontend enviando ao Rust + Spans estruturados providos pela Sprint 1), podemos atacar com microscópio as rotinas de maior risco assíncrono: a conversão do Watcher monobloco em um Pipeline puro e o nivelamento profissional do Streaming.

## 🛠 Tarefas de Implementação

### 1. Segmentação Imutável do File Watcher
- **Escopo:** Substituir heurística de monobloco de gerenciamento filesystem por *pipeline* determinístico de 5 fases (_Single Responsibility Principle_).
- **Ações (Fases do Pipeline monitorado via Tracing):**
  - **Parse:** Interpretação simples focada puramente na decodificação passiva do SO (apenas identificação do evento).
  - **Normalize:** Agregação rápida de deduplicação temporal (*debounce*) com foco mínimo para ignorar múltiplos salvamentos (ex: autosaves massivos de Photoshop).
  - **Classify:** Identificação pesada invocando `FileFormat::detect` com isolamento de metadados iniciais e validação do `MediaType`.
  - **Persist:** Despacho em pacotes assíncronos (chunks) suportados no SQLite (utilizando `pool.begin()` global na transação).
  - **Emit:** Alerta ao cluster frontend usando as rotas padronizadas em TypeScript em `DomainEvents` via EventBus.
- **Validação:** Utilizando as ferramentas da *Sprint 1*, constatar que a perda de latência global entre o Save no OS e a visualização do Grid se mantém contida numa base milissecundária linear não bloqueante (ausência de picos).

### 2. Helpers Abstratos e Estabilização do Local Streaming
- **Escopo:** Mitigar `DataCloneError`, *network drops* abruptos e construções massivas de Response em `src-tauri`.
- **Ações:**
  - Concentrar e reutilizar lógicas de leitura transacional (`tokio::fs` para blocos pesados), aplicação de Range-Headers nativos (206 Partial Content) exigidos em MediaPlayers nativos e CORS *restrito*, todos em utilitários testáveis no domínio `streaming`.
  - Encapsular instabilidades do SO e tratar erros com polidez antes do envio ao frontend, assegurando a validade no `AppResult` formatado para JSON em vez de string-panics brutais no `Result<Response>`.
- **Validação:** Navegador exibe `206 Partial Content` corretamente em vídeos pesados (4K ProRes) localizados remotamente e logs disparam Warning/Error adequados se o fluxo for forçadamente interrompido ou trancado pelo SO.
