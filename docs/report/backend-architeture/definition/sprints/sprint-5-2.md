# Sprint 5.2: Transcoding On-the-fly (HLS Vídeo & Áudio)

**Status:** Concluída
**Data e hora de inicio:** 2026-03-09T23:15:00-03:00
**Data da conclusão:** 2026-03-09T23:38:00-03:00

**Fase 5:** A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo:** Acoplar e Refatorar o braço pesado de HLS (HTTP Live Streaming) para contêineres rejeitados pelo Chromium Webview (Ex: `.mkv`, `hevc` bruto cruzado, ou `.flac` enormes limitados). O backend deve transcodificá-los para Playlist `.m3u8` e sub-segmentos em real-time enquanto o Front reproduz.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Geração Dinâmica do HLS:** Solicitar o recurso de Stream HLS engatilha um processo FFmpeg Background Ativo gerando de .ts buffers em `/tmp` ou `.mundam/streams/`.
2. **Entrega de Manifesto (`m3u8`):** O Handler HTTP anterior reconhece o pedido e atira não o arquivo bruto, mas o Manifesto M3U8 local reativo engolindo os segmentos `0.ts, 1.ts`.
3. **Reprodução Chromium:** Reproduzir o link M3U8 via player HTML5 HLS.js validando o MKV exótico sendo tocado em um *Seek* agressivo em tempo hábil.

---

## 📋 Tarefas (Checklist do Agente)

### 1. HLS Stream Factory
- [x] Em `feature/transcoding/hls_manager.rs`, criar a fiação de comandos do Sub-Processo FFmpeg (`ProcessCommand::new("ffmpeg")`) ordenando o Output de Fragmentos curtos de 2 a 5 segundos via Codec acelerado (`-c:v libx264 -preset ultrafast`).
- [x] Isolar um "SessionManager" de Stream (`HashMap<SessionID, ChildProcess>`).

### 2. A Ponte HTTP Delivery
- [x] No `delivery/streaming/server.rs` da Sprint Passada, crie a rota explícita `asset://stream/hls/{asset_id}/playlist.m3u8`. 
- [x] A chamada inicia o HLS Factory se ele não estiver rodando (Lazy Instantiation), aponta a escuta de retorno do Request para as fatias `.ts` na cache. 

### 3. Killers Recursivos (Subprocess Limits)
- [x] Injetes lógicas de Expiry. O backend deve fechar o Spawn FFmpeg (`SIGKILL()`) se o player suspender Request do arquivo local há mais de N Segundos (Debounce Cleanup de Player nativo) impedindo um HLS órfão de queimar Disco e CPU até bater 100%.

### 4. Aceleradores de Vídeo e Áudio
- [x] Mudar flags nativas por Perfil (`feature/transcoding/profiles.rs`). `.FLAC` bruto recusa flags de libx264, o perfil deve mandar converter *apenas som* sob cópia AAC empacotando num pseudo-HLS audio buffer de altíssima performance.

---

## 💡 Notas para o Desenvolvedor / Agente
> Transcodificação *On The Fly* pode destruir a performance geral em CPUs lentas. É crucial invocar no shell do FFmpeg parâmetros brandos e `scale=-2:1080` (Resize se o conteúdo original for 4k sem suporte a Hardware API Decode). Focar em Latência de Play Incial vs Qualidade Master. Use flag de Copy no fluxo se o Codec do Vídeo bate com o suportado, fragmentando só o contêiner MKV pra MP4 sem alterar as faixas matriz.

---

## 🚀 Detalhes da Implementação

A sprint substituiu completamente a antiga arquitetura Web de Streaming que utilizava o Axum (Servidor Backend HTTP acoplado em Background V1), centralizando toda a entrega M3U8 nativamente sob os ganchos do protocolo customizado `asset://` do Tauri.

**Decisões e Adições Chave:**
- **Inversão de HTTP Server para Custom Protocol**: Toda orquestração (leitura dos arquivos `.ts` do sistema e envio das playlists) agora herda a velocidade e segurança do interceptador de protocolo `delivery/protocols/asset.rs`. Nenhuma porta aberta localmente foi exposta.
- **`DashMap` HLS Manager**: O módulo atua como uma fábrica estrita instanciando comandos para o processo original (FFmpeg). Ele armazena um ponteiro de referenciamento (Session Tracking) e o PID da sub-thread do FFmpeg no struct do Estado do App (lib).
- **Graceful Shutdown**: Usando canais e tarefas assíncronas do Tokio, a leitura do vídeo pelo lado do Desktop ativa um *heartbeat* (touch_session). Um limpador operatório de fundo varrerá a tabela espelho do DashMap a cada 10s procurando sessões não-tocadas há mais de 1 minuto, cortando processos zumbis de CPU (`kill()`) e enxugando fragmentos `.ts` do `/streams` caso o usuário pausassem ou abandonasse o reprodutor.

## 📁 Arquivos Modificados
- `src-tauri/src/delivery/protocols/asset.rs`
- `src-tauri/src/feature/mod.rs`
- `src-tauri/src/feature/transcoding/hls_manager.rs` (Novo)
- `src-tauri/src/feature/transcoding/mod.rs` (Novo)
- `src-tauri/src/feature/transcoding/profiles.rs` (Novo)
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml` (Adicionado pacote DashMap)
