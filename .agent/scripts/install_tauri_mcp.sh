#!/bin/bash
echo "Instalando tauri-mcp via git para garantir compatibilidade com protocolo 2024-11-05..."
cargo install --git https://github.com/dirvine/tauri-mcp

echo "Instalando dependências Python..."
/usr/local/bin/python3 -m pip install mcp
