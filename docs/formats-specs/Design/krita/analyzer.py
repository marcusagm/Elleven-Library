import zipfile
import os
import sys
import struct
import xml.etree.ElementTree as ET

def analyze_kra(filepath):
    print(f"--- Analyzing {os.path.basename(filepath)} ---")
    try:
        if not zipfile.is_zipfile(filepath):
            print("Not a valid ZIP file.")
            return

        with zipfile.ZipFile(filepath, 'r') as zf:
            file_list = zf.namelist()
            print(f"Total files in archive: {len(file_list)}")

            # 1. Structure Check
            print("\n-- Key Files Found --")
            key_files = ['mimetype', 'maindoc.xml', 'preview.png', 'mergedimage.png', 'documentinfo.xml']
            for k in key_files:
                if k in file_list:
                    print(f"[x] {k} : {zf.getinfo(k).file_size} bytes")
                else:
                    print(f"[ ] {k} (Missing)")

            # 2. Mimetype
            if 'mimetype' in file_list:
                with zf.open('mimetype') as f:
                    print(f"\nMimetype content: {f.read().decode('utf-8').strip()}")

            # 3. MainDoc Analysis (Dimensions)
            if 'maindoc.xml' in file_list:
                print("\n-- MainDoc XML Analysis --")
                with zf.open('maindoc.xml') as f:
                    try:
                        tree = ET.parse(f)
                        root = tree.getroot()
                        # Namespace usually: {http://www.calligra.org/DTD/karbon}
                        # We can just search for "IMAGE" tag or attributes
                        # Krita XML is namespaced.

                        # Find IMAGE tag
                        # The namespace is usually huge, let's just iterate
                        width = 0
                        height = 0

                        for elem in root.iter():
                            if 'width' in elem.attrib and 'height' in elem.attrib:
                                # Candidate for dimensions
                                # Usually in <krita:image> or <IMAGE>
                                tag = elem.tag.lower()
                                if 'image' in tag:
                                    print(f"Found Image Tag: {elem.tag}")
                                    print(f"Dimensions: {elem.attrib['width']} x {elem.attrib['height']}")

                                    # Try to convert to int/float
                                    try:
                                        w = float(elem.attrib['width'])
                                        h = float(elem.attrib['height'])
                                        print(f"Parsed Dimensions: {int(w)}x{int(h)}")
                                    except:
                                        pass

                                    if 'mime' in elem.attrib:
                                        print(f"Internal Mime: {elem.attrib['mime']}")

                                    if 'name' in elem.attrib:
                                        print(f"Layer/Image Name: {elem.attrib['name']}")
                    except Exception as e:
                        print(f"Error parsing XML: {e}")

            # 4. Preview Extraction Check
            if 'preview.png' in file_list:
                img_info = zf.getinfo('preview.png')
                print(f"\nPreview Image available (PNG), size: {img_info.file_size} bytes")

            if 'mergedimage.png' in file_list:
                img_info = zf.getinfo('mergedimage.png')
                print(f"Merged Image available (PNG), size: {img_info.file_size} bytes")

            # 5. Layer Structure (Check folders)
            # Krita layers usually in a folder, or just in maindoc.xml.
            # Actually, binary data for layers is usually in distinct files if not embedded in XML (which is old/slow).
            # Look for 'layers/' directory or similar?
            # Krita 4+ usually stores layer data in `layername/` or `data/`?
            # Actually Krita stores binary data in files like `layer2`, `layer3` inside the zip?
            # Let's check for large binary files that are not png/xml.

            print("\n-- Large Files / Potential Layer Data --")
            for info in zf.infolist():
                if info.file_size > 100000 and not info.filename.endswith(('.png', '.xml', 'icc')):
                    print(f"{info.filename}: {info.file_size} bytes")

                if info.filename.endswith('.icc'):
                    print(f"Color Profile Found: {info.filename}")

    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    import glob
    # Running from docs/formats-specs/Design/krita/
    # So samples are in ../../../../../file-samples/Imagens/Design/Ok - Krita
    sample_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - Krita"
    files = glob.glob(os.path.join(sample_dir, "*.kra"))

    for f in files[:3]:
        analyze_kra(f)
