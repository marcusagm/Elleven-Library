import struct
import os
import glob
import re

def analyze_affinity_publisher(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)
    print(f"File Size: {filesize} bytes")

    with open(filepath, 'rb') as f:
        header = f.read(64)
        magic = header[:4]
        if magic != b'\x00\xff\x4b\x41':
            print("Invalid Magic Signature.")
            return

        version = struct.unpack('<I', header[4:8])[0]
        persona_info = header[8:16].decode('ascii', errors='ignore')

        # Pointers in header
        content_ptr = struct.unpack('<Q', header[16:24])[0]
        thumb_ptr = struct.unpack('<Q', header[24:32])[0]

        print(f"Magic: {magic.hex().upper()}")
        print(f"Version/Flags: {version}")
        print(f"Persona Info: {persona_info}")
        print(f"Content Pointer: {content_ptr} (0x{content_ptr:X})")
        print(f"Thumbnail Pointer: {thumb_ptr} (0x{thumb_ptr:X})")

        # Inspect Thumbnail Block
        if 0 < thumb_ptr < filesize:
            f.seek(thumb_ptr)
            block_header = f.read(32)
            if block_header.startswith(b'\xff\xff\xff\xff'):
                sig = block_header[4:8].decode('ascii', errors='ignore')
                b_ver = struct.unpack('<I', block_header[8:12])[0]
                b_size = struct.unpack('<I', block_header[12:16])[0]
                print(f"Found Block at 0x{thumb_ptr:X}:")
                print(f"  Signature: {sig}")
                print(f"  Block Version: {b_ver}")
                print(f"  Block Size: {b_size}")

                # Search for PNG signature in the block
                f.seek(thumb_ptr)
                # Read a chunk to find PNG
                data = f.read(min(b_size + 100, 1024*1024))
                png_sig = b'\x89PNG\r\n\x1a\n'
                png_pos = data.find(png_sig)
                if png_pos != -1:
                    print(f"  PNG Thumbnail found at relative offset {png_pos}")
                    # Extract a bit of info or save it
                    # (In this case just confirming it's there)
                else:
                    print("  PNG Thumbnail NOT found in block.")
            else:
                print(f"Block at 0x{thumb_ptr:X} does not have FFFFFFFF prefix.")

        # Search for other structures
        # Looking for strings in the first 2KB
        f.seek(0)
        head_data = f.read(2048)
        found_strings = re.findall(b'[a-zA-Z0-9#_]{4,}', head_data)
        print(f"Header Strings: {found_strings[:15]}")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - Affinity/Affinity Publisher/"
    files = glob.glob(os.path.join(target_dir, "*.afpub"))
    for f in files:
        analyze_affinity_publisher(f)
