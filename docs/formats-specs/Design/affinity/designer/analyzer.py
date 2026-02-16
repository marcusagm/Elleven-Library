import struct
import os
import zlib
import re

def analyze_affinity(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)
    print(f"File Size: {filesize} (0x{filesize:X})")

    with open(filepath, 'rb') as f:
        # Header (64 bytes)
        header = f.read(64)
        print(f"Header Hex: {header.hex()}")

        # Magic
        magic = header[:4]
        if magic.hex() == '00ff4b41':
            print("Magic: 00FF4B41 (Affinity)")
        else:
            print(f"Magic: {magic.hex()} (Unknown)")

        # Pointers at 0x18
        # Header structure hypothesis:
        # 0x00: Magic (4)
        # 0x04: Version (4)
        # 0x08: Unknown string/id (8) "nsrP#Inf" ?
        # 0x10: Ptr to Content? (8)
        # 0x18: Ptr to Thumbnail Block? (8)

        f.seek(0x18)
        thumb_ptr_bytes = f.read(8)
        thumb_ptr = struct.unpack('<Q', thumb_ptr_bytes)[0]

        print(f"Thumbnail Pointer at 0x18: {thumb_ptr} (0x{thumb_ptr:X})")

        if 0 < thumb_ptr < filesize:
            f.seek(thumb_ptr)
            # Read Block Header
            # Expected: FF FF FF FF [Type] [Ver] [Size]
            block_header = f.read(32)
            print(f"Block at 0x{thumb_ptr:X}: {block_header.hex()}")

            # Check for Thmb
            if b'Thmb' in block_header:
                print("Found 'Thmb' block signature!")
                # Parse
                # 0x00: FFFFFFFF
                # 0x04: "Thmb"
                # 0x08: Version (u32)
                # 0x0C: Size (u32) ?

                b_ver = struct.unpack('<I', block_header[8:12])[0]
                b_size = struct.unpack('<I', block_header[12:16])[0]
                print(f"  Thmb Ver: {b_ver}")
                print(f"  Thmb Block Size?: {b_size}")

                # Check PNG inside
                # Usually after some header.
                # Let's verify if `b_size` matches PNG size approximately

                # Search PNG sig near here
                f.seek(thumb_ptr)
                block_data = f.read(b_size + 100) # Read a bit more
                png_idx = block_data.find(b'\x89PNG')
                if png_idx != -1:
                    print(f"  PNG starts at relative offset {png_idx}")
                    png_data = block_data[png_idx:]
                    print(f"  Extracting thumbnail to {filepath}_thumb.png")
                    with open(f"{filepath}_thumb.png", "wb") as out:
                        out.write(png_data)

        # Scan for Strings
        f.seek(0)
        content = f.read(2048) # Header area
        strings = re.findall(b'[A-Za-z0-9#]{4,}', content)
        print(f"Header Strings: {strings[:10]}")

if __name__ == "__main__":
    import glob
    files = glob.glob("*.afdesign")
    for file in files:
        analyze_affinity(file)
