import struct
import os
import glob

def read_be_u16(f):
    return struct.unpack('>H', f.read(2))[0]

def read_be_u32(f):
    return struct.unpack('>I', f.read(4))[0]

def analyze_psd(filepath):
    print(f"\n--- Deep Analysis: {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)

    with open(filepath, 'rb') as f:
        # Header
        magic = f.read(4) # 0x00
        version = read_be_u16(f) # 0x04
        f.seek(6, 1) # Reserved (0x06 - 0x0B)
        channels = read_be_u16(f) # 0x0C
        height = read_be_u32(f) # 0x0E
        width = read_be_u32(f) # 0x12
        depth = read_be_u16(f) # 0x16
        mode = read_be_u16(f) # 0x18

        print(f"Header: Magic={magic.decode()}, Ver={version}, Size={width}x{height}, Channels={channels}, Depth={depth}, Mode={mode}")

        # Color Mode Data
        cmd_len = read_be_u32(f)
        print(f"Color Mode Data Length: {cmd_len}")
        f.seek(cmd_len, 1)

        # Image Resources
        ir_len = read_be_u32(f)
        print(f"Image Resources Length: {ir_len}")
        ir_end = f.tell() + ir_len

        while f.tell() < ir_end:
            res_sig = f.read(4)
            if res_sig != b'8BIM' and res_sig != b'MeSa':
                # Pad to 2 bytes?
                if not res_sig: break
                continue

            res_id = read_be_u16(f)
            # Pascal String (padded to 2)
            name_len = f.read(1)[0]
            name = f.read(name_len)
            if (name_len + 1) % 2 != 0:
                f.read(1)

            res_data_size = read_be_u32(f)
            res_pos = f.tell()

            if res_id == 1033 or res_id == 1036:
                fmt = read_be_u32(f)
                tw = read_be_u32(f)
                th = read_be_u32(f)
                print(f"  [Resource {res_id}] Thumbnail: Format={fmt}, Dim={tw}x{th}, Size={res_data_size}")
            elif res_id == 1061: # Caption digest
                print(f"  [Resource {res_id}] Caption Digest found")
            elif res_id == 1005: # Resolution info
                print(f"  [Resource {res_id}] Resolution Info found")

            f.seek(res_pos + res_data_size, 0)
            if res_data_size % 2 != 0:
                f.read(1) # Padded to even

        # Layer and Mask Info
        lm_len = read_be_u32(f)
        print(f"Layer and Mask Info Length: {lm_len}")
        f.seek(lm_len, 1)

        # Image Data
        if f.tell() < filesize:
            compression = read_be_u16(f)
            comp_types = {0: "Raw", 1: "RLE (PackBits)", 2: "Zip without prediction", 3: "Zip with prediction"}
            print(f"Image Data Compression: {comp_types.get(compression, 'Unknown')}")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Adobe/Photoshop/"
    for psd in sorted(glob.glob(os.path.join(target_dir, "*.psd"))):
        analyze_psd(psd)
