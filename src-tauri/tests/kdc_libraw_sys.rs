use std::ffi::CString;
use std::path::Path;

#[test]
fn test_libraw_open_file() {
    let path = Path::new("/Users/marcusmaia/Documents/Desenvolvimento/Mundam/file-samples/Arquivos para testes/Image/kdc/P003911.KDC");
    let c_path = CString::new(path.to_str().unwrap()).unwrap();

    unsafe {
        let raw_data = rsraw_sys::libraw_init(0);
        let res = rsraw_sys::libraw_open_file(raw_data, c_path.as_ptr());
        println!("libraw_open_file returned: {}", res);
        rsraw_sys::libraw_close(raw_data);
    }
}
