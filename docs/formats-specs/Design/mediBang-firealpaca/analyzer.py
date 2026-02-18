import struct
import os
import zlib

def parse_mdp(filename):
    basename = os.path.basename(filename)
    print(f"\n--- MDP Analyzer: {basename} ---")
    filesize = os.path.getsize(filename)

    with open(filename, 'rb') as f:
        # 1. Header
        magic = f.read(8)
        if magic != b'mdipack\x00':
            print("Invalid Magic Signature")
            return

        version = struct.unpack('<I', f.read(4))[0]
        bin_size = struct.unpack('<I', f.read(4))[0]
        xml_size = struct.unpack('<I', f.read(4))[0]

        print(f"File Header: Version={version}, BinSize={bin_size}, XMLSize={xml_size}")

        f.seek(20)
        xml_region = f.read(xml_size)

        # 2. Extract XML Text
        xml_end = xml_region.find(b'</Mdiapp>')
        if xml_end != -1:
            xml_text = xml_region[:xml_end+9]
            # print(f"XML Metadata: {len(xml_text)} bytes")

        # 3. Find first PAC
        pac_marker = b'PAC '
        pac_idx = xml_region.find(pac_marker)
        if pac_idx == -1:
            # Fallback scan the whole file
            f.seek(20)
            data = f.read()
            pac_idx_abs = data.find(pac_marker)
            if pac_idx_abs == -1:
                print("No PAC blocks found.")
                return
            current_pos = 20 + pac_idx_abs
        else:
            current_pos = 20 + pac_idx

        print(f"First PAC found at offset 0x{current_pos:X}")

        # 4. Traverse PAC blocks
        blocks = []
        while current_pos < filesize:
            f.seek(current_pos)
            header = f.read(132)
            if len(header) < 132 or header[:4] != b'PAC ':
                break

            total_size = struct.unpack('<I', header[4:8])[0]
            compressed_size = struct.unpack('<I', header[12:16])[0]

            # Find name in the 116-byte metadata area
            metadata = header[16:]
            # The name seems to be at a variable offset or just after some nulls/u32s.
            # Let's search for 'thumb' or 'layer' strings
            name = "unknown"
            for tag in [b"thumb", b"layer", b"icc", b"preview"]:
                if tag in metadata:
                    idx = metadata.find(tag)
                    name_bytes = metadata[idx:].split(b'\x00')[0]
                    name = name_bytes.decode('ascii', errors='ignore')
                    break

            print(f"PAC Block: '{name:12}' | DataSize={compressed_size:8} | TotalSize={total_size:8} | Offset=0x{current_pos:X}")

            if name == 'thumb':
                f.seek(current_pos + 132)
                zdata = f.read(compressed_size)
                try:
                    raw = zlib.decompress(zdata)
                    print(f"    -> Thumbnail Size: {len(raw)} bytes")
                    # Save thumbnail as raw for verification if needed
                    # with open(f"{basename}_thumb.raw", "wb") as tf: tf.write(raw)
                except Exception as e:
                    print(f"    -> Failed to decompress: {e}")

            if total_size == 0: break
            current_pos += total_size

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - MediBang Paint - Firealpaca/"
    for f in ["8bit_test.mdp", "aula_silhueta.mdp", "checkerboard5.mdp"]:
        parse_mdp(os.path.join(target_dir, f))
