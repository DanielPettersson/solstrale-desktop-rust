use crate::ErrorInfo;
use eframe::egui;
use eframe::egui::TextBuffer;
use egui::Context;
use egui_file::FileDialog;
use std::fs;
use std::path::PathBuf;

pub fn create(initial_path: Option<PathBuf>) -> FileDialog {
    let mut dialog = FileDialog::save_file(initial_path.clone());

    if initial_path.is_none() {
        dialog = dialog.default_filename("scene.yaml");
    }

    dialog = dialog.filter(Box::new(|f| match f.extension() {
        None => false,
        Some(ext) => ext.eq_ignore_ascii_case("yaml"),
    }));

    dialog
}

pub fn handle_dialog(
    dialog: &mut FileDialog,
    error_info: &mut ErrorInfo,
    scene_yaml: &dyn TextBuffer,
    ctx: &Context,
) {
    if dialog.show(ctx).selected() {
        if let Some(file_path) = dialog.path() {
            if let Err(err) = fs::write(file_path, scene_yaml.as_str()) {
                error_info.handle(Box::new(err))
            }
        }
    }
}
