import struct
import os

def read_be_uint16(f):
    data = f.read(2)
    if len(data) < 2: return None
    return struct.unpack('>H', data)[0]

def read_be_uint32(f):
    data = f.read(4)
    if len(data) < 4: return None
    return struct.unpack('>I', data)[0]

def analyze_psd(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)
    print(f"File Size: {filesize}")

    with open(filepath, 'rb') as f:
        # Header (26 bytes)
        magic = f.read(4)
        if magic != b'8BPS':
            print("Invalid Magic.")
            return

        version = read_be_uint16(f)
        f.read(6) # Reserved
        channels = read_be_uint16(f)
        height = read_be_uint32(f)
        width = read_be_uint32(f)
        depth = read_be_uint16(f)
        color_mode = read_be_uint16(f)

        print(f"PSD Version: {version}")
        print(f"Canvas: {width}x{height}, Channels: {channels}, Depth: {depth}, Mode: {color_mode}")

        # Color Mode Data
        cmd_size = read_be_uint32(f)
        print(f"Color Mode Data Size: {cmd_size}")
        f.seek(cmd_size, 1)

        # Image Resources
        ir_size = read_be_uint32(f)
        print(f"Image Resources Size: {ir_size}")
        ir_end = f.tell() + ir_size

        while f.tell() < ir_end:
            sig = f.read(4)
            if sig != b'8BIM':
                # Sometimes 8BIM is followed by something else or alignment?
                # Actually sig must be 8BIM
                if not sig: break
                # Skip 1 byte for alignment if it's not 8BIM?
                # PSR format says alignment is to 2 bytes.
                pass

            res_id = read_be_uint16(f)

            # Name (Pascal string, padded to even)
            name_len = f.read(1)[0]
            name = f.read(name_len)
            if (name_len + 1) % 2 != 0:
                f.read(1)

            res_data_size = read_be_uint32(f)
            res_data_start = f.tell()

            # Thumbnails are usually 1033 (old) or 1036 (new)
            if res_id in [1033, 1036]:
                print(f"Found Thumbnail Resource ID: {res_id}, Size: {res_data_size}")
                # Analyze thumbnail header
                # 4 bytes: Format (1 = kJpegRGB, 0 = kRawRGB)
                # 4 bytes: Width
                # 4 bytes: Height
                # 4 bytes: WidthBytes
                # 4 bytes: TotalSize
                # 4 bytes: SizeAfterCompression
                # 2 bytes: BitsPerPixel
                # 2 bytes: NumberOfPlanes

                fmt = read_be_uint32(f)
                tw = read_be_uint32(f)
                th = read_be_uint32(f)
                print(f"  Thumb Meta: Format={fmt}, Dim={tw}x{th}")

                if fmt == 1:
                    # JPEG thumbnail
                    # Data follows.
                    # We need to skip the remaining header (total 28 bytes for thumbnail info)
                    # We already read 12 (fmt, tw, th)
                    f.seek(16, 1) # Skip rest of thumb header

                    jpeg_data = f.read(res_data_size - 28)
                    thumb_filename = f"{filepath}.thumb.jpg"
                    with open(thumb_filename, 'wb') as tf:
                        tf.write(jpeg_data)
                    print(f"  Saved JPEG thumbnail to {thumb_filename}")

            f.seek(res_data_start + res_data_size, 0)
            # Alignment to 2 bytes
            if res_data_size % 2 != 0:
                f.read(1)

        # Layer and Mask Information
        f.seek(ir_end, 0)
        lm_size = read_be_uint32(f)
        print(f"Layer and Mask Info Size: {lm_size}")

        # Image Data (Compression)
        f.seek(lm_size, 1)
        comp = read_be_uint16(f)
        comp_names = {0: "Raw", 1: "RLE", 2: "Zip without prediction", 3: "Zip with prediction"}
        print(f"Image Data Compression: {comp_names.get(comp, 'Unknown')} ({comp})")

if __name__ == "__main__":
    import glob
    files = sorted(glob.glob("*.psd"))
    for file in files:
        analyze_psd(file)
