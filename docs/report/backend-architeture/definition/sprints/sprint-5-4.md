# Sprint 5.4: Settings, App Lifecycle e Graceful Shutdown

**Status:** Pendente
**Data e hora de inicio:** -
**Data da conclusão:** -

**Fase 5:** A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo:** Instanciar um mecanismo resiliente de encerramento (`Graceful Shutdown`) do processo e a mecânica persistência de `AppConfigs`. Um fechamento bruto do Backend no meio de transações (SQL WAL) ou num SubProcesso FFmpeg rodando solto seria brutal. Devemos rastrear e fechar com perfeição todos os loops.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Configuração Persistida Em Disco Seguro:** Alterações de UI relativas as paths de Database, Limites de Worker-Threads e UI-Language sobrevivem ao fechar e abrir via Serialization num arquivo leve JSON/TOML fora do Banco sujo principal.
2. **Cancellation Token:** O Evento `Window_Close` / `tokio::signal` emite Ordem Oficial ao Token Global do sistema. O loop inifiníto da Fila Worker de Thumbs entende o drop via flag e expira limpamente, salvando Logs.
3. **Child Process Cleanup:** Tarefas delegadas pelo Transcoding ao FFmpeg em M3U8 tem suas IDs engessadas e mortas explicitamente antes de confirmar ao Sistema Operacional que a porta foi desligada. Retornar Memory/Code = 0.

---

## 📋 Tarefas (Checklist do Agente)

### 1. AppSettings Struct e Adapter (Domínio & Infra)
- [ ] Criar a base Hexagonal (`core/settings/`) designando as Definições: Diretório Oculto default, Opções de Idioma Misto, Auto-Scan Flags etc.
- [ ] Em `infra/config/` plugar a macro de serialização usando `serde_json` guardando isso num `AppDataFolder/settings.json` padrão extraível em caso de Reset Físico do software. Atrelar as Modificações Ativas com salvamento iminente transacional.

### 2. CancellationToken e Shutdown Channels
- [ ] Instanciar o `tokio_util::sync::CancellationToken` Global (Root Token). Ele é injetado via Dependency Injection em TODOS os Listeners do Bus e Handlers Paralelos (*FileWatchers*, *Workers*, *Jobs* e *Servers*).
- [ ] Todo loop (`while let Some = run`) do Backend se encapará com `tokio::select!` vigiando a branch padrão contra a Ordem `.cancelled()` despencando o loop seguro pro final da função e executando limpeza do pool.

### 3. Interceptador Tauri de Saída
- [ ] Configurar os Hooks do Tauri Window Events (`RunEvent::WindowEvent { event: WindowEvent::CloseRequested { api, .. } }`). 
- [ ] Bloqueie o Abort Padrão Instantâneo C++ (`api.prevent_close()`). Envie Ordem ao `.cancel()` global, aguarde via Threads uma barreira limpa de Drop de Bancos SQLx Connection Pool e finalmente `app_handle.exit(0)` em tela preta reluzente e sem logs de crash.

---

## 💡 Notas para o Desenvolvedor / Agente
> Uma Task no Tokio que roda dentro de Spawn Blocking só morrerá passivamente se os Subprocess IDs (Child Commands) forem atrelados com Hooks `Kill_on_drop(true)` presentes da Std Command/Tokio Command nativos, do contrário, zumbis nativos comem a RAM no Background do Sistema operacional do usuário desavisadamente. Amarre a Vida do App! O App não desliga sem permissão e salvar da Infraestrutura!
