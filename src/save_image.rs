use crate::{ErrorInfo, RenderedImage};
use eframe::egui;
use egui::Context;
use egui_file_dialog::FileDialog;
use image::ColorType;
use solstrale::util::wgpu_util::buffer_to_image;

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
        if let Some(render_resources) = rendered_image.render_resources.as_ref() {
            let image_buffer = rendered_image
                .output_buffer
                .as_ref()
                .expect("Dialog is only displayed when there is an image")
                .as_ref();

            let image = buffer_to_image(
                &render_resources.device,
                &render_resources.queue,
                image_buffer,
                rendered_image.width,
                rendered_image.height,
            );

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
