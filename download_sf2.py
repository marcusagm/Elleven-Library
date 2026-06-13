import urllib.request

url = "https://freepats.zenvoid.org/SoundSets/general-midi/freepats-general-midi.zip"
# actually freepats is a zip, we need a raw sf2.

urls = [
    "https://musical-artifacts.com/artifacts/125/downloads/8MBGMSFX.SF2",
    "https://raw.githubusercontent.com/sinshu/rustysynth/master/testsrc/TimGM6mb.sf2", # Not sure if it exists
    "https://github.com/urish/cinto/raw/master/media/TimGM6mb.sf2",
]

for url in urls:
    try:
        print(f"Trying {url}")
        urllib.request.urlretrieve(url, "src-tauri/resources/soundfonts/soundfont.sf2")
        with open("src-tauri/resources/soundfonts/soundfont.sf2", "rb") as f:
            header = f.read(4)
            if header == b"RIFF":
                print("Downloaded successfully!")
                exit(0)
    except Exception as e:
        print(e)
