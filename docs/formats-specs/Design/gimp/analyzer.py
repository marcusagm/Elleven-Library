import struct
import os
import sys

def read_be_uint32(f):
    data = f.read(4)
    if len(data) < 4: return None
    return struct.unpack('>I', data)[0]

def read_be_int32(f):
    data = f.read(4)
    if len(data) < 4: return None
    return struct.unpack('>i', data)[0]

def get_prop_name(prop_type):
    props = {
        0: "PROP_END",
        1: "PROP_COLORMAP",
        2: "PROP_ACTIVE_LAYER",
        3: "PROP_ACTIVE_CHANNEL",
        4: "PROP_SELECTION",
        5: "PROP_FLOATING_SELECTION",
        6: "PROP_OPACITY",
        7: "PROP_MODE",
        8: "PROP_VISIBLE",
        9: "PROP_LINKED",
        10: "PROP_LOCK_ALPHA",
        11: "PROP_APPLY_MASK",
        12: "PROP_EDIT_MASK",
        13: "PROP_SHOW_MASK",
        14: "PROP_SHOW_MASKED",
        15: "PROP_OFFSETS",
        16: "PROP_COLOR",
        17: "PROP_COMPRESSION",
        18: "PROP_GUIDES",
        19: "PROP_RESOLUTION",
        20: "PROP_TATTOO",
        21: "PROP_PARASITES",
        22: "PROP_UNIT",
        23: "PROP_PATHS",
        24: "PROP_USER_UNIT",
        25: "PROP_VECTORS",
        26: "PROP_TEXT_LAYER_FLAGS",
        27: "PROP_SAMPLE_POINTS",
        28: "PROP_LOCK_CONTENT",
        29: "PROP_GROUP_ITEM_FLAGS",
        30: "PROP_BUTT_END" # ...
    }
    return props.get(prop_type, f"Unknown ({prop_type})")

def read_ptr(f, version):
    if version >= 11:
        data = f.read(8)
        if len(data) < 8: return None
        return struct.unpack('>Q', data)[0]
    else:
        data = f.read(4)
        if len(data) < 4: return None
        return struct.unpack('>I', data)[0]

def parse_xcf(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    try:
        with open(filepath, 'rb') as f:
            # 1. Header
            magic = f.read(9)
            if magic != b'gimp xcf ':
                print("Invalid Magic.")
                return

            version_bytes = f.read(4) # v001, v011, etc.
            version_str = version_bytes.decode('utf-8')
            print(f"Version: {version_str}")
            try:
                version_int = int(version_str[1:])
            except:
                version_int = 0

            f.read(1) # Null terminator

            width = read_be_uint32(f)
            height = read_be_uint32(f)
            base_type = read_be_uint32(f)

            base_types = {0: "RGB", 1: "Grayscale", 2: "Indexed"}
            print(f"Dimensions: {width}x{height}")
            print(f"Base Type: {base_types.get(base_type, base_type)}")

            # 2. Properties
            print("\n-- Image Properties --")
            while True:
                prop_type = read_be_uint32(f)
                prop_len = read_be_uint32(f)

                if prop_type is None: break

                name = get_prop_name(prop_type)
                print(f"  Prop: {name}, Len: {prop_len}")

                if prop_type == 0: break # PROP_END

                val_start = f.tell()

                if prop_type == 17: # PROP_COMPRESSION
                    comp_type = f.read(1)
                    if comp_type:
                        print(f"    Compression Value: {comp_type[0]} (0=None, 1=RLE, 2=ZLIB, 3=Fractal)")

                f.seek(val_start + prop_len)

            # 3. Layer/Channel Pointers
            # Use read_ptr sensitive to version
            layer_ptr = read_ptr(f, version_int)
            channel_ptr = read_ptr(f, version_int)
            print(f"\nLayer Index Pointer: {layer_ptr}")
            print(f"Channel Index Pointer: {channel_ptr}")

            # 4. Parse Layer List
            if layer_ptr and layer_ptr > 0:
                print(f"Layer Offset List starts at: {layer_ptr}")
                f.seek(layer_ptr)
                layer_offsets = []
                while True:
                    # Offsets in the list are also ptr-sized
                    off = read_ptr(f, version_int)
                    if off == 0 or off is None: break
                    layer_offsets.append(off)

                print(f"Found {len(layer_offsets)} layers: {layer_offsets}")

                for idx, layer_offset in enumerate(layer_offsets):
                    print(f"\n  -- Examining Layer {idx} at offset {layer_offset} --")
                    f.seek(layer_offset)

                    l_width = read_be_uint32(f)
                    l_height = read_be_uint32(f)
                    l_type = read_be_uint32(f)
                    l_name_len = read_be_uint32(f)

                    print(f"    Values: W={l_width}, H={l_height}, Type={l_type}, NameLen={l_name_len}")

                    # Name
                    if 0 < l_name_len < 1024:
                        l_name = f.read(l_name_len - 1).decode('utf-8', errors='ignore')
                        f.read(1) # Null
                        print(f"    Name: '{l_name}'")
                    else:
                        print("    [!] Invalid name length.")

                    # Layer Properties
                    print("    Layer Properties:")
                    while True:
                        p_type = read_be_uint32(f)
                        p_len = read_be_uint32(f)
                        if p_type is None: break
                        if p_type == 0: break

                        p_name = get_prop_name(p_type)
                        print(f"      {p_name} ({p_type}), Len: {p_len}")

                        val_pos = f.tell()
                        if p_type == 15: # PROP_OFFSETS
                            # v11 also changes how this property stores data?
                            # Usually offsets to hierarchy, mask...
                            # These are pointers, so 8 bytes in v11?
                            h_off = read_ptr(f, version_int)
                            m_off = read_ptr(f, version_int)
                            print(f"        -> Hierarchy Offset: {h_off}")
                            print(f"        -> Mask Offset: {m_off}")

                        f.seek(val_pos + p_len)

    except Exception as e:
        print(f"Error parsing {filepath}: {e}")

if __name__ == "__main__":
    import glob
    files = glob.glob("*.xcf")
    # Prioritize interesting files
    priority = ["gimp-splash.xcf", "default_icon.xcf"]
    for p in priority:
        if p in files:
            parse_xcf(p)
            files.remove(p)

    # Process others (limit count)
    for f in files[:3]:
        parse_xcf(f)
