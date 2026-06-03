use mundam_lib::processing::media::extractors::image::*;
use std::path::Path;

#[test]
fn test_kdc_libraw() {
    let paths = [
        "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Image/kdc/P003911.KDC",
        "/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Image/kdc/RAW_KODAK_DC120_WITH_JPEG.KDC",
    ];
    for p in paths {
        let path = Path::new(p);
        println!("Testing LibRaw for {:?}", p);
        let res = generate_raw_thumbnail(path, 512);
        match res {
            Ok(bytes) => println!("Success! Generated WebP {} bytes", bytes.len()),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
