# Sprint 5.3: Bindings IPC e Frontend Wiring

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 5:** A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo:** Mapear ostensivelmente a Porta de Delivery Passiva do sistema (Eventos Bidirecionais emitidos do Frontend -> Backend e Backend -> Frontend via IPC Nativo). Os Contratos Estritos (JSONs) devem fluir aqui imaculados e tipados de acordo com os `Commands` Hexagonais e EventBus.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **JSON DTO Payload Seguro:** Uma submissão UI React/Solid engatilhando um Update Asset é negada se as Chaves Semânticas (Type Checking) colidirem ou falharem antes de chegar aos Casos de Uso.
2. **Reatividade do UI:** O Tauri `app_handle.emit_all()` transportou um Evento de Backend (Ex: `ThumbGeneratedEvent` rodando do Worker Global Oculto da Fase 4) até o Listner do Frontend, e o Framework JS reativou uma imagem póstuma sem refresh ou nova chamada HTTP.
3. **Reflexão E2E Rápida de Busca:** O motor `Search_Assets` despeja sua matriz de retorno com `SearchCriteria` fluindo sem delays de marshaling massivos do JSON.

---

## 📋 Tarefas (Checklist do Agente)

### 1. Injeção Final dos `tauri::command` (Controllers Hexagonais)
- [ ] Organizar `src-tauri/src/delivery/tauri/commands/`. Fazer o espelho de:
    - Mutação: `create_tag`, `apply_folder`, `update_metadata`.
    - Busca: `search_assets`, `get_folder_tree`, `lookup_asset`.
- [ ] Eles não contêm REGRA NENHUMA! O método deve apenas: Converter Parâmetros Tauri -> Evocar a Interface do Domínio (`AssetLedger` ou `QueryHandler`) que estão instanciados no wrapper de `Tauri::State`.

### 2. Event Bridge (Listener & Emit)
- [ ] No Cargo de Bootloader (`main.rs`), subscrever OPR Ouro no `EventBus` que a gente formou lá nas raízes (Sprint 1.2). Mapear esse Listener Global passivo repassando qualquer Enum derivado atirado no core para o Tauri `app.emit_all()`.
- [ ] Construir o Typings `types.ts` correspondente na UI pra fechar a assinatura limpa de `window.__TAURI__.event`.

### 3. Error Translation Map
- [ ] Rever as traduções do Erro Universal da Sprint 1.1 e testá-las integralmente na UI. Os Alerts ou Toasts de Tela do Frontend reagem às Tags de ErrorCode providenciadas nativamente (ex: `ASSET_UNREACHABLE` = Toast Vermelho nativo de "Dispositivo Desconectado" no Typescript).

---

## 💡 Notas para o Desenvolvedor / Agente
> O `Delivery` (A Interface Controladora em Portos Hexagonais) serve apenas para converter JSON Tipado e Protocolos HTTP pra Structs do Domínio interno. Ela não conhece FFmpeg, ela não conhece SQLx, ela não abre canais de mensageria paralelos. Um Erro neste contexto significa falha catastrófica de Types/Parametização na Fronteira e deve explodir logadamente nos Hooks do React/SolidJS antes de foder o banco de dados.
