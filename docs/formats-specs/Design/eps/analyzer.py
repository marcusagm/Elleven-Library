import os
import re
import struct
import base64

def analyze_eps(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)

    with open(filepath, 'rb') as f:
        magic = f.read(4)

        if magic == b'\xc5\xd0\xd3\xc6':
            print("Format: Binary EPS")
            f.seek(4)
            data = f.read(24)
            ps_offset, ps_size, meta_offset, meta_size, tiff_offset, tiff_size = struct.unpack('<IIIIII', data)
            print(f"  PostScript: Offset={ps_offset}, Size={ps_size}")
            print(f"  WMF/Metafile: Offset={meta_offset}, Size={meta_size}")
            print(f"  TIFF Preview: Offset={tiff_offset}, Size={tiff_size}")

        elif magic.startswith(b'%!PS'):
            print("Format: ASCII PostScript EPS")
            f.seek(0)
            content = f.read()

            # Check for DSC comments
            if b'%%BeginPreview' in content:
                print("  Found %%BeginPreview segment")
                # Usually hex encoded

            if b'%%BeginMetadata' in content or b'<x:xmpmeta' in content:
                print("  Found XMP Metadata")
                xmp_match = re.search(rb'<xmpGImg:image>(.*?)</xmpGImg:image>', content, re.DOTALL)
                if xmp_match:
                    print(f"    Found XMP Thumbnail (Base64 JPEG, length={len(xmp_match.group(1))})")

            # Check for AI (Illustrator) private data which often has previews
            if b'%AI7_Thumbnail' in content:
                print("  Found %AI7_Thumbnail (Illustrator Legacy Preview)")

            # Check for TIFF signature embedded near the beginning (some EPS start with TIFF)
            if b'II*\x00' in content[:128] or b'MM\x00*' in content[:128]:
                print("  Suspicious TIFF-like signature found in header area")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Eps/"
    import glob
    files = sorted(glob.glob(os.path.join(target_dir, "*.[eE][pP][sS]")) + glob.glob(os.path.join(target_dir, "*.[pP][sS]")))
    for f in files:
        if os.path.isfile(f):
            analyze_eps(f)
