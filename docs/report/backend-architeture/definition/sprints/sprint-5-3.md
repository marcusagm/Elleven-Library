# Sprint 5.3: Bindings IPC e Frontend Wiring

**Status:** Concluído
**Data e hora de inicio:** 2026-03-09 23:40
**Data da conclusão:** 2026-03-10 00:38

**Fase 5:** A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo:** Mapear ostensivelmente a Porta de Delivery Passiva do sistema (Eventos Bidirecionais emitidos do Frontend -> Backend e Backend -> Frontend via IPC Nativo). Os Contratos Estritos (JSONs) devem fluir aqui imaculados e tipados de acordo com os `Commands` Hexagonais e EventBus.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. ✅ **JSON DTO Payload Seguro:** Uma submissão UI React/Solid engatilhando um Update Asset é negada se as Chaves Semânticas (Type Checking) colidirem ou falharem antes de chegar aos Casos de Uso.
2. ✅ **Reatividade do UI:** O Tauri `app_handle.emit()` transportou um Evento de Backend (Ex: `ThumbGeneratedEvent` rodando do Worker Global Oculto da Fase 4) até o Listner do Frontend, e o Framework JS reativou uma imagem póstuma sem refresh ou nova chamada HTTP.
3. ✅ **Reflexão E2E Rápida de Busca:** O motor `Search_Assets` despeja sua matriz de retorno com `SearchCriteria` fluindo sem delays de marshaling massivos do JSON.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Injeção Final dos `tauri::command` (Controllers Hexagonais)
- [x] Organizar `src-tauri/src/delivery/tauri/commands/`. Fazer o espelho de:
    - Mutação: `create_tag`, `apply_folder`, `update_metadata`.
    - Busca: `search_assets`, `get_folder_tree`, `lookup_asset`.
- [x] Eles não contêm REGRA NENHUMA! O método deve apenas: Converter Parâmetros Tauri -> Evocar a Interface do Domínio (`AssetLedger` ou `QueryHandler`) que estão instanciados no wrapper de `Tauri::State`.

### 2. Event Bridge (Listener & Emit)
- [x] No Cargo de Bootloader (`main.rs`), subscrever OPR Ouro no `EventBus` que a gente formou lá nas raízes (Sprint 1.2). Mapear esse Listener Global passivo repassando qualquer Enum derivado atirado no core para o Tauri `app.emit_all()`.
- [x] Construir o Typings `types.ts` correspondente na UI pra fechar a assinatura limpa de `window.__TAURI__.event`.

### 3. Error Translation Map
- [x] Rever as traduções do Erro Universal da Sprint 1.1 e testá-las integralmente na UI. Os Alerts ou Toasts de Tela do Frontend reagem às Tags de ErrorCode providenciadas nativamente (ex: `ASSET_UNREACHABLE` = Toast Vermelho nativo de "Dispositivo Desconectado" no Typescript).

---

## 💡 Notas para o Desenvolvedor / Agente
> O `Delivery` (A Interface Controladora em Portos Hexagonais) serve apenas para converter JSON Tipado e Protocolos HTTP pra Structs do Domínio interno. Ela não conhece FFmpeg, ela não conhece SQLx, ela não abre canais de mensageria paralelos. Um Erro neste contexto significa falha catastrófica de Types/Parametização na Fronteira e deve explodir logadamente nos Hooks do React/SolidJS antes de foder o banco de dados.

---

## 📝 Informações da Implementação

### Dificuldades Encontradas
- O ciclo de substituição de APIs do Tauri (v1 para v2) e a necessidade de preservar as URIs dos invokes da UI legada obrigaram a usar uma wrapper interceptadora, prevenindo refatoração exaustiva de componentes visuais neste momento.
- A restrição severa de Complexidade Ciclomática (max 10) do linter Frontend conflitou com a grande árvore do enum `AppError`, forçando a substituição de dezenas de instâncias de `switch/case` por um Pattern de mapeamento estático (`Record<string, Object>`).
- **Mismatch de Tipos no SQLx:** Durante a integração da extração de cores, identificamos um erro de decodificação (`DATABASE_ERROR`) onde o SQLite retornava `TEXT` para o `asset_id` (compatível com UUIDs), mas o Rust esperava `i64`. Isso exigiu a alteração do modelo `AssetColor` para usar `String` em todos os níveis da aplicação.

### Melhorias Realizadas e Integrações Fora do Escopo
- **Discriminated Unions Naturais:** Foi construído um tipo global TypeScript exato para o Enum de Eventos do Domínio do Rust. Além da união, adicionou-se a Tag `#[serde(tag = "type", content = "payload")]` no Backend para eliminar verificações impuras no Frontend, garantindo Intellisense perfeito para eventos interceptadores do Tauri.
- **Ponte Central de Controle IPC (`invokeCommand`):** A estrutura foi blindada sem interferir nos componentes existentes. A interceptação agora funciona como uma Middle Layer capturando os mapeamentos *thiserror* (`AppResult`) e lançando notificações de erro globalmente de forma atômica no *Sonner*.
- **Refatoração Completa do Frontend:** Além do plano inicial, realizamos uma varredura completa em todos os Stores, Services, Utils e Componentes remanescentes, garantindo que 100% das interações com o backend passem pelo wrapper seguro, eliminando o uso direto do `invoke` do Tauri Core em toda a aplicação.

### 📄 Arquivos Criados ou Modificados
#### Backend (Rust)
- `src-tauri/migrations/20260310000000_add_color_analysis.sql` (Migração de Dados)
- `src-tauri/src/db/models.rs` (Ajuste de Tipos para Interoperabilidade V1/V2)
- `src-tauri/src/db/colors.rs` (Persistência de Paletas CIE-LAB)
- `src-tauri/src/library/commands/colors.rs` (Novos Comandos de Análise)
- `src-tauri/src/thumbnails/worker.rs` (Trigger Automático de Cores pós-Thumbnail)
- `src-tauri/src/delivery/tauri/commands/mod.rs` (Hub de Comandos)
- `src-tauri/src/delivery/tauri/commands/mutations.rs` (Refatoração Hexagonal)
- `src-tauri/src/delivery/tauri/commands/queries.rs` (Refatoração Hexagonal)
- `src-tauri/src/delivery/tauri/mod.rs` (Configuração de Delivery)
- `src-tauri/src/core/events/payloads.rs` (Event Schema)
- `src-tauri/src/lib.rs` (Bootloader & Event Bridge)

#### Frontend (TypeScript/SolidJS)
- `src/lib/api.ts` (Universal `invokeCommand` Wrapper)
- `src/types/events.ts` (Domain Event Unions)
- `src/core/tauri/services.ts` (Service Refactor)
- `src/core/utils/LifecycleManager.ts` (Listener Refactor)
- `src/core/store/formatStore.ts` (Store Refactor)
- `src/core/store/library/itemActions.ts` (Actions Refactor)
- `src/core/store/library/libraryActions.ts` (Actions Refactor)
- `src/core/store/metadata/searchActions.ts` (Static Import Refactor)
- `src/core/input/store/shortcutStore.ts` (Store Refactor)
- `src/lib/hls-player.ts` (Library Refactor)
- `src/lib/stream-utils.ts` (Library Refactor)
- `src/components/features/inspector/image/ColorPaletteSection.tsx` (Component Refactor)
- `docs/report/backend-architeture/definition/sprints/sprint-5-3.md` (Update Tracker)

