use crate::{Dialogs, ErrorInfo, RenderControl};
use eframe::egui;
use eframe::egui::TextBuffer;
use egui::Context;
use egui_file::FileDialog;
use std::fs::File;
use std::io::Read;

pub fn show(dialogs: &mut Dialogs) {
    let mut dialog = FileDialog::open_file(None);
    dialog = dialog.filter(Box::new(|f| match f.extension() {
        None => false,
        Some(ext) => ext.eq_ignore_ascii_case("yaml"),
    }));
    dialog.open();
    dialogs.load_scene_dialog = Some(dialog);
}

pub fn handle_dialog(
    dialogs: &mut Dialogs,
    error_info: &mut ErrorInfo,
    scene_yaml: &mut dyn TextBuffer,
    render_control: &mut RenderControl,
    ctx: &Context,
) {
    if let Some(dialog) = &mut dialogs.load_scene_dialog {
        if dialog.show(ctx).selected() {
            if let Some(file_path) = dialog.path() {
                match File::open(file_path) {
                    Ok(mut f) => {
                        let mut file_content = String::new();
                        match f.read_to_string(&mut file_content) {
                            Ok(_) => {
                                scene_yaml.replace(&file_content);
                                render_control.render_requested = true;
                            }
                            Err(err) => error_info.handle(Box::new(err)),
                        };
                    }
                    Err(err) => error_info.handle(Box::new(err)),
                }
            }
        }
    }
}
