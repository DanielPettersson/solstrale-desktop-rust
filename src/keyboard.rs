use eframe::egui;
use eframe::egui::Event::Key;
use eframe::egui::{Modifiers, Ui};

pub fn is_ctrl_r(ui: &Ui) -> bool {
    is_key_combo(ui, egui::Key::R, Modifiers::CTRL)
}

pub fn is_ctrl_space(ui: &Ui) -> bool {
    is_key_combo(ui, egui::Key::Space, Modifiers::CTRL)
}

pub fn is_enter(ui: &Ui) -> bool {
    is_key_combo(ui, egui::Key::Enter, Modifiers::NONE)
}

fn is_key_combo(ui: &Ui, pressed_key: egui::Key, modifier: Modifiers) -> bool {
    ui.input(|input| {
        for event in input.events.clone() {
            if match event {
                Key {
                    key,
                    pressed: false,
                    repeat: false,
                    modifiers,
                    physical_key: _,
                } => key == pressed_key && modifiers.matches_logically(modifier),
                _ => false,
            } {
                return true;
            }
        }
        false
    })
}
