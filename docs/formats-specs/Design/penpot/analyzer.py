import struct
import os
import zipfile
import json

def analyze_penpot(filepath):
    print(f"\n--- Technical Analysis: {os.path.basename(filepath)} ---")
    with open(filepath, 'rb') as f:
        magic = f.read(4)
        f.seek(0)

        if magic == b'PK\x03\x04':
            print("Format Type: V1 (ZIP Container)")
            try:
                with zipfile.ZipFile(filepath, 'r') as zf:
                    namelist = zf.namelist()
                    print(f"Total files in ZIP: {len(namelist)}")

                    # Find root project file
                    root_files = [n for n in namelist if n.startswith('files/') and n.endswith('.json') and n.count('/') == 1]
                    if root_files:
                        print(f"Root project file: {root_files[0]}")
                        with zf.open(root_files[0]) as rf:
                            meta = json.load(rf)
                            print(f"  Project Name: {meta.get('name')}")
                            print(f"  Scheme Version: {meta.get('version')}")

                    # Search for previews
                    previews = [n for n in namelist if 'thumbnails' in n and n.endswith('.png')]
                    objects = [n for n in namelist if n.startswith('objects/') and n.endswith('.png')]
                    print(f"Thumbnails/Previews found: {len(previews) + len(objects)} assets.")

            except zipfile.BadZipFile:
                print("Error: Invalid ZIP archive.")

        elif magic == b'\x01\x0b\x1a\x86':
            print("Format Type: V2 (Modern Zstd Binary)")
            # Analysis of header
            header = f.read(17)
            print(f"Header: {header.hex(' ')}")

            # Check for Zstd Magic
            f.seek(17)
            zstd_magic = f.read(4)
            if zstd_magic == b'(\xb5/\xfd':
                print("Zstandard payload detected at offset 17.")
            else:
                print(f"Unknown payload signature: {zstd_magic.hex(' ')}")
        else:
            print(f"Unknown format signature: {magic.hex(' ')}")

if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:
        target = sys.argv[1]
    else:
        target = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Penpot/"

    if os.path.isdir(target):
        files = [os.path.join(target, f) for f in os.listdir(target) if f.endswith('.penpot')]
        # Select one of each type if possible
        for f in files[:5]:
            analyze_penpot(f)
    else:
        analyze_penpot(target)
