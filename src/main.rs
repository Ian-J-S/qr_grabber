use eframe::egui;
use arboard::Clipboard;
use image::{DynamicImage, RgbaImage};

type Error = Box<dyn std::error::Error>;

struct App {
    clipboard: Option<Clipboard>,
    content: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let clipboard = Clipboard::new().ok();
        App {
            clipboard,
            content: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("QR Grabber");
        });
    }
}

fn qr_decode() -> Result<(), Error> {
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

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([250.0, 110.0]),
        ..Default::default()
    };

    eframe::run_native(
        "QR Grabber",
        options,
        Box::new(|_cc| {
            Ok(Box::<App>::default())
        }),
    )
}
