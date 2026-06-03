use ddsfile::Dds;
use std::fs::File;
use std::io::BufReader;

#[test]
fn test_dds() {
    let mut f = File::open("../../file-samples/Arquivos para testes/Image/dds/sample.dds").unwrap();
    let mut reader = BufReader::new(f);
    let dds = Dds::read(&mut reader).unwrap();
    println!("{}x{}", dds.get_width(), dds.get_height());
    let img = image_dds::image_from_dds(&dds, 0).unwrap();
    println!("{}x{}", img.width(), img.height());
}

#[test]
fn test_icns() {
    let file = BufReader::new(File::open("../../file-samples/Arquivos para testes/Image/icns/sample.icns").unwrap());
    let icon_family = icns::IconFamily::read(file).unwrap();
    println!("Available icons:");
    for icon in icon_family.available_icons() {
        println!("{:?}", icon);
    }
}
