import asyncio
import os
import sys

# Ajuste de path para import
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from tauri_driver import run_tauri_test, TauriTestSession

APP_PATH = os.path.abspath(os.path.join(
    os.path.dirname(__file__), 
    "../../src-tauri/target/debug/elleven-library"
))

if not os.path.exists(APP_PATH):
    print(f"⚠️ Binário não encontrado no caminho padrão: {APP_PATH}")
    # Tenta release ou outro path comum?
    # Por hora, deixa falhar ou avisa.
    pass

async def my_smoke_test(tauri: TauriTestSession):
    print("--- Início do Lógica de Teste ---")
    
    # Lança
    await tauri.launch_app(APP_PATH)
    
    # Espera carregar
    await asyncio.sleep(5)
    
    # Screenshot
    screen_path = os.path.abspath(os.path.join(
        os.path.dirname(__file__), 
        "../../smoke_test.png"
    ))
    await tauri.take_screenshot(screen_path)
    
    # Fecha
    await tauri.stop_app()
    print("--- Fim do Lógica de Teste ---")

if __name__ == "__main__":
    try:
        asyncio.run(run_tauri_test(my_smoke_test))
    except KeyboardInterrupt:
        pass
    except Exception as e:
        print(f"FAILED: {e}")
