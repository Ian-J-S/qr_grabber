use arboard::Clipboard;
use eframe::egui;
use image::{DynamicImage, RgbaImage};

type Error = Box<dyn std::error::Error>;

struct App {
    clipboard: Option<Clipboard>,
    content: Option<String>,
    copied_msg: String,
}

impl Default for App {
    fn default() -> Self {
        let clipboard = Clipboard::new().ok();
        App {
            clipboard,
            content: None,
            copied_msg: String::from(""),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("QR Grabber");
            if self.clipboard.is_none() {
                self.content = Some("ERROR - unable to connect to system clipboard!".to_string());
            } else if ui.button("From clipboard").clicked()
                && let Some(clipboard) = self.clipboard.as_mut()
            {
                self.content = Some(match qr_from_clipboard(clipboard) {
                    Ok(s) => s.trim_end_matches('/').to_string(), // strip '/' from end
                    Err(e) => format!("ERROR - {e}"),
                });
            }

            ui.add_space(10.0);

            if let Some(content) = self.content.as_deref() {
                ui.label(content);

                ui.horizontal(|ui| {
                    if ui.button("Copy").clicked() {
                        self.copied_msg = self
                            .clipboard
                            .as_mut()
                            .map(|cb| match cb.set_text(content) {
                                Ok(()) => "Copied!".to_string(),
                                Err(e) => format!("ERROR - {e}"),
                            })
                            .unwrap_or_else(|| "ERROR - Unable to copy!".to_string());
                    }

                    if !self.copied_msg.is_empty() {
                        ui.label(&self.copied_msg);
                    }
                });
            }
        });
    }
}

fn qr_from_clipboard(cb: &mut Clipboard) -> Result<String, Error> {
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

    let mut last_error = None;

    for grid in grids {
        match grid.decode() {
            Ok((_metadata, content)) => return Ok(content),
            Err(e) => last_error = Some(e),
        }
    }

    match last_error {
        Some(e) => Err(e.into()),
        None => Err("No readable QR codes detected".into()),
    }
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
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}
