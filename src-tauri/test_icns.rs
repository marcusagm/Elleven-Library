use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = BufReader::new(File::open("../file-samples/Arquivos para testes/Image/icns/sample.icns").unwrap());
    let icon_family = icns::IconFamily::read(file).unwrap();
    println!("Available icons:");
    for icon in icon_family.available_icons() {
        println!("{:?}", icon);
    }
}
