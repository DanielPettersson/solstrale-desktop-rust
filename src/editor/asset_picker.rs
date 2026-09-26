use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui::{Context, Id};
use egui_file_dialog::FileDialog;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AssetKind {
    Obj,
    Image,
}

/// One file dialog shared by all the file fields in the inspector. Remembers
/// which field asked, so the picked file goes back to it.
#[derive(Default)]
pub struct AssetPicker {
    dialog: Option<FileDialog>,
    picked: HashMap<Id, PathBuf>,
}

impl AssetPicker {
    pub fn request(&mut self, field: Id, kind: AssetKind, current: &str) {
        let mut dialog = match kind {
            AssetKind::Obj => FileDialog::new()
                .add_file_filter_extensions("Obj models", vec!["obj"])
                .default_file_filter("Obj models"),
            AssetKind::Image => FileDialog::new()
                .add_file_filter_extensions(
                    "Images",
                    vec!["png", "jpg", "jpeg", "bmp", "tga", "hdr", "exr"],
                )
                .default_file_filter("Images"),
        };
        if let Some(dir) = Path::new(current).parent().filter(|d| d.is_dir()) {
            dialog = dialog.initial_directory(dir.to_path_buf());
        }
        dialog.set_user_data(field);
        dialog.pick_file();
        self.dialog = Some(dialog);
    }

    pub fn update(&mut self, ctx: &Context) {
        if let Some(dialog) = &mut self.dialog {
            dialog.update(ctx);
            if let Some(path) = dialog.take_picked() {
                if let Some(field) = dialog.user_data::<Id>() {
                    self.picked.insert(*field, path);
                }
                self.dialog = None;
            }
        }
    }

    /// The file picked for `field`, once
    pub fn take(&mut self, field: Id) -> Option<PathBuf> {
        self.picked.remove(&field)
    }
}
