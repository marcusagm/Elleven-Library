use mundam_lib::core::formats::registry::FormatRegistry;
use mundam_lib::core::formats::provider::FormatProvider;
use mundam_lib::processing::media::providers::audio::mpeg4_audio::Mpeg4AudioProvider;
use mundam_lib::processing::media::providers::audio::midi::MidiAudioProvider;
use mundam_lib::core::formats::capabilities::MetadataCapability;
use std::path::Path;

#[tokio::main]
async fn main() {
    let path = Path::new("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Audio/m4a/audio_only.m4a");
    let provider = Mpeg4AudioProvider::new();
    let meta = provider.metadata().unwrap().extract_technical(path).await;
    println!("m4a meta: {:?}", meta);

    let mid_path = Path::new("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Audio/mid/bohemian_rhapsody.mid");
    let mid_provider = MidiAudioProvider::new();
    let mid_meta = mid_provider.metadata().unwrap().extract_technical(mid_path).await;
    println!("mid meta: {:?}", mid_meta);
}
