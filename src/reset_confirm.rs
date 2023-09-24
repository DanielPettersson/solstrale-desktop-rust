use eframe::egui::{Align, Context, Layout, Window};

use crate::DEFAULT_SCENE;

pub fn dialog(show_reset_confirm_dialog: &mut bool, scene_yaml: &mut String, ctx: &Context) {
    if *show_reset_confirm_dialog {
        Window::new("Do you want to reset the scene to the default?")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            *show_reset_confirm_dialog = false;
                        }
                        if ui.button("Yes!").clicked() {
                            *scene_yaml = DEFAULT_SCENE.to_owned();
                            *show_reset_confirm_dialog = false;
                        }
                    });
                });
            });
    }
}
