use crate::{Dialogs, ErrorInfo};
use eframe::egui;
use eframe::egui::TextBuffer;
use egui::Context;
use egui_file::FileDialog;
use std::fs;

pub fn show(dialogs: &mut Dialogs) {
    let mut dialog = FileDialog::save_file(None).default_filename("scene.yaml");
    dialog.open();
    dialogs.save_scene_dialog = Some(dialog);
}

pub fn handle_dialog(
    dialogs: &mut Dialogs,
    error_info: &mut ErrorInfo,
    scene_yaml: &dyn TextBuffer,
    ctx: &Context,
) {
    if let Some(dialog) = &mut dialogs.save_scene_dialog {
        if dialog.show(ctx).selected() {
            if let Some(file_path) = dialog.path() {
                if let Err(err) = fs::write(file_path, scene_yaml.as_str()) {
                    error_info.handle(Box::new(err))
                }
            }
        }
    }
}
