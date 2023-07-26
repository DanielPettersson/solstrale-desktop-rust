use eframe::egui::Ui;

use crate::{ErrorInfo, RenderControl};
use crate::keyboard::is_ctrl_r;

pub fn is_enabled(render_control: &RenderControl) -> bool {
    !render_control.render_requested && !render_control.loading_scene
}

pub fn handle_click(clicked: bool, render_control: &mut RenderControl, error_info: &mut ErrorInfo, ui: &Ui) {
    if clicked || is_ctrl_r(ui) {
        render_control.render_requested = true;
        error_info.show_error = false;
    }
}


