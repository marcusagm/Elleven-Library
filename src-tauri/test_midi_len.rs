use std::fs::File;
use std::sync::Arc;
use rustysynth::{Settings, SoundFont, Synthesizer, MidiFile};

fn main() {
    let midi_data = std::fs::read("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Audio/mid/bohemian_rhapsody.mid").unwrap();
    let midi_file = MidiFile::new(&mut std::io::Cursor::new(midi_data)).unwrap();
    println!("Length: {}", midi_file.get_length());
}
