use fast_image_resize as fr;
use image::DynamicImage;
use image::ImageFormat;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn generate_thumbnail(
    input_path: &Path,
    output_path: &Path,
    size_px: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the image
    let img = image::open(input_path)?;
    let width = img.width();
    let height = img.height();

    // 2. Calculate aspect ratio
    let aspect = width as f32 / height as f32;
    let (new_w, new_h) = if aspect > 1.0 {
        (size_px, (size_px as f32 / aspect) as u32)
    } else {
        ((size_px as f32 * aspect) as u32, size_px)
    };

    // 3. Resize using fast_image_resize
    let src_image = fr::images::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    )
    .map_err(|e| e.to_string())?;

    let mut dst_image = fr::images::Image::new(new_w, new_h, fr::PixelType::U8x4);
    let mut resizer = fr::Resizer::new();
    resizer
        .resize(&src_image, &mut dst_image, None)
        .map_err(|e| e.to_string())?;

    // 4. Save as WebP
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);

    // Convert back to dynamic image for saving via the image crate
    let buffer = dst_image.buffer();
    let dynamic_img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(new_w, new_h, buffer.to_vec()).unwrap(),
    );

    dynamic_img.write_to(&mut writer.into_inner()?, ImageFormat::WebP)?;

    Ok(())
}

pub fn get_thumbnail_filename(image_path: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    image_path.hash(&mut hasher);
    format!("{:x}.webp", hasher.finish())
}
