fn main() {
    println!("{}", mime_guess::from_ext("m4a").first_or_octet_stream().to_string());
}
