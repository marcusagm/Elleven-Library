# Como usar Testes E2E com Tauri MCP

Este documento explica como utilizar a integração configurada do [Tauri MCP](https://github.com/dirvine/tauri-mcp) para realizar testes automatizados com dados reais.

## Pré-requisitos (Configurados Automaticamente)
- O ambiente já possui um build customizado do `tauri-mcp` localizado em `.agent/.deps/tauri-mcp/target/debug/tauri-mcp`.
- Este build contém patches críticos para:
    1. Suporte ao protocolo MCP mais recente (compatível com `pip install mcp`).
    2. Correção de conflitos de Runtime Tokio (evita panic ao lançar apps).
    3. Formatação de respostas JSON compatível com SDK Python strict.
- Python 3.14+ (usar `/usr/local/bin/python3`) + `pip install mcp`.
- Aplicação Tauri compilada em debug (`src-tauri/target/debug/elleven-library`).

## Arquitetura
Utilizamos um script "Driver" (`.agent/scripts/tauri_driver.py`) que gerencia a sessão MCP.

A infraestrutura funciona assim:
1. Agente/Script Python -> Cliente MCP (stdio)
2. Cliente MCP -> Processo `tauri-mcp` (Build Customizado)
3. `tauri-mcp` -> Lança e controla o binário `elleven-library`
4. O `tauri_driver.py` detecta automaticamente o binário customizado.

## Como Rodar um Teste
### 1. Teste Básico (Smoke Test)
Execute o script de verificação para garantir que tudo funciona:
```bash
/usr/local/bin/python3 .agent/scripts/smoke_test_tauri.py
```
Isso deve abrir o app, tirar um screenshot em `smoke_test.png` e fechar.

### 2. Criando Novos Testes
Use o template abaixo em novos arquivos em `.agent/scripts/`:
```python
import asyncio
import os
from tauri_driver import run_tauri_test, APP_PATH

async def my_test_logic(tauri):
    # Lançar app
    pid = await tauri.launch_app(APP_PATH)
    print(f"App lançado: {pid}")
    
    # Exemplo: Tirar screenshot
    await tauri.take_screenshot("meu_teste.png")
    
    # Exemplo: Executar JS (se WebView acessível)
    # await tauri.execute_js("console.log('Hello from MCP')")

    # Finalizar
    await tauri.stop_app(pid)

if __name__ == "__main__":
    asyncio.run(run_tauri_test(my_test_logic))
```

## Solução de Problemas
- **Permissões:** No macOS, na primeira execução, o sistema pode pedir permissão de Acessibilidade para o terminal/Python/Cargo para controlar o mouse/screenshot.
- **Recompilação:** Se precisar atualizar o `tauri-mcp`, vá até `.agent/.deps/tauri-mcp` e rode `cargo build`. O driver usará o binário em `target/debug` ou `target/release`.
