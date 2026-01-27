# Implementação do Tauri MCP para Testes E2E Reais

> **Estado:** Proposed
> **Objetivo:** Configurar o `tauri-mcp` e criar uma camada de abstração (Bridge) para permitir que o Agente AI instancie e controle a aplicação Tauri real, acessando tanto a interface (WebView) quanto o backend Rust (IPC) sem mocks.

## 1. Contexto & Requisitos
- **Problema:** O `browser_agent` nativo não interage com janelas desktop/Tauri. Mocks de backend não cobrem lógica real de Rust/Sistema de Arquivos.
- **Solução:** Integrar `tauri-mcp` como servidor de controle e scripts Python como clientes.
- **Instalação:** Global via Cargo.

## 2. Pano de Ação

### Fase 1: Setup do Ambiente (CONCLUÍDO)
- [x] Instalar `tauri-mcp` (Custom Build).
    - Compilado do Git (master) para suporte a protocolo novo.
    - Patched localmente para corrigir panic de runtime e compliance MCP.
- [x] Instalar biblioteca Python `mcp` (v1.26+).

### Fase 2: Criação do "Agente Driver" (Python) (CONCLUÍDO)
- [x] Criar `.agent/scripts/tauri_driver.py`:
    - Encapsula conexão com MCP Server.
    - Suporta `launch_app`, `stop_app`, `take_screenshot`, `get_app_logs`.
    - Classe `TauriTestSession` implementada.

### Fase 3: Integração e Teste (CONCLUÍDO)
- [x] Criar script de teste inicial `smoke_test_tauri.py`.
- [x] Validar execução com sucesso (App lança, screenshot tirado, app fecha).

### Fase 4: Documentação
- [ ] Atualizar `docs/HOWTO-tauri-testing.md` com detalhes do setup customizado.
