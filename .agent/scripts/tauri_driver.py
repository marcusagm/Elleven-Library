import asyncio
import os
import shutil
import sys
from typing import Optional, List

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

class TauriTestSession:
    """Wrapper para facilitar o uso do MCP Client em testes."""
    def __init__(self, session: ClientSession):
        self.session = session
        self.process_id = None

    async def launch_app(self, app_path: str, args: List[str] = None):
        if args is None: args = []
        print(f"🚀 Lançando app: {app_path}")
        result = await self.session.call_tool("launch_app", arguments={
            "app_path": app_path,
            "args": args
        })
        
        # O resultado vem como CallToolResult.
        # Precisamos inspecionar o conteúdo.
        # O tauri-mcp retorna um texto ou json no content.
        # Vamos logar para debug se falhar o parse.
        print(f"📦 Resultado Launch: {result}")
        
        # Tentativa de extração simplificada (baseada em payloads comuns)
        # Se o conteúdo for JSON text:
        for content in result.content:
            if hasattr(content, "text"):
                # Tentar achar o PID no texto ou assumir sucesso
                import json
                try:
                    data = json.loads(content.text)
                    if "process_id" in data:
                        self.process_id = data["process_id"]
                except:
                    pass
        
        # Fallback: Se não achou PID, apenas assume que rodou, 
        # mas métodos subsequentes podem falhar se precisarem de PID.
        return self.process_id

    async def take_screenshot(self, output_path: str):
        print(f"📸 Tirando screenshot...")
        # Se tiver PID, usa. Se não, o mcp pode tentar achar o main? Não, precisa de PID.
        # Vamos listar tools para ver se existe algo sem PID? Não.
        if not self.process_id:
            print("⚠️ Aviso: Sem PID, tentando screenshot sem argumento (pode falhar)")
            args = {"output_path": output_path}
        else:
             args = {"process_id": self.process_id, "output_path": output_path}

        await self.session.call_tool("take_screenshot", arguments=args)
        print(f"✅ Salvo em {output_path}")

    async def stop_app(self):
        if self.process_id:
            print(f"🛑 Parando app PID {self.process_id}")
            await self.session.call_tool("stop_app", arguments={"process_id": self.process_id})

async def run_tauri_test(test_callback):
    """
    Função helper para rodar um teste.
    :param test_callback: Função async que recebe (session: TauriTestSession)
    """
    # Prioridade para a versão corrigida localmente pelo agente
    local_release = os.path.abspath(os.path.join(
        os.path.dirname(__file__), 
        "../.deps/tauri-mcp/target/release/tauri-mcp"
    ))
    local_debug = os.path.abspath(os.path.join(
        os.path.dirname(__file__), 
        "../.deps/tauri-mcp/target/debug/tauri-mcp"
    ))
    
    if os.path.exists(local_release):
        server_bin = local_release
        print(f"🔧 Usando tauri-mcp corrigido (release): {server_bin}")
    elif os.path.exists(local_debug):
        server_bin = local_debug
        print(f"🔧 Usando tauri-mcp corrigido (debug): {server_bin}")
    elif not shutil.which(server_bin):
        home = os.path.expanduser("~")
        cargo_bin = os.path.join(home, ".cargo", "bin", "tauri-mcp")
        if os.path.exists(cargo_bin):
            server_bin = cargo_bin
        else:
            raise FileNotFoundError("tauri-mcp não encontrado")

    env = os.environ.copy()
    env["RUST_LOG"] = "error" # Silenciar logs informativos que podem quebrar o protocolo

    server_params = StdioServerParameters(
        command=server_bin,
        args=["serve"],
        env=env
    )

    print(f"🔌 Conectando ao servidor: {server_bin}")
    try:
        async with stdio_client(server_params) as (read, write):
            async with ClientSession(read, write) as session:
                await session.initialize()
                
                # Listar tools para debug
                tools = await session.list_tools()
                print(f"🛠️ Ferramentas disponíveis: {[t.name for t in tools.tools]}")
                
                tt = TauriTestSession(session)
                await test_callback(tt)
    except Exception as e:
        print(f"❌ Erro CRÍTICO na sessão MCP: {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()
        raise
