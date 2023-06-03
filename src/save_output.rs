use crate::{Dialogs, ErrorInfo, RenderedImage};
use eframe::egui;
use egui::Context;
use egui_file::FileDialog;
use image::ColorType;

pub fn show(dialogs: &mut Dialogs) {
    let mut dialog = FileDialog::save_file(None).default_filename("image.png");
    dialog.open();
    dialogs.save_output_dialog = Some(dialog);
}

pub fn handle_dialog(
    dialogs: &mut Dialogs,
    error_info: &mut ErrorInfo,
    rendered_image: &RenderedImage,
    ctx: &Context,
) {
    if let Some(dialog) = &mut dialogs.save_output_dialog {
        if dialog.show(ctx).selected() {
            if let Some(file_path) = dialog.path() {
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
            }
        }
    }
}
