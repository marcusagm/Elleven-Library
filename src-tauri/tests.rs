#[test]
fn test_length() {
    let midi_data = std::fs::read("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Audio/mid/bohemian_rhapsody.mid").unwrap();
    let midi_file = rustysynth::MidiFile::new(&mut std::io::Cursor::new(midi_data)).unwrap();
    println!("Length: {}", midi_file.get_length());
}
