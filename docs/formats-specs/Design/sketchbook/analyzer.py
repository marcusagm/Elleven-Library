import struct
import os

def read_u16(f, byte_order='<'):
    return struct.unpack(byte_order + 'H', f.read(2))[0]

def read_u32(f, byte_order='<'):
    return struct.unpack(byte_order + 'I', f.read(4))[0]

def get_tag_data(f, tag_entry, order):
    tag, t_type, count, t_val = tag_entry
    if count == 0: return None

    # Types: 1=Byte, 2=ASCII, 3=Short, 4=Long, 5=Rational
    type_sizes = {1: 1, 2: 1, 3: 2, 4: 4, 5: 8}
    size = type_sizes.get(t_type, 1) * count

    if size <= 4:
        # Data is in the val field themselves
        f.seek(-4, 1) # Backtrack to raw values
        raw = f.read(4)
        return raw[:size]
    else:
        save = f.tell()
        f.seek(t_val)
        data = f.read(size)
        f.seek(save)
        return data

def analyze_sketchbook_tiff(filepath):
    print(f"\n===== Technical Analysis: {os.path.basename(filepath)} =====")
    with open(filepath, 'rb') as f:
        magic = f.read(2)
        if magic not in [b'II', b'MM']:
            print("Not a valid TIFF file.")
            return
        order = '<' if magic == b'II' else '>'
        version = read_u16(f, order)
        first_ifd_offset = read_u32(f, order)

        print(f"Endianness: {'Little' if order == '<' else 'Big'}")
        print(f"File Size: {os.path.getsize(filepath)} bytes")

        def process_ifd(offset, label="Primary"):
            f.seek(offset)
            entries = read_u16(f, order)
            print(f"\n--- {label} IFD at 0x{offset:X} ({entries} tags) ---")

            subifd_offsets = []
            for _ in range(entries):
                tag_raw = (read_u16(f, order), read_u16(f, order), read_u32(f, order), read_u32(f, order))
                tag, t_type, count, t_val = tag_raw

                if tag == 256: print(f"  Width: {t_val}")
                if tag == 257: print(f"  Height: {t_val}")
                if tag == 305: # Software
                    soft = get_tag_data(f, tag_raw, order).decode('ascii', errors='ignore').strip()
                    print(f"  Software: {soft}")
                if tag == 285: # PageName
                    name = get_tag_data(f, tag_raw, order).decode('ascii', errors='ignore').strip()
                    print(f"  Layer Name: {name}")
                if tag == 330: # SubIFDs
                    sub_data = get_tag_data(f, tag_raw, order)
                    for i in range(len(sub_data)//4):
                        subifd_offsets.append(struct.unpack(order + 'I', sub_data[i*4:i*4+4])[0])
                if tag == 50784: # Layer Props
                    props = get_tag_data(f, tag_raw, order).decode('ascii', errors='ignore').strip()
                    print(f"  SketchBook Metadata: {props}")

            for idx, soff in enumerate(subifd_offsets):
                process_ifd(soff, f"Layer {idx}")

        process_ifd(first_ifd_offset)

if __name__ == "__main__":
    target = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/SketchBook/"
    files = [f for f in os.listdir(target) if f.endswith(".tif")]
    if files:
        analyze_sketchbook_tiff(os.path.join(target, files[0]))
