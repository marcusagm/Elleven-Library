# Refactoring Report: `lib.rs` to Modular Bootstrap

**Date:** 2026-06-13
**Author:** AI Assistant
**Status:** Completed

## 1. Contexto do Problema

O arquivo principal da aplicação, `src-tauri/src/lib.rs`, havia se tornado um gigantesco "God Composition Root". Ele estava acumulando dezenas de instâncias de domínios distintos, desde rotas de Streaming de Vídeo até inicialização de Banco de Dados e Event Buses, num bloco `.setup()` procedural maciço de quase 400 linhas.

Isso apresentava um risco para a escalabilidade do projeto:
- **Dificuldade em testes isolados** (ex: boot isolado do banco).
- **Complexidade cognitiva extrema**, ocultando a cadeia de dependências.
- **Risco de regressões**, pois uma alteração num worker poderia impactar o setup do HLS.

## 2. Abordagem e Solução

Foi decidido **não adotar** a tática de "Tauri Plugins" para evitar quebrar o princípio Hexagonal do projeto, onde a camada `delivery` abriga todos os comandos passivos `invoke()`.

Em vez disso, a lógica de inicialização foi delegada para o módulo recém-criado `src-tauri/src/bootstrap/`. Esta pasta atua como a única **Raiz de Composição (Composition Root)** do projeto, conhecendo o framework Tauri (`AppHandle`) para atuar puramente como orquestrador e injetor de dependências via Service Locator (`app.manage()` e `app.state()`).

### 2.1 Estrutura de Módulos Implementada

1. **`bootstrap/mod.rs`**: O módulo central que exporta as sub-funcionalidades. Ele define a estrutura `AppDirectories`, que resolve e guarda os caminhos vitais do file system (AppData, Banco, Thumbnails) de forma O(1) para os outros módulos.
2. **`bootstrap/system.rs`**: Resolve arquivos de fundação. Levanta o `TokioEventBus`, o sistema de telemetria, carrega as definições de configurações (`SettingsService`) e prepara o `FormatRegistry` e o `LifecycleRegistry`.
3. **`bootstrap/database.rs`**: Inicializa as conexões com o SQLite, injeta o `DbManager`, engata os *Query Handlers* (leitura) e o *Asset Ledger* (escrita transacional), além de disparar rotinas únicas de normalização de caminhos.
4. **`bootstrap/streaming.rs`**: Monta o servidor HTTP embutido via Axum, instancia o `HlsManager` e os caches dinâmicos de transcode, e faz verificações cruciais (`health-check`) do binário FFmpeg.
5. **`bootstrap/workers.rs`**: Inicia os "Background Consumers", como o `ThumbnailWorker` (baseado em fila) e o `ColorWorker` (reativo a eventos do barramento).
6. **`bootstrap/library.rs`**: Fecha o ciclo preparando os `Watchers` do SO para as pastas radiculares, aciona o `LibraryIndexer` e despacha varreduras de boot (Boot Scan) visando a recuperação de inconsistências.

O bloco `.setup()` do Tauri no `lib.rs` agora consome cerca de 20 linhas organizadas de inicialização procedimental pura.

## 3. Conformidade com as Diretrizes e Padrões (Guidelines)

Foi feita uma revisão rígida do código gerado à luz dos documentos de regras do projeto (`documentation.md` e `guidelines.md`):

- **Arquitetura Hexagonal Intacta:** A refatoração foi restrita unicamente à montagem inicial. O módulo `delivery` continua sendo a borda imutável. Nenhum dos domínios Core foi "sujado" com macros do Tauri.
- **Service Locator (State):** Não foram utilizadas passagens de cadeias infinitas de `Arc<T>` nas assinaturas de métodos. Em vez disso, usou-se de forma madura o `.state()` e `.manage()` do framework.
- **Documentação Rustdoc:** Todos os novos arquivos receberam docstrings aderentes às restrições:
  - Adição do Big Picture usando `//!` no início de todos os novos módulos.
  - Documentação nas funções principais via `///`, com foco no **Por Que (Arquitetura)** e descrevendo secções de `# Arguments` e `# Errors` quando aplicável.
  - O código não faz uso de divisores visuais proibidos (`// =======`).
- **Nomenclatura Limpa:** As variáveis são explícitas (`app_directories`, `thumbnail_worker`), fugindo rigorosamente de encurtamentos como `dir` ou `wk`.
- **Tratamento Seguro:** Não inserimos novos `.unwrap()`, com exceção das rotinas deliberadamente fixadas no start block inicial onde as variáveis não são recuperáveis via fallback (comportamento de "Panic Early" adequado para Composition Roots na inicialização).

A arquitetura agora está fluida, documentada e modularizada, finalizando com sucesso o isolamento de setup do backend do Mundam.
