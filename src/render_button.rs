use eframe::egui;
use eframe::egui::Event::Key;
use eframe::egui::{Modifiers, Ui};

use crate::RenderControl;

pub fn handle_click(clicked: bool, render_control: &mut RenderControl, ui: &Ui) {
    if clicked || is_ctrl_r(ui) {
        render_control.render_requested = true;
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
