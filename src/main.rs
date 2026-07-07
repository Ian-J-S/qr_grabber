use std::path::Path;

use arboard::Clipboard;
use iced::{
    Element,
    alignment::Vertical,
    border, padding,
    widget::{Button, button, column, container, row, text},
};
use image::{DynamicImage, RgbaImage};
use rfd::FileDialog;

type Error = Box<dyn std::error::Error>;

const REG_FONT_SIZE: f32 = 14.0;
const HORIZONTAL_SPACING: u32 = 10;

const BUTTON_LABEL_SIZE: f32 = 12.0;
const BUTTON_RADIUS: f32 = 5.0;
const BUTTON_HEIGHT: u32 = 20;

struct App {
    clipboard: Option<Clipboard>,
    content: String,
    copied_msg: String,
    file_display_name: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            clipboard: Clipboard::new().ok(),
            content: Default::default(),
            copied_msg: Default::default(),
            file_display_name: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    FromClipboard,
    FromFile,
    NewCopiedMsg,
}

fn styled_button(label: &str, message: Message) -> Button<'_, Message> {
    button(
        text(label)
            .size(BUTTON_LABEL_SIZE)
            .align_y(Vertical::Center),
    )
    .on_press(message)
    .style(|theme, status| {
        let mut style = button::primary(theme, status);
        style.border.radius = border::Radius::from(BUTTON_RADIUS);
        style
    })
    .height(BUTTON_HEIGHT)
}

impl App {
    fn set_content(&mut self, result: Result<String, Error>) {
        self.copied_msg.clear();

        self.content = match result {
            Ok(s) => s.trim_end_matches('/').to_string(),
            Err(e) => format!("ERROR - {e}"),
        };
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FromClipboard => {
                let result = if let Some(clipboard) = self.clipboard.as_mut() {
                    qr_from_clipboard(clipboard)
                } else {
                    Err("ERROR - unable to connect to system clipboard".into())
                };
                self.set_content(result);
            }
            Message::FromFile => {
                self.copied_msg.clear();
                self.file_display_name.clear();

                let picked_file = FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
                    .pick_file();

                if let Some(file) = picked_file.as_deref() {
                    let result = qr_from_file(file);
                    self.set_content(result);

                    if let Some(basename) = file.file_name() {
                        let name = basename.to_string_lossy();

                        // Truncate filename if it's too long
                        let display = if name.chars().count() > 10 {
                            format!("{}...", name.chars().take(10).collect::<String>())
                        } else {
                            name.into_owned()
                        };

                        self.file_display_name = display;
                    }
                }
            }
            Message::NewCopiedMsg => {
                self.copied_msg = self
                    .clipboard
                    .as_mut()
                    .map(|cb| match cb.set_text(&self.content) {
                        Ok(()) => "Copied!".to_string(),
                        Err(e) => format!("ERROR - {e}"),
                    })
                    .unwrap_or_else(|| "ERROR - Unable to copy!".to_string());
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let copy_row = if !self.content.is_empty() {
            row![
                styled_button("Copy", Message::NewCopiedMsg),
                if !self.copied_msg.is_empty() {
                    text(self.copied_msg.as_str()).size(REG_FONT_SIZE)
                } else {
                    text("")
                },
            ]
            .spacing(HORIZONTAL_SPACING)
        } else {
            row![]
        };

        container(column![
            container(text("QR Grabber"),).padding(padding::bottom(5)),
            row![
                styled_button("From clipboard", Message::FromClipboard),
                styled_button("From file", Message::FromFile),
                text(&self.file_display_name).size(REG_FONT_SIZE),
            ]
            .spacing(HORIZONTAL_SPACING)
            .padding(padding::bottom(15)),
            container(text(&self.content).size(REG_FONT_SIZE)).padding(padding::bottom(5)),
            copy_row,
        ])
        .padding(10)
        .into()
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

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .window_size(iced::Size::new(300.0, 150.0))
        .run()
}
