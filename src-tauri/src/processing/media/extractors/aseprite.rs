//! Aseprite file preview extractor.
//!
//! Uses `asefile` to parse .ase/.aseprite files.
//! Provides static PNG for single frames and Animated GIF for multi-frame animations.

use asefile::AsepriteFile;
use image::{codecs::gif::{GifEncoder, Repeat}, Delay, DynamicImage, Frame};
use std::path::Path;
use std::time::Duration;

pub fn extract_aseprite_preview(path: &Path) -> Result<(Vec<u8>, String), Box<dyn std::error::Error>> {
    let ase = AsepriteFile::read_file(path)?;
    let total = ase.num_frames();
    if total == 0 { return Err("No frames".into()); }

    if total == 1 {
        let frame = ase.frame(0);
        let img_v24 = frame.image();
        let (w, h) = (img_v24.width(), img_v24.height());
        let pix = img_v24.into_raw();
        let img_v25 = image::RgbaImage::from_raw(w, h, pix).ok_or("Failed re-wrap")?;
        let mut buf = Vec::new();
        DynamicImage::ImageRgba8(img_v25).write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
        Ok((buf, "image/png".to_string()))
    } else {
        let mut buf = Vec::new();
        {
            let mut enc = GifEncoder::new(&mut buf);
            enc.set_repeat(Repeat::Infinite)?;
            for i in 0..total {
                let f = ase.frame(i);
                let img_v24 = f.image();
                let (w, h) = (img_v24.width(), img_v24.height());
                let pix = img_v24.into_raw();
                let img_v25 = image::RgbaImage::from_raw(w, h, pix).ok_or("Failed re-wrap anim")?;
                let delay = Delay::from_saturating_duration(Duration::from_millis(f.duration() as u64));
                enc.encode_frame(Frame::from_parts(img_v25, 0, 0, delay))?;
            }
        }
        Ok((buf, "image/gif".to_string()))
    }
}
