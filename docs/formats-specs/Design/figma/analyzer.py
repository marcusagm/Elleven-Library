import os
import zipfile
import json
import struct

def find_signatures(data):
    """Find common compression and image signatures."""
    sigs = {
        'PNG': b'\x89PNG',
        'JPG': b'\xff\xd8\xff',
        'Zstd': b'\x28\xb5\x2f\xfd',
        'Zlib': b'\x78\x9c',
    }
    found = []
    for name, sig in sigs.items():
        idx = data.find(sig)
        while idx != -1:
            found.append((name, idx))
            idx = data.find(sig, idx + 1)
    return found

def analyze_figma_file(filepath):
    print(f"\n===== Technical Analysis: {os.path.basename(filepath)} =====")
    with open(filepath, 'rb') as f:
        magic = f.read(4)
        f.seek(0)
        file_data = f.read()

    # 1. Container Check
    if magic == b'PK\x03\x04':
        print("[Type] Standard exported .fig file (ZIP container)")
        with zipfile.ZipFile(filepath, 'r') as zf:
            print(f"[Contents] {len(zf.namelist())} entries")

            # Metadata
            if 'meta.json' in zf.namelist():
                meta = json.loads(zf.read('meta.json'))
                print(f"[Metadata] File: {meta.get('file_name')} | Exported at: {meta.get('exported_at')}")

            # Thumbnail
            if 'thumbnail.png' in zf.namelist():
                thumb_size = len(zf.read('thumbnail.png'))
                print(f"[Preview] thumbnail.png found ({thumb_size} bytes)")

            # Main Content (Kiwi)
            if 'canvas.fig' in zf.namelist():
                canvas_data = zf.read('canvas.fig')
                analyze_kiwi_blob(canvas_data, "Internal canvas.fig")

    elif magic == b'fig-':
        print("[Type] Standalone internal storage blob (fig-kiwi)")
        analyze_kiwi_blob(file_data, "Root File")
    else:
        print("[Type] Unknown or Malformed")

def analyze_kiwi_blob(data, label):
    sig = data[:8]
    if sig != b'fig-kiwi':
        print(f"[{label}] Not a valid fig-kiwi blob")
        return

    version = struct.unpack('<I', data[8:12])[0]
    length = struct.unpack('<I', data[12:16])[0]

    print(f"[{label}] Signature: fig-kiwi")
    print(f"[{label}] Schema Version: {version}")
    print(f"[{label}] Data Payload Length: {length} bytes")

    sigs = find_signatures(data)
    if sigs:
        # Filter duplicates or nearby sigs
        print(f"[{label}] Detected Signatures: {sigs[:5]}...")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - Figma/"
    files = ["Apple.fig", "Hotel booking app UI (Community).fig", "example.canvas.fig", "Upload file ui kit.fig"]
    for f in files:
        path = os.path.join(target_dir, f)
        if os.path.exists(path):
            analyze_figma_file(path)
