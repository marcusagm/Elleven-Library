import struct
import os
import glob
import zipfile

def analyze_cdr(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)

    with open(filepath, 'rb') as f:
        magic = f.read(4)

        if magic == b'PK\x03\x04':
            print("Format: Modern ZIP-based CDR")
            try:
                with zipfile.ZipFile(filepath) as z:
                    file_list = z.namelist()
                    print(f"  Files in ZIP: {file_list[:10]}...")
                    previews = [name for name in file_list if 'preview' in name.lower() or 'thumb' in name.lower()]
                    print(f"  Preview Candidates: {previews}")
            except Exception as e:
                print(f"  Error reading zip: {e}")

        elif magic == b'RIFF':
            print("Format: Legacy RIFF-based CDR")
            f.seek(8)
            sig = f.read(4)
            print(f"  RIFF Signature: {sig}")

            # Walk chunks
            f.seek(12)
            while f.tell() < filesize - 8:
                try:
                    chunk_id = f.read(4)
                    if not chunk_id: break
                    chunk_size = struct.unpack('<I', f.read(4))[0]
                    curr_pos = f.tell()
                    print(f"  Chunk: {chunk_id.decode(errors='ignore')} Size: {chunk_size}")

                    if chunk_id == b'LIST':
                        list_type = f.read(4)
                        print(f"    LIST Type: {list_type.decode(errors='ignore')}")
                        f.seek(4, 1) # Skip type for next iteration if needed, or stay
                    elif chunk_id in [b'DISP', b'icp0', b'bmp ']:
                        print(f"    Potential Preview Chunk found: {chunk_id}")

                    # Jump to next chunk
                    f.seek(curr_pos + chunk_size, 0)
                    if chunk_size % 2 != 0: f.read(1) # Alignment
                except:
                    break
        else:
            print(f"Format: Unknown/Legacy Magic ({magic.hex()})")
            # Check for WLm. or other older headers
            f.seek(0)
            data = f.read(16)
            print(f"  Hex: {data.hex()}")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/CorelDRAW/"
    files = sorted(glob.glob(os.path.join(target_dir, "*.cdr")) + glob.glob(os.path.join(target_dir, "*.CDR")))
    for f in files:
        analyze_cdr(f)
