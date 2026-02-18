import zipfile
import os
import xml.etree.ElementTree as ET

def analyze_reb(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    if not zipfile.is_zipfile(filepath):
        print("Error: Not a valid ZIP file.")
        return

    with zipfile.ZipFile(filepath, 'r') as zf:
        file_list = zf.namelist()
        print(f"Total files in ZIP: {len(file_list)}")

        # Check for core files
        core_files = ["artwork.xml", "canvas.png"]
        for cf in core_files:
            if cf in file_list:
                info = zf.getinfo(cf)
                print(f"  [FOUND] {cf} (Size: {info.file_size}, Compressed: {info.compress_size})")
            else:
                print(f"  [MISSING] {cf}")

        # Basic metadata from artwork.xml
        if "artwork.xml" in file_list:
            with zf.open("artwork.xml") as f:
                try:
                    tree = ET.parse(f)
                    root = tree.getroot()
                    print(f"  Root Tag: {root.tag}")

                    canvas = root.find('canvas')
                    if canvas is not None:
                        width = canvas.get('width')
                        height = canvas.get('height')
                        print(f"  Canvas Dimensions: {width}x{height}")

                    # Try to find version
                    version = root.get('version')
                    if version:
                        print(f"  Software Version: {version}")

                except Exception as e:
                    print(f"  Error parsing XML: {e}")

        # Sample some layer files
        layers = [f for f in file_list if f.startswith('layer') and f.endswith('.png')]
        print(f"  Found {len(layers)} image layers.")

        dat_files = [f for f in file_list if f.endswith('.dat')]
        print(f"  Found {len(dat_files)} simulation (.dat) files.")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - Rebelle/"
    for f in ["Gordin.reb", "portrait.reb"]:
        analyze_reb(os.path.join(target_dir, f))
