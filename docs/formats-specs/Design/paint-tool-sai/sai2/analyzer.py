import struct
import os

def read_u32(f):
    data = f.read(4)
    if not data: return None
    return struct.unpack('<I', data)[0]

def read_u64(f):
    data = f.read(8)
    if not data: return None
    return struct.unpack('<Q', data)[0]

def analyze_sai2(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)

    with open(filepath, 'rb') as f:
        # Header
        f.seek(32)
        width = read_u32(f)
        height = read_u32(f)
        # Scan for real chunk count
        f.seek(64)
        chunks = []
        while f.tell() < filesize:
            descriptor = f.read(16)
            if len(descriptor) < 16: break
            tag = descriptor[0:4]
            if not all(32 <= c <= 126 for c in tag):
                f.seek(-16, 1) # Backtrack
                break
            tag_str = tag.decode('ascii', errors='ignore')
            chunks.append({'tag': tag_str, 'size': struct.unpack('<Q', descriptor[8:16])[0], 'id': struct.unpack('<I', descriptor[4:8])[0]})

        chunk_count = len(chunks)
        data_region_start = 64 + (chunk_count * 16)
        print(f"Detected {chunk_count} chunks. Data region starts at 0x{data_region_start:X}")

        running_offset = data_region_start
        for i, chunk in enumerate(chunks):
            chunk['offset'] = running_offset
            if chunk['tag'] in ['thum', 'view']:
                print(f"  [{i:3}] Tag='{chunk['tag']}', Size={chunk['size']}, Offset=0x{chunk['offset']:X}")
                f.seek(chunk['offset'])
                data = f.read(min(32, chunk['size']))
                print(f"    Data (hex): {data.hex()}")
            running_offset += chunk['size']

if __name__ == "__main__":
    filepath = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Paint tool SAI/sai2/elfinha4.sai2"
    analyze_sai2(filepath)
