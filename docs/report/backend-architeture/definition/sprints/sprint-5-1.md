# Sprint 5.1: Servidor HTTP & Delivery Estático (Asset Protocol)

**Status:** Concluído
**Data e hora de inicio:** 2026-03-09 21:00
**Data da conclusão:** 2026-03-09 23:04

**Fase 5:** A Fronteira da Aplicação (Tauri, HTTP, UI)
**Objetivo:** Eliminar o gargalo do IPC do Tauri para transporte de Mídias Pesadas. O Frontend Solid.js precisa consumir miniaturas e previews originais de imediato e reproduzir vídeos nativos (MP4/WebM) com suporte a Scrubbing (`Range: bytes=0-1000`). Para isso, ergueremos um servidor Axum/Warp em background ou estenderemos agressivamente o `asset://` handler de Tauri.

---

## 🎯 Critérios de Aceite (E2E Testable)
1. **Host Estático Operacional:** Mídias fornecidas pelo endereço abstrato `asset://localhost/{asset_id}` ou `http://localhost:{port}/media/{id}` carregam limpidamente `<img src="...">` no Browser via Blob do disco cacheado.
2. **Buffer Streaming (HTTP 206):** Ao abrir um vídeo nativo pesadíssimo de 12GB (MP4), a visualização responde de imediato, e o Header de Retorno do Servidor atira `HTTP 206 Partial Content`, com *Drag/Scrub* da Timeline de Vídeo 100% fluido.
3. **Resolução de Path Blindada:** Nenhum endereço direto C:/ ou /home/ deve fluir para a Internet ou UI de modo cru para a leitura da media de saída. O Handler HTTP local converte o `Hash`/`ID` para o path físico seguro validando no CQRS lido (`QueryHandler`).

---

## 📋 Tarefas (Checklist do Agente)

### 1. Tauri Custom Protocol ou Micro-Host
- [x] Construir em `delivery/streaming/server.rs` ou `infra/http/` o serviço de ponta. O padrão recomendável de performance pro Mundam é usar `tauri::http::ResponseBuilder` através de `app.register_uri_scheme_protocol("asset")`.
- [x] No interceptor, resgatar o parâmetro de Hash/Id da URI invocada. 

### 2. Leitura File IO & Range Header
- [x] Recuperar através dos `AssetQueryHandlers` a URL real absoluta correspondente no PC (`.disk_path`).
- [x] Construir o *Chunking*. Responder ao parser "Range" interceptando pedaços limpos através de `tokio::fs::File::open` limitando leitura do buffer pelo Range requisitado (`tokio::io::AsyncReadExt`).

### 3. Thumb Delivery
- [x] Roteamento para Cache de Miniaturas: Criar o sufixo opcional (Ex: `asset://image?id=500&type=thumb`) que vai instruir o Handler a não abrir a rotação original, mas ler na subpasta `.appdata/thumbnails/` otimizada.

### 4. Bateria Teste Stream JS
- [x] Instanciar o HTML Vídeo Tag no Front para rodar o arquivo mastodonte com o DevTools (Aba Network) aberto. Inspecione se *1 Request de 4GB* ou *Múltiplos Requests 206 de 2MB* fluem ao longo de uma reprodução arrastada.

---

## 💡 Notas para o Desenvolvedor / Agente
> Em hipótese alguma ordene ao `std::fs::read(path)` jogar uma file de vídeo de 500MB bruta inteira para a Memória da aba RAM e tentar mandar tudo pro JS Webview. Use Stream Async do File Descriptor cru, empacotando apenas por causa do Bytes Limits dos canos do SO Local. Você está montando um CDNs Local reativo, proteja o Out of Memory em arquivos insanos (8K Raws etc).

---

## 🛠️ Informações da Implementação

**Dificuldades Encontradas:**
- A implementação assíncrona usando `tokio::fs` no `AppHandler` síncrono exigiu uma ponte `block_on` e contornos sintáticos em torno dos Lifetimes de retorno da closure de protocolo em Rust.
- Erros de empréstimo (Borrow Checker) com `AsyncReadExt::take()` sendo movido do escopo durante iteradores assíncronos foram mitigados migrando para um bloco de vetor e operando com `read_exact()` limitando o range explicitamente.

**Melhorias Feitas & Escopo Adicional:**
- O protocolo foi construído encapsulando validações de tipos mime automáticas (`mime_guess`) garantindo detecção para fallback de streams variados sem dependência imediata no registry.
- Ajuste e adaptação não previstas em `useVideoSource.ts` e outros componentes renderizadores (`ItemView`, `VideoInspector`) de modo a acomodar o roteamento lógico baseado em `AssetId` para não mais buscar o path físico injetado em URIs do formato web `video://`.
- Implementada suite de testes integrados via Vitest para o controlador do frontend.

## 📁 Arquivos Modificados
- `src-tauri/src/delivery/mod.rs` (Modificado)
- `src-tauri/src/delivery/protocols/asset.rs` (Criado)
- `src-tauri/src/delivery/protocols/mod.rs` (Criado)
- `src-tauri/src/lib.rs` (Modificado)
- `src/components/features/inspector/video/VideoInspector.tsx` (Modificado)
- `src/components/features/itemview/ItemView.tsx` (Modificado)
- `src/components/features/itemview/renderers/video/VideoPlayer.tsx` (Modificado)
- `src/core/hooks/useVideoSource.ts` (Modificado)
- `src/lib/stream-utils.spec.ts` (Criado)
- `src/lib/stream-utils.ts` (Modificado)
- `docs/report/backend-architeture/definition/sprints/sprint-5-1.md` (Modificado)
