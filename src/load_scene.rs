use crate::{ErrorInfo, RenderControl, save_scene};
use eframe::egui;
use eframe::egui::TextBuffer;
use egui::Context;
use egui_file_dialog::FileDialog;
use std::fs::File;
use std::io::Read;

pub fn create() -> FileDialog {
    FileDialog::new()
        .add_save_extension("Yaml", "yaml")
        .add_file_filter_extensions("Yaml files", vec!["yaml"])
}

pub fn handle_dialog(
    load_scene_dialog: &mut FileDialog,
    save_scene_dialog: &mut FileDialog,
    error_info: &mut ErrorInfo,
    scene_yaml: &mut dyn TextBuffer,
    render_control: &mut RenderControl,
    ctx: &Context,
) {
    load_scene_dialog.update(ctx);

    if let Some(file_path) = load_scene_dialog.take_picked() {
        match File::open(&file_path) {
            Ok(mut f) => {
                let mut file_content = String::new();
                match f.read_to_string(&mut file_content) {
                    Ok(_) => {
                        scene_yaml.replace_with(&file_content);
                        *save_scene_dialog = save_scene::create(Some(file_path));
                        error_info.show_error = false;
                        render_control.render_requested = true;
                    }
                    Err(err) => error_info.handle(Box::new(err)),
                };
            }
            Err(err) => error_info.handle(Box::new(err)),
        }
    }
}
