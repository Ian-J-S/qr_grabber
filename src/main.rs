use arboard::Clipboard;
use image::{DynamicImage, RgbaImage};

type Error = Box<dyn std::error::Error>;

fn main() -> Result<(), Error> {
    let mut cb = Clipboard::new()?;

    let cb_img = cb.get_image()?;
    let img_width = cb_img.width as u32;
    let img_height = cb_img.height as u32;

    let rgba = RgbaImage::from_raw(img_width, img_height, cb_img.bytes.into_owned())
        .ok_or("Failed to parse clipboard image as RGBA")?;

    let gray = DynamicImage::ImageRgba8(rgba).to_luma8();

    let mut prepared = rqrr::PreparedImage::prepare(gray);

    let grids = prepared.detect_grids();
    if grids.is_empty() {
        return Err("No QR codes detected".into());
    }

    for grid in grids {
        match grid.decode() {
            Ok((_metadata, content)) => println!("{content}"),
            Err(e) => eprintln!("{e}"),
        }
    }

    Ok(())
}
