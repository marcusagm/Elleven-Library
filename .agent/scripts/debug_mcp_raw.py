import asyncio
import os
import sys
import json
import shutil
from asyncio import subprocess

async def raw_mcp_handshake():
    server_bin = "tauri-mcp"
    if not shutil.which(server_bin):
        # Tenta path manual
        home = os.path.expanduser("~")
        cargo_bin = os.path.join(home, ".cargo", "bin", "tauri-mcp")
        if os.path.exists(cargo_bin):
             server_bin = cargo_bin
    
    print(f"Debug Handshake com: {server_bin}")
    
    # Inicia processo
    proc = await asyncio.create_subprocess_exec(
        server_bin, "serve",
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    
    # Envia handshake manual
    handshake = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05", 
            "capabilities": {},
            "clientInfo": {"name": "raw-debug", "version": "1.0"}
        }
    }
    
    print("-> Enviando Handshake (2024-11-05)...")
    proc.stdin.write(json.dumps(handshake).encode() + b"\n")
    await proc.stdin.drain()
    
    # Lê resposta
    try:
        line = await asyncio.wait_for(proc.stdout.readline(), timeout=5.0)
        print(f"<- Resposta: {line.decode().strip()}")
        
        # Se falhou, vamos tentar descobrir qual versão ele quer (geralmente nao diz, só erro)
        # Mas as vezes o erro contem dicas.
    except asyncio.TimeoutError:
        print("<- Timeout esperando resposta.")
        # Lê stderr para ver logs
        err = await proc.stderr.read(1024)
        print(f"Stderr: {err.decode()}")

    proc.terminate()
    try:
        await proc.wait()
    except: 
        pass

if __name__ == "__main__":
    asyncio.run(raw_mcp_handshake())
