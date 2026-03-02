# Sprint 1: Observability Foundation & Lifecycle Stabilization

**Data:** 2026-03-02
**Status:** Concluído
**Data e hora da conclusão:** 2026-03-02 17:25

## 📌 Objetivo
Estabelecer a fundação de observabilidade do sistema (Rust e TS) e garantir previsibilidade no ciclo de vida (Lifecycle). Integrando as vantagens da **Opção C (OpenTelemetry + IPC Centralizado)** de forma pragmática: focar em correlação de traces de ponta-a-ponta para diagnosticar gargalos reais de indexação/streaming no desenvolvimento, mas limitando excessos (*over-engineering*) no build de produção.

## 🛠 Tarefas de Implementação

### 1. Sistema de Tracing Estruturado e OTLP (Backend Pragmático)
- **Escopo:** Substituir chamadas imperativas (`println`) por telemetria baseada em *spans*.
- **Ações:**
  - Implementar os crates `tracing` e `tracing-subscriber`.
  - **Pragmatismo OTLP:** Configurar o exportador OpenTelemetry (Jaeger/Prometheus) associando-o estritamente a *feature flags* (uso exclusivo no desenvolvimento e profile local). Isso permite medir latências p95 e engarrafamentos I/O sem injetar peso extra no binário e no processamento do cliente em *release*.
  - Injetar a macro `#[tracing::instrument]` apenas nas fronteiras de alto atrito: comandos públicos (`tauri::command`) e operações SQL lentas documentadas em `src/library/commands/`.
- **Validação:** Logs propagam *thread identifier*, tempo, nível e erros estruturados (`AppError`) via propagação (`?`), banindo `.unwrap()`.

### 2. IPC Centralizado de Telemetria e `LifecycleManager` (Frontend)
- **Escopo:** Evitar vazamentos de memória (bindings Rust <-> TS) e correlacionar eventos de UX com o Backend local.
- **Ações:**
  - Desenvolver `src/core/utils/LifecycleManager.ts` focado no agrupamento tipado de `UnlistenFn`s via `onCleanup()`.
  - Desenvolver *Bridge IPC de Telemetria*: O frontend (React/Solid) enviará logs estruturados selecionados (ex.: tempo de hidratação de tela de galerias volumosas ou falhas graves de layout) diretamente para o *LifecycleManager*, que delega ao Rust. A thread Rust atua como a única fonte emissora de logs, unificando a linha do tempo TS e Rust num único *trace* distribuído sem recriar infraestruturas paralelas complexas no TS.
- **Validação:** Tipagem estrita com Zod nos *payloads* IPC, isolamento de domínios via abstração `tauriService`, conforme `core-architecture.md`.

### 3. Integração do Evento de Shutdown e Resiliência (Tauri/Backend)
- **Escopo:** Finalização garantida das *background tasks*.
- **Ações:**
  - Capturar `app.on_event` (`RunEvent::ExitRequested`) globalmente para acionar `LifecycleRegistry::shutdown_all()`.
  - Gravar *timestamps* para o rastreio da vida útil da task (Spawn/Cancel) preenchendo as métricas faltantes mapeadas no backlog.
  - **Fallback Timeout:** Adicionar mecanismo para o *LifecycleRegistry*: caso uma *task* pendurada (por file locks de SO ou falha no disco) ignore o encerramento em `N` segundos, acionar fallback explícito com `handle.abort()`.
- **Validação:** Instâncias liberam travas no SQLite e no SO de forma suave ao sair, acompanhados de testes unitários exemplificados.
