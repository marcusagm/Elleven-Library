use std::path::Path;

#[test]
fn run() {
    let raw_data = std::fs::read("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Image/kdc/P003911.KDC").unwrap();
    let res = quickraw::Export::export_thumbnail_data(&raw_data);
    match res {
        Ok((data, _)) => println!("Quickraw extracted {} bytes", data.len()),
        Err(e) => println!("Quickraw failed: {:?}", e),
    }
}
