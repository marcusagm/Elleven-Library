import struct
import os
import re

def analyze_rif_full(filepath):
    print(f"--- Technical Analysis: {os.path.basename(filepath)} ---")
    with open(filepath, 'rb') as f:
        data = f.read()

    # 1. Header (8 bytes)
    ver, flags, width, height = struct.unpack('>HHHH', data[:8])
    print(f"Header: Version={ver}, Flags={flags:04x}, Canvas={width}x{height}")

    # 2. Find JPEG Thumbnails
    jpeg_pos = data.find(b'\xff\xd8\xff\xe0')
    if jpeg_pos != -1:
        jpeg_end = data.find(b'\xff\xd9', jpeg_pos)
        if jpeg_end != -1:
            print(f"Thumbnail Found: Type=JPEG, Offset=0x{jpeg_pos:X}, Size={jpeg_end+2-jpeg_pos} bytes")
    else:
        print("No JPEG thumbnail found (standard JFIF).")

    # 3. Block Analysis (Scan for [Size][Tag])
    # Corel blocks are [u32 Size][4-byte Tag] where Size = 4 (tag) + payload
    # Or sometimes [u32 Size][4-byte Tag][u32 PayloadSize] where Size = 8 + payload

    blocks = []
    # Search for known tags like PCOL, FSKT, ANNO, NOTE, ICCP, BUMB
    known_tags = [b'PCOL', b'FSKT', b'ANNO', b'NOTE', b'ICCP', b'BUMB', b'VIEW', b'LAYR', b'FSPG']
    for tag in known_tags:
        it = 0
        while True:
            idx = data.find(tag, it)
            if idx == -1: break
            # Check for size prefix (4 bytes before tag)
            if idx >= 4:
                size = struct.unpack('>I', data[idx-4:idx])[0]
                if 4 <= size < 10000000:
                    blocks.append({'offset': idx-4, 'tag': tag.decode(), 'size': size})
            it = idx + 4

    # Sort blocks by offset
    blocks.sort(key=lambda x: x['offset'])
    for b in blocks:
        print(f"Block: 0x{b['offset']:08X} Tag={b['tag']} Size={b['size']}")

if __name__ == "__main__":
    analyze_rif_full("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Corel Painter/Line Sketches1.rif")
    analyze_rif_full("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Corel Painter/splat.rif")
