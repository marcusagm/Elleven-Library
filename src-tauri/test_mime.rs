#[test]
fn test_m4a() {
    println!("{}", mime_guess::from_path("test.m4a").first_or_octet_stream().to_string());
}
