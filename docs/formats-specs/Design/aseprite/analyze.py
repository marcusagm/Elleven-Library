import struct
import os
import zlib

def parse_aseprite(filepath):
    print(f"--- Analyzing {os.path.basename(filepath)} ---")
    with open(filepath, 'rb') as f:
        # Header (128 bytes)
        header_data = f.read(128)
        if len(header_data) < 128:
            print("File too short for header")
            return

        file_size, magic, frames, width, height, depth, flags, speed = struct.unpack('<IHHHHHIH', header_data[:20])
        print(f"File Size: {file_size}")
        print(f"Magic: {hex(magic)} (Expected 0xA5E0)")
        print(f"Frames: {frames}")
        print(f"Dimensions: {width}x{height} @ {depth}bpp")
        print(f"Flags: {hex(flags)}")

        # Checking 0x00 at 28 (Palette entry)
        palette_entry = header_data[28]
        print(f"Palette Entry (Transp Index): {palette_entry}")

        # Checking num colors
        num_colors = struct.unpack('<H', header_data[32:34])[0]
        print(f"Number of Colors: {num_colors}")

        if magic != 0xA5E0:
            print("Invalid Magic Number")
            return

        # Frame Parsing
        current_offset = 128
        frame_count = 0

        # Only parse first few frames for brevity
        while frame_count < min(frames, 3):
            f.seek(current_offset)
            frame_header_data = f.read(16)
            if len(frame_header_data) < 16:
                break

            frame_size, frame_magic, old_chunks, frame_duration, _, new_chunks = struct.unpack('<IHHH2sI', frame_header_data)

            print(f"Frame {frame_count}: Size={frame_size}, Magic={hex(frame_magic)}, Duration={frame_duration}ms, Chunks={new_chunks}")

            if frame_magic != 0xF1FA:
                print("Invalid Frame Magic")
                break

            chunk_count = new_chunks
            if chunk_count == 0:
                chunk_count = old_chunks

            chunk_offset = current_offset + 16

            for i in range(chunk_count):
                f.seek(chunk_offset)
                chunk_header_data = f.read(6)
                if len(chunk_header_data) < 6:
                    break

                chunk_size, chunk_type = struct.unpack('<IH', chunk_header_data)
                print(f"  Chunk {i}: Type={hex(chunk_type)} ({get_chunk_name(chunk_type)}), Size={chunk_size}")

                # Analyze Cel Chunk for compression
                if chunk_type == 0x2005:
                    cel_header_data = f.read(16) # Read standard celestial header part
                    # layer (2), x(2), y(2), opacity(1), type(2), reserved(7) = 14 bytes
                    # structure:
                    # WORD layer index
                    # SHORT x
                    # SHORT y
                    # BYTE opacity
                    # WORD type
                    # BYTE[7] reserved

                    # Read 16 bytes to be safe? No, let's read exactly according to spec.
                    f.seek(chunk_offset + 6) # skip chunk header
                    cel_data = f.read(16)
                    layer_idx, x, y, opacity, cel_type = struct.unpack('<HhhBH', cel_data[:9]) # 2+2+2+1+2 = 9 bytes?
                    # The spec says:
                    # WORD Layer Index
                    # SHORT X
                    # SHORT Y
                    # BYTE Opacity
                    # WORD Type
                    # BYTE[7] Reserved
                    # Total 2+2+2+1+2+7 = 16 bytes.

                    print(f"    Cel: Layer={layer_idx}, Pos=({x},{y}), Opacity={opacity}, Type={cel_type}")

                    if cel_type == 2: # Compressed Image
                        width, height = struct.unpack('<HH', f.read(4)) # offset 16+6 = 22.
                        # This f.read continuation depends on where the file pointer is.
                        # f.read(16) advanced it 16 bytes.
                        # So we are at offset 22 relative to chunk start.
                        print(f"      Compressed Image Size: {width}x{height}")
                        zlib_header = f.read(2)
                        print(f"      Zlib Header: {zlib_header.hex()}")


                # Analyze Palette Chunk
                if chunk_type == 0x2019:
                    palette_size, first_idx, last_idx = struct.unpack('<III', f.read(12))
                    print(f"    Palette: {palette_size} size, Indices {first_idx}-{last_idx}")

                chunk_offset += chunk_size

            current_offset += frame_size
            frame_count += 1

def get_chunk_name(chunk_type):
    names = {
        0x0004: "Old Palette 1",
        0x0011: "Old Palette 2",
        0x2004: "Layer",
        0x2005: "Cel",
        0x2006: "Cel Extra",
        0x2007: "Color Profile",
        0x2016: "Mask (Deprecated)",
        0x2017: "Path",
        0x2018: "Tags",
        0x2019: "Palette",
        0x2020: "User Data",
        0x2022: "Slice",
        0x2023: "Tileset"
    }
    return names.get(chunk_type, "Unknown")

import glob
files = glob.glob("*.aseprite")
for file in files:
    parse_aseprite(file)
    print("\n")
