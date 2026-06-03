use mundam_lib::core::error::AppResult;
use std::path::Path;

fn test_libraw_full_decode(path: &Path) -> AppResult<()> {
    let file_handle = std::fs::File::open(path).unwrap();
    let memory_map = unsafe { memmap2::MmapOptions::new().map(&file_handle).unwrap() };
    
    let mut raw_image = rsraw::RawImage::open(&memory_map).unwrap();
    
    // unpack the RAW data
    raw_image.unpack().unwrap();
    
    // Process the RAW data (demosaicing, white balance, etc)
    let image = raw_image.process::<{ rsraw::BIT_DEPTH_8 }>().unwrap();
    
    println!("Successfully decoded full RAW image! Width: {}, Height: {}, Colors: {}", image.width(), image.height(), image.colors());
    Ok(())
}

#[test]
fn run() {
    test_libraw_full_decode(Path::new("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Image/kdc/P003911.KDC")).unwrap();
}
