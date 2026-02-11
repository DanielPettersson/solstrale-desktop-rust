use crate::{ErrorInfo, RenderedImage};
use eframe::egui;
use egui::Context;
use egui_file_dialog::FileDialog;
use image::ColorType;

pub fn create() -> FileDialog {
    FileDialog::new().default_file_name("image.png")
}

pub fn handle_dialog(
    dialog: &mut FileDialog,
    error_info: &mut ErrorInfo,
    rendered_image: &RenderedImage,
    ctx: &Context,
) {
    dialog.update(ctx);

    if let Some(file_path) = dialog.take_picked() {
        /*
        let image = rendered_image
            .rgb_image
            .as_ref()
            .expect("Dialog is only displayed when there is an image");
        if let Err(err) = image::save_buffer(
            file_path,
            image.as_ref(),
            image.width(),
            image.height(),
            ColorType::Rgb8,
        ) {
            error_info.handle(Box::new(err));
        }
        */
    }
}
