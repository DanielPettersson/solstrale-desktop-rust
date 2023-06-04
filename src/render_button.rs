use eframe::egui;
use eframe::egui::Event::Key;
use eframe::egui::{Modifiers, Ui};

use crate::{ErrorInfo, RenderControl};

pub fn is_enabled(render_control: &RenderControl) -> bool {
    !render_control.render_requested && !render_control.loading_scene
}

pub fn handle_click(clicked: bool, render_control: &mut RenderControl, error_info: &mut ErrorInfo, ui: &Ui) {
    if clicked || is_ctrl_r(ui) {
        render_control.render_requested = true;
        error_info.show_error = false;
    }
}

fn is_ctrl_r(ui: &Ui) -> bool {
    ui.input(|input| {
        for event in input.events.clone() {
            if match event {
                Key {
                    key: egui::Key::R,
                    pressed: _pressed,
                    repeat: false,
                    modifiers,
                } => modifiers.matches(Modifiers::CTRL),
                _ => false,
            } {
                return true;
            }
        }
        false
    })
}
