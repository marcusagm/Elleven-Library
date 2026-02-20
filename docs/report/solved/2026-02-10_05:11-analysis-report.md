# Relatório de Análise Técnica e Arquitetural - Mundam
> **Data**: 10 de Fevereiro de 2026
> **Versão do Projeto**: Em desenvolvimento ativo

## 1. Visão Geral e Arquitetura

O projeto **Mundam** é uma aplicação desktop de gerenciamento de ativos digitais (DAM) de alta performance, construída sobre o framework **Tauri**. Sua arquitetura segue o modelo "Local-First", priorizando velocidade, privacidade e manipulação direta de arquivos no sistema operacional do usuário.

### Stack Tecnológico
*   **Core/Backend**: Rust (Tauri). Gerencia I/O de arquivos, banco de dados, servidor de streaming HLS, processamento de imagens (via FFmpeg/ImageMagick) e lógica de negócios pesada.
*   **Frontend**: SolidJS + Vite + TypeScript. Escolha excelente para performance, evitando o overhead de Virtual DOM do React.
*   **Banco de Dados**: SQLite (via `sqlx`). Armazenamento relacional local para metadados, estrutura de pastas e tags.
*   **Estilização**: CSS Modules / Variáveis CSS nativas (evidenciado por arquivos `.css` em componentes).


---

## 📚 Índice de Relatórios Detalhados

Este documento serve como um **Resumo Executivo**. Para aprofundamento técnico, consulte os relatórios específicos gerados:

*   **[🎨 Análise Frontend](2026-02-10_05:11-analysis-report-frontend.md)**: Detalhes de arquitetura SolidJS, componentes, estado e Design System.
*   **[⚙️ Análise Backend](2026-02-10_05:11-analysis-report-backend.md)**: Análise profunda do Rust, Streaming Server, Indexador e Estrutura de Módulos.
*   **[🗄️ Análise Banco de Dados](2026-02-10_05:11-analysis-report-database.md)**: Review do Schema SQLite, SQLx Performance e Migrações.
*   **[🗺️ Roadmap & Features](2026-02-10_05:11-analysis-report-roadmap.md)**: Análise de Gap de Features (vs Ideia Original) e plano de ação futuro.

---


## 2. Análise de Código e Implementação

### 2.1 Backend (Rust/Tauri)

O backend é robusto e bem modularizado. A separação de responsabilidades está clara.

**Pontos Fortes:**
*   ✅ *Realizado* - **Modularidade**: O arquivo `lib.rs` demonstra uma clara separação em módulos: `database`, `ffmpeg`, `indexer`, `protocols`, `streaming`, `thumbnails`.
*   **Streaming de Mídia**: A implementação de um servidor HLS customizado (`streaming` module) para entregar vídeo e áudio é um diferencial técnico avançado, permitindo reprodução fluida de formatos que navegadores não suportam nativamente (MKV, AVI, etc.).
*   ✅ *Realizado* - **Suporte a Formatos (`formats.rs`)**: O registro centralizado de formatos (`SUPPORTED_FORMATS`) é elegante e facilita a expansão. O uso de Enums (`ThumbnailStrategy`, `PlaybackStrategy`) torna a lógica de tratamento de arquivos segura e previsível.
*   **Indexação Assíncrona**: O uso de `Tokio` para operações de I/O e a arquitetura de "Watcher" + "Scanner" é correta para este tipo de aplicação.

**Pontos de Atenção e Melhoria:**
*   **Gerenciamento de Banco de Dados (`database.rs`)**:
    *   ✅ *Realizado* - **Migrações Manuais**: O método `Db::new` contém uma longa lista de `if !column_names.contains...`. Isso é frágil e difícil de manter.
    *   **Segurança de Tipos**: Embora `sqlx` ajude, há muitas queries SQL escritas como strings puras (`sqlx::query`).
    *   **Recomendação**: Adotar o sistema de migrações nativo do `sqlx` (`sqlx migrate`) para versionar o esquema do banco de dados.
*   **Tratamento de Erros**:
    *   ✅ *Realizado* - Existem usos de `.unwrap()` e `.expect()` em locais que poderiam causar crash da aplicação (ex: `lib.rs` na inicialização de caminhos).
    *   **Recomendação**: Substituir por tratamento de erros propagável (`Result<T, AppError>`) para garantir que o app falhe graciosamente ou notifique o usuário.
*   **Escalabilidade da Indexação**:
    *   O indexador parece varrer diretórios recursivamente. Para bibliotecas com centenas de milhares de arquivos, isso pode ser lento se não houver um mecanismo de cache ou "checkpoint" robusto.

### 2.2 Frontend (SolidJS)

O frontend utiliza SolidJS, o que garante uma reatividade fina e alta performance, essencial para interfaces com milhares de itens (Masonry grids).

**Pontos Fortes:**
*   **Gerenciamento de Estado**: O uso de Stores (`libraryStore.ts`, `filterStore.ts`) centraliza bem a lógica de dados.
*   **Separação de Componentes**: Estrutura clara `components/layout`, `components/features`, `components/ui`.
*   **Virtualização**: A existência de um `Viewport` sugere preocupação com a renderização de grandes listas, embora a implementação detalhada (Masonry vs Grid) precise ser verificada se é feita via CSS ou JS.

**Pontos de Atenção e Melhoria:**
*   **Complexidade no Store (`libraryStore.ts`)**:
    *   ✅ *Realizado* - A função `handleBatchChange` contém lógica complexa de travessia de árvore (DAG para identificar pastas pai) executada no thread principal do JavaScript.
    *   ✅ *Realizado* - **Recomendação**: Mover a lógica de "pertencimento a pasta" (se um arquivo modificado pertence à view atual) para o Backend (Rust). O Rust já possui os dados em memória/banco e processa isso ordens de magnitude mais rápido.
*   ✅ *Realizado* - **Componente Raiz (`App.tsx`)**:
    *   ✅ *Realizado* - O `App.tsx` está acumulando responsabilidades: inicialização de sistema, atalhos de teclado globais, gerenciamento de janelas e renderização de layout.
    *   ✅ *Realizado* - **Recomendação**: Extrair Providers (ex: `ShortcutProvider`, `InitializationProvider`) para limpar o componente raiz.

---

## 3. Conformidade com a Ideia Inicial

Comparando o estado atual com o documento de visão (`docs/idea/features.md`):

### ✅ Implementado / Em Progresso Avançado
1.  **Ingestão de Desktop**:
    *   Monitoramento de pastas (`folder watching`) e importação recursiva estão implementados no backend.
    *   Suporte a Drag & Drop nativo.
2.  **Organização**:
    *   Hierarquia de pastas e "Pastas Inteligentes" (Smart Folders) estão presentes no código (`db_smart_folders.rs`).
    *   Sistema de Tags e Taxonomia implementado (`db_tags.rs`).
3.  **Visualização Universal**:
    *   Suporte massivo a formatos (Images, Raw, Videos, 3D, Fonts) está codificado em `formats.rs`.
    *   Player de Vídeo com HLS e Transcoding para formatos legados.
    *   ✅ *Realizado* - Geração de thumbnails via FFmpeg e estratégias nativas.
4.  **Performance**:
    *   Arquitetura Rust + SQLite + SolidJS cumpre a promessa de performance.

### ⚠️ Parcialmente Implementado / Incerteza
1.  **Análise Cromática (`2.4`)**:
    *   A extração de paleta de cores e busca por cor não foi encontrada explicitamente nos arquivos analisados (`ffmpeg.rs`, `database.rs`). A coluna de cores parece ausente no esquema do banco visto nas queries.
2.  **3D e Fontes**:
    *   ✅ *Realizado* - O suporte está declarado em `formats.rs`, mas a implementação da visualização (renderização 3D interativa ou preview de fontes customizável) depende de componentes de frontend que não foram analisados a fundo, mas as "Estratégias" (`ThumbnailStrategy::Model3D`) sugerem que ao menos a thumbnail é gerada.
3.  **Masonry Layout via Wasm/Rust**:
    *   A ideia original mencionava layout processado via Rust/Wasm. Atualmente, a lista de itens parece ser gerenciada pelo `libraryStore` (JS) e renderizada pelo `Viewport`.

### ❌ Faltante (Não Identificado no Código Atual)
1.  ✅ *Realizado* - **Web Clipper (`1.1`)**:
    *   Não há vestígios de extensão de navegador ou API para receber dados de uma extensão.
2.  **Integração com Nuvem (`5.1`)**:
    *   A integração com Google Drive/Dropbox/etc. parece depender apenas do sistema de arquivos local (o que é ok para a proposta "Cloud-Agnostic"), mas não há lógica específica para detectar conflitos de sincronização.
3.  **Exportação e Portabilidade (`5.3`)**:
    *   Funcionalidades de "Empacotar" (.eaglepack) e exportação com metadados não foram vistas nos comandos do backend.

---

## 4. Recomendações e Roadmap

Para elevar o projeto ao nível "Premium" e garantir manutenibilidade a longo prazo:

### Imediato (Refatoração & Estabilidade)
1.  **Migração SQL Real**: Substituir a lógica manual de `database.rs` por `sqlx migrate`. Isso evitará bugs críticos em atualizações futuras.
2.  **Otimização do Store**: Refatorar `libraryStore.ts` para delegar cálculos pesados (filtragem de árvore, ordenação complexa) para o Rust. O Frontend deve apenas "exibir" o que o Backend manda.
3.  **Tratamento de Erros**: Realizar uma varredura por `unwrap()` no código Rust e substituir por tratamento de erros adequado.

### Curto Prazo (Features Core)
1.  ✅ *Realizado* - **Implementar Análise de Cores**: Adicionar uma etapa no `indexer` ou `thumbnail_worker` para extrair cores dominantes das imagens e salvar no banco para permitir a "Busca por Cor".
2.  ✅ *Realizado* - **Refinar Viewer 3D e Fontes**: Garantir que, além da thumbnail, o usuário consiga interagir com o modelo 3D (rotacionar) e testar a fonte com texto customizado.

### Médio Prazo (Expansão)
1.  ✅ *Realizado* - **Web Clipper**: Desenvolver a extensão de navegador e um endpoint local no Tauri (via `tauri-plugin-localhost` ou similar) para receber os assets.
2.  **Plugins/Exportação**: Implementar o sistema de exportação de pacotes para backup ou compartilhamento.

## Conclusão
O Mundam possui uma fundação técnica excelente. A escolha de Rust + SolidJS é perfeita para o objetivo de performance. O código atual é limpo e bem estruturado, mas começa a apresentar sinais de complexidade no gerenciamento de banco de dados e no estado do frontend que merecem atenção antes de escalar novas funcionalidades.
