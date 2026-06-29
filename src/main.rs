use std::path::{Path, PathBuf};

use arboard::Clipboard;
use eframe::egui;
use image::{DynamicImage, RgbaImage};
use rfd::FileDialog;

type Error = Box<dyn std::error::Error>;

struct App {
    clipboard: Option<Clipboard>,
    content: Option<String>,
    copied_msg: String,
    picked_file: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        let clipboard = Clipboard::new().ok();
        App {
            clipboard,
            content: None,
            copied_msg: String::from(""),
            picked_file: None,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("QR Grabber");
            ui.horizontal(|ui|{
                if self.clipboard.is_none() {
                    self.content = Some("ERROR - unable to connect to system clipboard!".to_string());
                } else if ui.button("From clipboard").clicked()
                    && let Some(clipboard) = self.clipboard.as_mut()
                {
                    self.content = Some(match qr_from_clipboard(clipboard) {
                        Ok(s) => s.trim_end_matches('/').to_string(), // strip '/' from end
                        Err(e) => format!("ERROR - {e}"),
                    });
                    // Reset "Copied!" message when a new image is loaded
                    if !self.copied_msg.is_empty() {
                        self.copied_msg.clear();
                    }
                }
                // Button - open default file picker and look for 
                // QR codes within
                else if ui.button("From file").clicked() {
                    self.content = None;
                    self.copied_msg.clear();
                    self.picked_file = FileDialog::new().pick_file();

                    if let Some(file) = self.picked_file.as_deref() {
                        self.content = Some(match qr_from_file(file) {
                            Ok(s) => s.trim_end_matches('/').to_string(), // strip '/' from end
                            Err(e) => format!("ERROR - {e}"),
                        });

                        if let Some(basename) = file.file_name() {
                            let name = basename.to_string_lossy();

                            // Truncate filename if it's too long
                            let display = if name.chars().count() > 30 {
                                format!("{}...", name.chars().take(10).collect::<String>())
                            } else {
                                name.into_owned()
                            };

                            ui.label(display);
                        }
                    }
                }

            });

            ui.add_space(10.0);

            // Show decoded content
            if let Some(content) = self.content.as_deref() {
                ui.label(content);

                // Button - copies QR contents to clipboard
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

    // Convert copied image so that rqrr can read it
    let rgba = RgbaImage::from_raw(img_width, img_height, cb_img.bytes.into_owned())
        .ok_or("Failed to parse clipboard image as RGBA")?;
    let gray = DynamicImage::ImageRgba8(rgba).to_luma8();

    decode_from_grayscale(gray)
}

fn qr_from_file(path: &Path) -> Result<String, Error> {
    let img = image::open(path)?.into_luma8();
        
    decode_from_grayscale(img)
}

fn decode_from_grayscale(img: image::GrayImage) -> Result<String, Error> {
    let mut prepared = rqrr::PreparedImage::prepare(img);

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
        None => Err("No QR codes detected".into()),
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
