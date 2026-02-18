import zipfile
import os

def analyze_xmind(filepath):
    print(f"\n--- Analyzing {os.path.basename(filepath)} ---")
    try:
        with zipfile.ZipFile(filepath, 'r') as zf:
            file_list = zf.namelist()
            print(f"Total files: {len(file_list)}")

            # Identify core files
            core_patterns = ["metadata.json", "content.json", "Thumbnails/", "Thumbnails/thumbnail.png", "Revisions/", "manifest.json", "content.xml"]
            found_cores = [f for f in file_list if any(p in f for p in core_patterns)]
            print(f"Interested files found: {found_cores}")

            # Check for thumbnails
            thumbnails = [f for f in file_list if "Thumbnails" in f]
            if thumbnails:
                print(f"Thumbnails found: {thumbnails}")
                for t in thumbnails:
                    info = zf.getinfo(t)
                    print(f"  - {t}: {info.file_size} bytes")

            # Peek into manifest or metadata
            manifest = [f for f in file_list if "manifest.json" in f or "manifest.xml" in f or "META-INF/manifest.xml" in f]
            if manifest:
                print(f"Manifest found: {manifest}")
                with zf.open(manifest[0]) as f:
                    content = f.read(200)
                    print(f"  Peek: {content}")

            # Check if it's based on JSON or XML
            if "content.json" in file_list:
                print("Type: JSON-based (XMind 8 Update 2 or newer, XMind Zen/Pro)")
            elif "content.xml" in file_list:
                print("Type: XML-based (XMind 8 or older)")

    except zipfile.BadZipFile:
        print("Error: Not a valid ZIP file.")

if __name__ == "__main__":
    target_dir = "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Imagens/Design/Ok - XMind/"
    xmind_files = [f for f in os.listdir(target_dir) if f.endswith(".xmind")]
    for f in xmind_files[:3]:  # Analyze first 3
        analyze_xmind(os.path.join(target_dir, f))
