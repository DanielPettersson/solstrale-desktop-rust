use egui_file_dialog::FileDialog;

pub fn create() -> FileDialog {
    FileDialog::new()
        .add_save_extension("Yaml", "yaml")
        .add_file_filter_extensions("Yaml files", vec!["yaml"])
}
