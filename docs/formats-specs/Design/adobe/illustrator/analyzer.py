import os
import re
import struct

def analyze_ai(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    filesize = os.path.getsize(filepath)

    with open(filepath, 'rb') as f:
        data = f.read(4096) # Read first block

        # PDF Check
        if data.startswith(b'%PDF-'):
            version = data[5:8].decode()
            print(f"Format: PDF-compatible Illustrator (Version {version})")

        f.seek(0)
        content = f.read()

        # Look for Private Data markers
        markers = [
            b'%AI9_PrivateData',
            b'%AI7_Thumbnail',
            b'%%BeginData',
            b'%%EndData',
            b'%%BeginMetadata',
            b'%Adobe_Direct_PGF',
            b'Adobe_AGP',
            b'/Thumb'
        ]

        for m in markers:
            pos = content.find(m)
            if pos != -1:
                print(f"Found Marker: {m.decode()} at offset {pos} (0x{pos:X})")
                # Preview a bit of data after marker
                snippet = content[pos:pos+100]
                print(f"  Snippet: {snippet}")

        # Search for XMP Metadata
        xmp_start = content.find(b'<x:xmpmeta')
        if xmp_start != -1:
            xmp_end = content.find(b'</x:xmpmeta>', xmp_start)
            if xmp_end != -1:
                print(f"XMP Metadata found at {xmp_start} to {xmp_end}")
                xmp_data = content[xmp_start:xmp_end+12]
                # Check for thumbnails in XMP
                if b'xmpGImg:image' in xmp_data:
                    print("  XMP Thumbnail found (Base64 JPEG expected)")

        # PDF Object scanning
        # Look for the Catalog and AI-specific keys
        catalog_pos = content.find(b'/Type /Catalog')
        if catalog_pos != -1:
            print(f"PDF Catalog found at {catalog_pos}")

        # Search for AI Thumbnail markers (PostScript style)
        if b'%%BeginPreview' in content:
            print("Legacy PostScript Preview found (%%BeginPreview)")

if __name__ == "__main__":
    import glob
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Adobe/Illustrator/"
    files = glob.glob(os.path.join(target_dir, "*.ai"))
    for f in files:
        analyze_ai(f)
