# Roadmap de Implementação Arquitetural

Este documento consolida o plano de ataque para a transição do backend do Mundam rumo à Arquitetura Hexagonal, CQRS e Event-Driven Architecture.

Devido à natureza de desenvolvimento colaborativo entre o humano e Agentes de IA, o progresso é fatiado em **Sprints Curtas e de Baixo Contexto**. Cada Sprint foca numa única responsabilidade do sistema, garantindo que a base de código não se quebre e que as interações no prompt (contexto) não se desgastem. 

As Sprints são organizadas em **Fases (Grupos)** que culminam numa Entrega de Valor Testável de ponta-a-ponta (E2E).

---

## Fase 1: Fundação & Observabilidade (Core Mínimo)
**Objetivo Testável:** Uma infraestrutura primária na qual o backend compila a nova estrutura de pastas, liga o banco SQLite limpo e se comunica via canais assíncronos e tratamentos de erro formatados sem acoplar a interface.

*   **Sprint 1.1: Arcabouço Físico e Error Handling**
    *   **Escopo:** Criação das pastas (`core/`, `feature/`, `infra/`, `delivery/`). Refatoração e centralização da `AppError` e `AppResult<T>`.
    *   **Teste E2E:** Disparo de erros simulados em endpoints HTTP ou Command do Tauri, garantindo a interceptação segura (`Tracing`) e conversão para payloads JSON polidos.
*   **Sprint 1.2: EventBus Assíncrono**
    *   **Escopo:** Implementação do núcleo de Mensageria em Memória (`tokio::sync::broadcast`) no `core/events`. Definição do Enum `DomainEvent`.
    *   **Teste E2E:** Módulos autônomos assinando tópicos (Publisher -> Subscriber) em testes unitários.
*   **Sprint 1.3: Data Model Base Base (Infra)**
    *   **Escopo:** Mapeamento de Entidades do banco e adaptações da conexão DB (`SqlitePoolManager`).
    *   **Teste E2E:** Aplicação das Migrations vazias contra um DB isolado (In-Memory ou File).

---

## Fase 2: Domínio & Operações de Mutação (CQRS)
**Objetivo Testável:** Realizar Inserções (Mutação) auditadas na base de dados de assets, separadas brutalmente das rotinas de Leitura Rápida, e disparar eventos colaterais no Ledgers.

*   **Sprint 2.1: Traits do Domínio e Modelos (Commands)**
    *   **Escopo:** Estabelecimento de Data Transfer Objects (`CreateAssetCommand`) e Trait `AssetLedger`.
    *   **Teste E2E:** Interface Rust puramente estática. Mock local validando regras de State-Machine.
*   **Sprint 2.2: O Adaptador do Ledger (Infra)**
    *   **Escopo:** Criação do `SqliteLedgerAdapter`, contendo as querys seguras (Transactions/Rollbacks) de escrita (Mutativa).
    *   **Teste E2E:** Inserir mil ativos num Teste de Integração e checar Atomicidade.
*   **Sprint 2.3: Query Handlers Base (Leitura Flexível)**
    *   **Escopo:** Serviços de Listagem Rápida e paginação simples por Extensão. (Operação Isenta do Ledger).
    *   **Teste E2E:** Gravar via Ledger e Buscar limpo via Query Handler nos testes.
*   **Sprint 2.4: Taxonomia, Metadata e Pastas (Grafos e Hierarquia)**
    *   **Escopo:** Gestão da árvore de pastas lógicas (`Folder`) e Associação livre de Tags N:N (`Asset_Tags`).
    *   **Teste E2E:** Mover um arquivo para uma pasta virtual via Ledger e aplicar uma Tag "Aprovado", refletindo nos joins dos Queries.
*   **Sprint 2.5: Search Builder e Buscas Avançadas (Cores, Arrays e Dicionários)**
    *   **Escopo:** Restauração do mecanismo avançado de pesquisa do Mundam. Operadores lógicos integrados aos `QueryHandlers` para combinar filtro de Cor LAB Euclidiana, Filtro de Hash e Tags.
    *   **Teste E2E:** Emissão de uma busca combinada (Ex: Cor Hex + Extension "PSD").

---

## Fase 3: O Format-Kit Registry (Extração Pura)
**Objetivo Testável:** Isolamento do coração extrator do app. Injetar provedores dinamicamente via HashMaps(O(1)) sem trancar o CPU com Iteradores gigantes. Passos baseados no Guia Oficial (*format-implementation-guide.md*).

*   **Sprint 3.1: Interface FormatRegistry & Estrutura**
    *   **Escopo:** Definição da Fabrica O(1) de Identificação. Os HashMaps por extensão e Traits Subjacentes (`ThumbnailCapability`, `MetadataCapability`).
    *   **Teste E2E:** Resolver Mock Providers ultra-rápido perante Extensões ou Magic Bytes.
*   **Sprint 3.2: Migração das Mídias Nativas Primárias**
    *   **Escopo:** Substituir velhas diretrizes pelas Novas Classes: `ImageFormatProvider` e `PdfFormatProvider`.
    *   **Teste E2E:** Metadados limpos e Array de Bytes fluindo no terminal com Mock de Disco.
*   **Sprint 3.3: Fallbacks Dinâmicos (Vídeos & Arquivos Raros)**
    *   **Escopo:** Adaptação da interface `FfmpegFormatProvider` e provedores baseados exclusivamentes em Magic Byte Header.
    *   **Teste E2E:** Falta de extensão resolvida assertivamente baseada nos bytes canônicos de header, emitindo Extrações Nulas limpas.
*   **Sprint 3.4: Extratores Especiais (RAW, SVG, Fonts, Modelos 3D e ZIP Prev)**
    *   **Escopo:** Migração robusta de formatadores complexos: Câmeras RAW, Fontes (TTF/OTF), Renders de SVG, Thumbnails embutidos em ZIP/CBZ e extração de Previews High-Res para renderização Full-Screen.
    *   **Teste E2E:** Ingestão de um `.CR2` (Raw) ou `.ZIP` resultando na correta extração técnica e geração da imagem WebP correspondente através de seus provedores dedicados.

---

## Fase 4: O Músculo Operacional (Workflows) 
**Objetivo Testável:** Processamento em lote massivo e Watcher operando no Submundo para Indexar HDs sem onerar a fluidez da UI.

*   **Sprint 4.1: Thumbnail Worker Pool & Fila de Prioridade**
    *   **Escopo:** Criação de Fila de Trabalhadores em Tokio consumindo o `Format-Kit` para geração dupla: Miniaturas Rápidas e Previews Otimizados. Implementação estrita da Fila de Prioridade LIFO para itens visíveis na tela da UI.
    *   **Teste E2E:** Navegação do usuário dispara dezenas de pedidos prioritários; o Worker interrompe a fila comum silenciosa e processa sob-demanda instantaneamente.
*   **Sprint 4.2: FileSystem Watcher & Scan Debouncer**
    *   **Escopo:** Varredura recursiva unificada. A captura dos hooks Create/Delete no OS para emissor de Eventos nativo do SO pro Ledger.
    *   **Teste E2E:** Adicionar uma pasta no sistema e ver o Scan converter aquilo em Commands pro Ledger e EventBus rodando no modo automático.
*   **Sprint 4.3: Extração de Cores e Semântica (Análise em Background)**
    *   **Escopo:** Adição explícita do Workflow `color_analysis`. O Worker extrai a paleta de Cores e Grava as Variáveis CIELAB Euclidiana atrelando ao Asset Pós-geração de Thumbnail.
    *   **Teste E2E:** Thumbnail Gerada dispara Evento -> Worker de Cor captura -> Insere as Hexadecimais relativas na Database.

---

## Fase 5: A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo Testável:** O Front (Solid.js) e o Sistema se encontram. Downloads de Mídias de 4GB acontecem via range bypassando o IPC Payload limit, transcodificações em HLS fluem sem travamentos de thread principal e a aplicação encerra de forma cirúrgica.

*   **Sprint 5.1: Servidor HTTP & Delivery Estático (Axum/Warp)**
    *   **Escopo:** Prover um Host local (`localhost:Porta`) com protocolo assinado nativamente (`asset://` local) para entrega de Thumbnails pesadas, Previews e Mídias com `206 Partial Content`.
    *   **Teste E2E:** Leitura/Scrubbing instantâneo em um arquivo de áudio ou vídeo MP4 local simulando o Front consumindo com `Range:` headers sem gargalos na memória.
*   **Sprint 5.2: Transcoding On-the-fly (HLS Video & Audio)**
    *   **Escopo:** Migração completa da `transcoding/`. Conversão sob demanda (FFmpeg) fatiando `.MKV` / `.FLAC` pesados incompatíveis com o Chromium em pequenos manifestos e segmentos Stream HLS.
    *   **Teste E2E:** Dar Play num arquivo de vídeo incompatível na Web; o Rust inicia um subprocesso de Segmentação `m3u8` e o HTML5 Video Player começa a reprodução a partir do pedaço `000.ts`.
*   **Sprint 5.3: Bindings IPC / Frontend Wiring**
    *   **Escopo:** Conclusão dos Commands `#[tauri::command]` (Porta de Entrada). Resposta aos payloads IPC seguindo JSON Schema restritos.
    *   **Teste E2E:** Cliques Reais no Front acionando os Comandos Hexagonais do Backend, vendo miniaturas renderizarem no DOM do Mundam.
*   **Sprint 5.4: Settings & Lifecycle (Graceful Shutdown)**
    *   **Escopo:** Recuperação e Gravação das Sessões (`config::AppConfig`). Amarra oficial da Trait do App Lifecycle controlando Cancellation Tokens, impedindo Fechamento Bruto enquanto processos de Transcode ou Thumbnails estão em execução.
    *   **Teste E2E:** Ordem de `SIGTERM` enviada; O Backend aborta novos processos suavemente, salva a config file e morre retornando exit code 0.

---

## Fase 6: Cleanup e Consolidação V2
**Objetivo Testável:** Remover todo o lixo e componentes legados do V1 (Frontend Queries, Rust Obsolete Modules, Sqlite Legacy Tables), unificando inteiramente a arquitetura sobre o modelo Hexagonal e Event-Driven, mantendo o build limpo e com performance estável.

*   **Sprint 6.1: Mapeamento e Refatoração do Frontend**
    *   **Escopo:** Substituir invocações Tauri (`src/lib/tags.ts`, `src/lib/db.ts`) antigas pelo mapeamento correto dos novos Handlers V2 (`/delivery/tauri/commands`).
    *   **Teste E2E:** Todas as listas de pastas, assets e smart-searches fluem na interface do SolidJS conectadas ao backend Hexagonal sem erros de unmapped command.
*   **Sprint 6.2: Purificação do Tidy Backend (Eliminação V1)**
    *   **Escopo:** Exclusão física das pastas `src-tauri/src/library/`, `db/`, `indexer/`, `transcoding/`, `media/` e `thumbnails/`, expurgando-as do `lib.rs`.
    *   **Teste E2E:** `cargo build` gerando warnings 0, removendo handlers zumbis e reduzindo o target time.
*   **Sprint 6.3: Limpeza de Banco de Dados e Migração Definitiva**
    *   **Escopo:** Construção de SQLx migration descartando estruturas da versão velha (`assets`, `folders`...) garantindo que `v2_assets` e complementares se tornam as entidades únicas de Querying.
    *   **Teste E2E:** Banco de dados otimizado, `cargo sqlx prepare` sem acoplamentos, operando em FK seguras.
*   **~~Sprint 6.4: E2E Validation & Relatório Final~~** *(Cancelada — substituída pela Fase 8, Sprint 8.4)*

---

## Fase 7: Paridade IPC — Taxonomia e Organização
**Objetivo Testável:** Restaurar todas as funcionalidades de organização de assets (Tags, Folders, Smart Folders, Ratings, Notes, Metadata, Cores) que existiam no V1, agora fluindo pela arquitetura Hexagonal (Ledger + QueryHandler + EventBus). Ao final, o frontend opera 100% sobre os novos endpoints V2 para taxonomia e metadados.

*   **Sprint 7.1: Tags CRUD Completo**
    *   **Escopo:** Criar/Editar/Deletar tags via Ledger, queries de tags por asset, batch operations (add/remove/replace tags em múltiplos assets). Total: 7 novos IPC commands.
    *   **Teste E2E:** Criar tag no frontend, aplicar a assets, editar cor, deletar — tudo sem erros.
*   **Sprint 7.2: Folders Avançados, Indexação Manual e Contadores**
    *   **Escopo:** `remove_location` com limpeza de thumbs e stop watcher, `get_all_subfolders`, `get_subfolder_counts`, `get_location_root_counts`, `start_indexing`. Total: 5 novos IPC commands.
    *   **Teste E2E:** Remover pasta pela UI, confirmar limpeza. Disparar re-scan manual. Contadores na sidebar corretos.
*   **Sprint 7.3: Smart Folders CRUD e Contadores de Biblioteca**
    *   **Escopo:** Tabela `smart_folders` + CRUD (4 commands), `get_asset_count_filtered`, `get_library_stats`. Total: 6 novos IPC commands.
    *   **Teste E2E:** Criar smart folder com query, ver resultados filtrados, editar, deletar. Badges de contagem na sidebar atualizando.
*   **Sprint 7.4: Ratings, Notes, Metadata EXIF e Cores**
    *   **Escopo:** `update_asset_rating`, `update_asset_notes` via Ledger, `get_asset_exif` via FormatProvider MetadataCapability, `get_asset_colors`, `reextract_asset_colors`. Total: 5 novos IPC commands.
    *   **Teste E2E:** Atribuir rating, escrever nota, visualizar EXIF no inspector, ver paleta de cores.

---

## Fase 8: Paridade IPC — Mídia, Manutenção e Validação Final
**Objetivo Testável:** Restaurar as funcionalidades de mídia pesada (streaming, transcoding, waveforms), utilidades de manutenção, auditar cobertura de FormatProviders, e validar a migração completa end-to-end. Ao final, o backend V2 é 100% equivalente ao V1 em features com arquitetura drasticamente superior.

*   **Sprint 8.1: Thumbnails Avançados, Formatos Suportados e DB Maintenance**
    *   **Escopo:** `request_thumbnail_regenerate`, `get_library_supported_formats` (via FormatRegistry), `run_db_maintenance`, `send_telemetry_log`, `get_audio_waveform_data`. Total: 5 novos IPC commands.
    *   **Teste E2E:** Regenerar thumb de asset, listar formatos, disparar VACUUM, enviar log do frontend.
*   **Sprint 8.2: Streaming Server HTTP e Transcoding Commands**
    *   **Escopo:** Servidor HTTP embarcado com Range Requests (206), token de sessão, `get_stream_url`, `needs_transcoding`, `is_native_format`, `get_quality_options`, `transcode_file`, cache management, `ffmpeg_available`. Total: 11 novos IPC commands.
    *   **Teste E2E:** Reproduzir vídeo MP4 nativo no player, reproduzir formato que requer transcoding, verificar cache stats.
*   **Sprint 8.3: Auditoria de FormatProviders e Cobertura de Extratores**
    *   **Escopo:** Auditoria completa de todos os formatos V1 (SAI, SAI2, Rebelle, CorelDRAW, CorelPainter, Sketch, Penpot, MDP, EPS, XCF, CLIP, etc.) contra FormatProviders V2. Criação de providers faltantes.
    *   **Teste E2E:** Cada extensão do V1 `definitions.rs` resolve para um FormatProvider V2 específico (não fallback).
*   **Sprint 8.4: Validação E2E, Compilação Limpa e Relatório Final**
    *   **Escopo:** `cargo build --release` 0 warnings, `cargo clippy` limpo, testes manuais de todos os fluxos UI, graceful shutdown, geração do walkthrough final.
    *   **Teste E2E:** App funcional, responsivo, com total paridade de features V1 e arquitetura Hexagonal V2 consolidada.

---

## Resumo de Sprints por Fase

| Fase                 | Sprints    | IPC Commands Adicionados | Foco                                                       |
| -------------------- | ---------- | ------------------------ | ---------------------------------------------------------- |
| **1-6** (concluídas) | 1.1 → 6.3  | 11                       | Fundação, Domínio, Formatos, Workers, Delivery, Cleanup    |
| **7**                | 7.1 → 7.4  | 23                       | Tags, Folders, Smart Folders, Ratings, Notes, EXIF, Colors |
| **8**                | 8.1 → 8.4  | 21 + Audit               | Thumbnails, Streaming, Transcoding, FormatProviders, E2E   |
| **Total**            | 31 sprints | **55+ IPC commands**     | Backend V2 completo e validado                             |
