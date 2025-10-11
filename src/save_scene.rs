use crate::ErrorInfo;
use eframe::egui::TextBuffer;
use egui::Context;
use egui_file_dialog::FileDialog;
use std::fs;
use std::path::PathBuf;

pub fn create(initial_path: Option<PathBuf>) -> FileDialog {
    let mut dialog = FileDialog::new();

    match initial_path {
        Some(path) => {
            if path.is_dir() {
                dialog = dialog.initial_directory(path);
            } else if let Some(parent) = path.parent() {
                dialog = dialog.initial_directory(parent.to_path_buf());
                if let Some(file_name) = path.file_name() {
                    dialog = dialog.default_file_name(file_name.to_string_lossy().as_str());
                }
            }
        }
        None => dialog = dialog.default_file_name("scene.yaml"),
    };

    dialog = dialog.add_file_filter_extensions("Yaml files", vec!["yaml"]);

    dialog
}

pub fn handle_dialog(
    dialog: &mut FileDialog,
    error_info: &mut ErrorInfo,
    scene_yaml: &dyn TextBuffer,
    ctx: &Context,
) {
    dialog.update(ctx);

    if let Some(file_path) = dialog.take_picked() {
        if let Err(err) = fs::write(file_path, scene_yaml.as_str()) {
            error_info.handle(Box::new(err))
        }
    }
}
