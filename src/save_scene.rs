use egui_file_dialog::FileDialog;
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
                    dialog = dialog.default_file_name(&file_name.to_string_lossy());
                }
            }
        }
        None => dialog = dialog.default_file_name("scene.yaml"),
    };

    dialog = dialog.add_file_filter_extensions("Yaml files", vec!["yaml"]);

    dialog
}
