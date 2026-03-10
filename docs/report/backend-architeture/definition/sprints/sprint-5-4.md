# Sprint 5.4: Settings, App Lifecycle e Graceful Shutdown

**Status:** Concluído [x]
**Data e hora de inicio:** 2026-03-10T02:10:00-03:00
**Data da conclusão:** 2026-03-10T02:30:00-03:00

**Fase 5:** A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo:** Instanciar um mecanismo resiliente de encerramento (`Graceful Shutdown`) do processo e a mecânica persistência de `AppConfigs`. Um fechamento bruto do Backend no meio de transações (SQL WAL) ou num SubProcesso FFmpeg rodando solto seria brutal. Devemos rastrear e fechar com perfeição todos os loops.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Configuração Persistida Em Disco Seguro:** Alterações de UI relativas as paths de Database, Limites de Worker-Threads e UI-Language sobrevivem ao fechar e abrir via Serialization num arquivo leve JSON/TOML fora do Banco sujo principal. [x]
2. **Cancellation Token:** O Evento `Window_Close` / `tokio::signal` emite Ordem Oficial ao Token Global do sistema. O loop inifiníto da Fila Worker de Thumbs entende o drop via flag e expira limpamente, salvando Logs. [x]
3. **Child Process Cleanup:** Tarefas delegadas pelo Transcoding ao FFmpeg em M3U8 tem suas IDs engessadas e mortas explicitamente antes de confirmar ao Sistema Operacional que a porta foi desligada. Retornar Memory/Code = 0. [x]

---

## 📋 Tarefas (Checklist do Agente)

### 1. AppSettings Struct e Adapter (Domínio & Infra)
- [x] Criar a base Hexagonal (`core/settings/`) designando as Definições: Diretório Oculto default, Opções de Idioma Misto, Auto-Scan Flags etc.
- [x] Em `infra/config/` plugar a macro de serialização usando `serde_json` guardando isso num `AppDataFolder/settings.json` padrão extraível em caso de Reset Físico do software. Atrelar as Modificações Ativas com salvamento iminente transacional.

### 2. CancellationToken e Shutdown Channels
- [x] Instanciar o `tokio_util::sync::CancellationToken` Global (Root Token). Ele é injetado via Dependency Injection em TODOS os Listeners do Bus e Handlers Paralelos (*FileWatchers*, *Workers*, *Jobs* e *Servers*).
- [x] Todo loop (`while let Some = run`) do Backend se encapará com `tokio::select!` vigiando a branch padrão contra a Ordem `.cancelled()` despencando o loop seguro pro final da função e executando limpeza do pool.

### 3. Interceptador Tauri de Saída
- [x] Configurar os Hooks do Tauri Window Events (`RunEvent::WindowEvent { event: WindowEvent::CloseRequested { api, .. } }`). 
- [x] Bloqueie o Abort Padrão Instantâneo C++ (`api.prevent_close()`). Envie Ordem ao `.cancel()` global, aguarde via Threads uma barreira limpa de Drop de Bancos SQLx Connection Pool e finalmente `app_handle.exit(0)` em tela preta reluzente e sem logs de crash.

---

## 🚀 Informações da Implementação

### Dificuldades e Desafios
- **Gestão de Processos Filhos**: Inicialmente, a limpeza de sessões do FFmpeg no `HlsManager` exigia iteração manual e `kill()` explícito. Isso gerava problemas de mutabilidade e complexidade no loop de shutdown.
- **Resolução de Módulos e Traits**: A introdução do novo adaptador de configurações exigiu ajustes finos na árvore de módulos (`infra/config/mod.rs`) e na visibilidade de traits (`SettingsRepository`) para que o `lib.rs` pudesse carregar as configurações corretamente.

### Melhorias Realizadas
- **Kill on Drop**: Otimizamos o `HlsManager` para usar `.kill_on_drop(true)` em todos os subprocessos FFmpeg. Isso garante que, assim que uma sessão é removida do mapa (seja por timeout ou shutdown), o processo SO seja encerrado imediatamente sem necessidade de lógica de limpeza complexa.
- **Resiliência do Shutdown**: Implementamos um sistema de `LifecycleRegistry` que centraliza todos os `JoinHandle` e tokens, permitindo que o Tauri aguarde de forma assíncrona o encerramento de todas as tarefas de background antes de finalizar o processo.

### Pontos Fora do Escopo Inicial
- **V1 Integration**: Além de preparar a nova arquitetura, integramos o `CancellationToken` também em componentes legados (V1 Thumbnail Worker) para garantir que o shutdown fosse global e não deixasse threads órfãs do sistema antigo.

---

## 📁 Arquivos Modificados
- `src-tauri/src/core/settings/model.rs` [NEW]
- `src-tauri/src/core/settings/port.rs` [NEW]
- `src-tauri/src/core/settings/mod.rs` [NEW]
- `src-tauri/src/infra/config/json_adapter.rs` [NEW]
- `src-tauri/src/infra/config/mod.rs` [NEW]
- `src-tauri/src/feature/settings/service.rs` [NEW]
- `src-tauri/src/feature/settings/mod.rs` [NEW]
- `src-tauri/src/delivery/tauri/commands/settings.rs` [NEW]
- `src-tauri/src/core/mod.rs`
- `src-tauri/src/infra/mod.rs`
- `src-tauri/src/feature/mod.rs`
- `src-tauri/src/delivery/tauri/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/feature/transcoding/hls_manager.rs`
- `src-tauri/src/processing/watcher/sensor.rs`
- `src-tauri/src/processing/workers/thumbnail_worker.rs`
- `src-tauri/src/lifecycle.rs`

---

## 💡 Notas para o Desenvolvedor / Agente
> Uma Task no Tokio que roda dentro de Spawn Blocking só morrerá passivamente se os Subprocess IDs (Child Commands) forem atrelados com Hooks `Kill_on_drop(true)` presentes da Std Command/Tokio Command nativos, do contrário, zumbis nativos comem a RAM no Background do Sistema operacional do usuário desavisadamente. Amarre a Vida do App! O App não desliga sem permissão e salvar da Infraestrutura!
