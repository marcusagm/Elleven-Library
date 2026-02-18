import os
import zipfile
import json
import struct

def analyze_sketch(filepath):
    print(f"\n===== Technical Analysis: {os.path.basename(filepath)} =====")
    if not zipfile.is_zipfile(filepath):
        print("Not a ZIP file.")
        return

    try:
        with zipfile.ZipFile(filepath, 'r') as zf:
            files = zf.namelist()
            print(f"File Count: {len(files)}")

            # Metadata
            if 'meta.json' in files:
                with zf.open('meta.json') as f:
                    meta = json.load(f)
                    print(f"Software: Sketch ({meta.get('app')})")
                    print(f"Version: {meta.get('appVersion')} (Build {meta.get('build')})")

            # Preview Detection
            if 'previews/preview.png' in files:
                preview_data = zf.read('previews/preview.png')
                if len(preview_data) > 24:
                    w, h = struct.unpack('>II', preview_data[16:24])
                    print(f"Preview: Available ({w}x{h})")
            else:
                print("Preview: MISSING")

            # Page and Asset count
            images = [f for f in files if f.startswith('images/')]
            pages = [f for f in files if f.startswith('pages/')]
            fonts = [f for f in files if f.startswith('fonts/')]

            print(f"Contents: {len(pages)} Pages, {len(images)} Images, {len(fonts)} Fonts")

            # Document properties
            if 'document.json' in files:
                with zf.open('document.json') as f:
                    doc = json.load(f)
                    cs = doc.get('colorSpace', 0)
                    cs_name = {0: "Unmanaged", 1: "sRGB", 2: "Display P3"}.get(cs, f"Unknown ({cs})")
                    print(f"Color Space: {cs_name}")

    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    target = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - Sketch"
    for f in sorted(os.listdir(target)):
        if f.endswith('.sketch'):
            analyze_sketch(os.path.join(target, f))
