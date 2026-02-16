import struct
import zlib
import os
import sqlite3

def parse_clip(filepath):
    print(f"--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)
    with open(filepath, 'rb') as f:
        # Validate Magic
        magic = f.read(8)
        if magic != b'CSFCHUNK':
            print(f"Invalid Magic: {magic}")
            return

        # File Size check
        f.read(4)
        total_size = struct.unpack('>I', f.read(4))[0]
        print(f"Total Size in Header: {total_size} (Real: {filesize})")

        # Scan for chunks
        print("\n-- Generic Chunk Scanning --")
        f.seek(0)
        data = f.read() # Read all into memory (files are < 2GB usually)

        # Find all occurrences of "CHNK"
        import re
        for match in re.finditer(b'CHNK[a-zA-Z0-9]{4}', data):
            pos = match.start()
            name = match.group()
            print(f"Found {name.decode()} at offset {pos}")

            # Read 8 bytes size?
            # Assuming header structure: Name(8) + Size(8)
            try:
                s1_offset = pos + 8
                s1_bytes = data[s1_offset:s1_offset+8]
                s1 = int.from_bytes(s1_bytes, 'big')

                print(f"  Size Field 1: {s1} ({hex(s1)})")

                # Check for Zlib
                # Look in the first 100 bytes of payload
                # Payload starts at pos + 8 + 8? Or + 8 + 8 + 8?
                # Let's search in [pos+16 : pos+200]

                search_region = data[pos+16 : pos+512]
                zlib_idx = search_region.find(b'\x78\x9c')
                if zlib_idx == -1:
                    zlib_idx = search_region.find(b'\x78\xda') # High compression
                if zlib_idx == -1:
                    zlib_idx = search_region.find(b'\x78\x01') # Low compression

                if zlib_idx != -1:
                    zlib_abs_pos = pos + 16 + zlib_idx
                    print(f"  Zlib signature at offset {zlib_abs_pos} (Relative +{16+zlib_idx})")

                    try:
                        # Slice from zlib start
                        # Use decompressobj to handle unknown size
                        decompressor = zlib.decompressobj()
                        # Feed the rest of the file or chunk size
                        # If s1 is size, feed s1 bytes
                        chunk_end = pos + s1 + 16 # rough estimate
                        if chunk_end > len(data): chunk_end = len(data)

                        compressed_payload = data[zlib_abs_pos : chunk_end]
                        decompressed = decompressor.decompress(compressed_payload)

                        print(f"  -> Decompressed {len(decompressed)} bytes")
                        header_preview = decompressed[:32].hex()
                        print(f"  -> Header preview: {header_preview}")

                        if b'SQLite format 3' in decompressed:
                            print("  -> Contains SQLite DB!")
                            dbname = f"{name.decode()}_extracted.db"
                            # Handle multiple DBs?
                            if os.path.exists(dbname): dbname = f"{name.decode()}_{pos}_extracted.db"

                            with open(dbname, 'wb') as dbf:
                                dbf.write(decompressed)
                            print(f"  -> Saved to {dbname}")
                            analyze_sqlite(dbname)

                        # Look for PNG/JPEG signature in decompressed data
                        if decompressed.startswith(b'\x89PNG'):
                            print("  -> Blob is PNG Image")
                            with open(f"{name.decode()}_preview.png", 'wb') as imgf:
                                imgf.write(decompressed)
                        elif decompressed.startswith(b'\xff\xd8'):
                            print("  -> Blob is JPEG Image")
                            with open(f"{name.decode()}_preview.jpg", 'wb') as imgf:
                                imgf.write(decompressed)

                    except Exception as e:
                        print(f"  Decompression failed: {e}")
            except Exception as e:
                print(f"  Error reading size: {e}")

def analyze_sqlite(db_path):
    print(f"\n  -- Analyzing SQLite: {db_path} --")
    try:
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()

        cursor.execute("SELECT name FROM sqlite_master WHERE type='table';")
        tables = cursor.fetchall()
        print(f"  Tables: {[t[0] for t in tables]}")

        # Check CanvasPreview
        if ('CanvasPreview',) in tables:
            print("  Found CanvasPreview table")
            # Schema?
            cursor.execute("PRAGMA table_info(CanvasPreview)")
            cols = cursor.fetchall()
            print(f"  Schema: {[c[1] for c in cols]}")

            cursor.execute("SELECT * FROM CanvasPreview LIMIT 1")
            row = cursor.fetchone()
            if row:
                print(f"  Row 0 found.")
                # Try to find image blob
                for idx, val in enumerate(row):
                    if isinstance(val, bytes):
                        print(f"    Col {idx} is {len(val)} bytes blob")
                        if len(val) > 1000:
                            save_blob(val, f"{db_path}_preview.bin")

        # Check CanvasContent (Layers)
        if ('CanvasContent',) in tables:
            print("  Found CanvasContent table")

        conn.close()
    except Exception as e:
        print(f"  SQLite Error: {e}")

def save_blob(data, name):
    with open(name, 'wb') as f:
        f.write(data)
    print(f"  Saved blob to {name}")
    if data.startswith(b'\x89PNG'):
        print("    -> Valid PNG signature")
        os.rename(name, name.replace('.bin', '.png'))
    elif data.startswith(b'\xff\xd8'):
        print("    -> Valid JPEG signature")
        os.rename(name, name.replace('.bin', '.jpg'))

if __name__ == "__main__":
    parse_clip("Sketches.clip")
