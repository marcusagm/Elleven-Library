import struct
import os
import zlib
import xml.etree.ElementTree as ET

def read_uint32(f):
    return struct.unpack('<I', f.read(4))[0]

def analyze_mdp(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)

    with open(filepath, 'rb') as f:
        magic = f.read(8)
        if magic != b'mdipack\x00': return

        f.seek(12)
        xml_size = read_uint32(f)
        bin_size = read_uint32(f)

        # XML
        f.seek(20)
        xml_data = f.read(xml_size)

        WIDTH=0
        HEIGHT=0
        try:
            root = ET.fromstring(xml_data)
            # Find canvas info
            WIDTH = int(root.attrib.get('width', 0))
            HEIGHT = int(root.attrib.get('height', 0))
            print(f"Canvas: {WIDTH}x{HEIGHT}")

            # Look for thumbnail info
            thumb_node = root.find("Thumb")
            if thumb_node is not None:
                t_w = int(thumb_node.attrib.get('width', 0))
                t_h = int(thumb_node.attrib.get('height', 0))
                print(f"Thumb Info: {t_w}x{t_h}")

        except Exception as e:
            print(f"XML Error: {e}")

        # BINARY BLOCKS
        f.seek(20 + xml_size)
        curr = f.tell()
        end = curr + bin_size

        idx = 0
        while f.tell() < end:
            block_start = f.tell()

            # Read first 4 bytes to check for PAC or Size
            indicator = f.read(4)
            f.seek(block_start)

            is_generic = True
            header_size = 0
            payload_start = 0
            block_size = 0

            if indicator == b'PAC ':
                # Special PAC Header?
                # Structure seems to be:
                # Magic(4) Size(4) Type?(4) Unk(4) => 16 bytes
                # Then padding/name area until 132?
                # Let's assume header is 132 bytes based on Zlib find
                print(f"Block {idx}: PAC Header")
                f.seek(block_start + 4)
                block_size = read_uint32(f)
                header_size = 132 # based on grep

                # Check name at +68?
                f.seek(block_start + 68)
                name_bytes = f.read(64).split(b'\x00')[0]
                print(f"  Name: {name_bytes.decode('ascii', errors='ignore')}")

            else:
                # Standard Block
                # Size(4) Unk(4) Unk(4) => 12 bytes
                # Name at +52? Or +48 relative to start?
                # Zlib at +140?
                block_size = read_uint32(f)
                header_size = 140 # based on grep

                f.seek(block_start + 48) # Name offset?
                # Actually earlier analysis suggested name at +52 relative to START?
                # Or +48?
                # Let's read 64 bytes at +48
                f.seek(block_start + 48) # 0x30
                name_bytes = f.read(64).split(b'\x00')[0]
                print(f"Block {idx}: Standard, Size={block_size}")
                print(f"  Name: {name_bytes.decode('ascii', errors='ignore')}")

            # Verify Zlib
            zlib_offset = block_start + header_size
            f.seek(zlib_offset)

            # Check range
            if zlib_offset >= block_start + block_size:
                print("  Warning: Header > Block Size")
            else:
                # Read payload
                payload_len = block_size - header_size
                payload = f.read(payload_len)

                if len(payload) > 2 and payload[0] == 0x78:
                    try:
                        decomp = zlib.decompress(payload)
                        print(f"  -> ZLIB OK. Size: {len(decomp)}")

                        # Try to save thumb
                        if idx == 0 and "thumb" in name_bytes.decode('ascii', 'ignore'):
                             filename = f"{filepath}_thumb.raw"
                             with open(filename, 'wb') as tf:
                                 tf.write(decomp)
                             print(f"  -> Saved thumb raw to {filename}")
                             # Check dimensions
                             # Thumb sizes are usually small
                             # e.g. 256x221 (from XML)
                             # 256*221*4 = 226304 bytes.
                             # If matches, we are good.

                    except Exception as e:
                        print(f"  -> ZLIB Error: {e}")
                else:
                    print(f"  -> Not ZLIB (First byte: {payload[0] if payload else '?'})")

            f.seek(block_start + block_size)
            idx += 1

if __name__ == "__main__":
    import glob
    files = glob.glob("*.mdp")
    for file in files:
        analyze_mdp(file)
