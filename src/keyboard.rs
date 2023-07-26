use eframe::egui;
use eframe::egui::Event::Key;
use eframe::egui::{Modifiers, Ui};

pub fn is_ctrl_r(ui: &Ui) -> bool {
    ui.input(|input| {
        for event in input.events.clone() {
            if match event {
                Key {
                    key: egui::Key::R,
                    repeat: false,
                    modifiers,
                    ..
                } => modifiers.matches(Modifiers::CTRL),
                _ => false,
            } {
                return true;
            }
        }
        false
    })
}

pub fn is_enter(ui: &Ui) -> bool {
    ui.input(|input| {
        for event in input.events.clone() {
            if match event {
                Key {
                    key: egui::Key::Enter,
                    pressed: false,
                    repeat: false,
                    modifiers,
                } => modifiers.is_none(),
                _ => false,
            } {
                return true;
            }
        }
        false
    })
}