//! The structured scene editor: an outline of the scene and an inspector for
//! the selected part of it.

use std::collections::BTreeSet;
use std::hash::Hash;

use eframe::egui::{self, RichText, Ui};

use crate::model::one_of::OneOf;
use crate::model::scope::Scope;

pub mod asset_picker;
pub mod inspector;
pub mod model_ui;
pub mod outline;
pub mod widgets;

#[cfg(test)]
mod ui_tests;

use asset_picker::AssetPicker;

/// What an editor needs besides the value it edits.
pub struct EditCx<'a> {
    /// Variables that expressions here can read
    pub visible: &'a BTreeSet<String>,
    /// Values to preview expressions with, when the scene evaluates this far
    pub preview: Option<&'a Scope>,
    pub assets: &'a mut AssetPicker,
}

/// A model value that can be edited in the inspector.
pub trait Edit {
    /// Returns whether the value changed
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool;
}

/// Widest the label column of a [`form`] gets
const LABEL_WIDTH: f32 = 110.;

/// Labelled rows, with the labels in a column to the left.
pub fn form(
    ui: &mut Ui,
    id_salt: impl Hash + std::fmt::Debug,
    add: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    ui.push_id(id_salt, |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.;
            add(ui)
        })
        .inner
    })
    .inner
}

/// A row in a [`form`], with `help` shown when hovering the label. The value
/// gets the width left of the panel after the label, and has to fit in it:
/// a grid would give it whatever its content asks for, pushing the rest of
/// the panel out of view.
pub fn row(ui: &mut Ui, label: &str, help: &str, add: impl FnOnce(&mut Ui) -> bool) -> bool {
    row_impl(ui, label, help, false, add)
}

/// A [`row`] for a value with rows of its own, like a material, which goes
/// under its label rather than beside it, to not be squeezed into what is
/// left beside it when nested.
pub fn block_row(ui: &mut Ui, label: &str, help: &str, add: impl FnOnce(&mut Ui) -> bool) -> bool {
    row_impl(ui, label, help, true, add)
}

fn row_impl(
    ui: &mut Ui,
    label: &str,
    help: &str,
    block: bool,
    add: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    if block {
        ui.label(label).on_hover_text(help);
        return ui.push_id(label, |ui| ui.indent("block", add).inner).inner;
    }
    ui.horizontal_top(|ui| {
        let label_width = LABEL_WIDTH.min(ui.available_width() * 0.35);
        ui.allocate_ui_with_layout(
            egui::vec2(label_width, 0.),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(label_width);
                // Level with the text of the widget next to it
                ui.add_space(2.);
                ui.add(egui::Label::new(label).wrap()).on_hover_text(help);
            },
        );
        let width = ui.available_width();
        ui.push_id(label, |ui| {
            ui.vertical(|ui| {
                ui.set_max_width(width);
                add(ui)
            })
            .inner
        })
        .inner
    })
    .inner
}

/// A [`row`] for a value the renderer has a default for. Shows the default
/// until it is changed, and leaves the value out of the scene while it is the
/// default.
pub fn default_row<T: Clone + PartialEq>(
    ui: &mut Ui,
    label: &str,
    help: &str,
    value: &mut Option<T>,
    default: T,
    edit: impl FnOnce(&mut Ui, &mut T) -> bool,
) -> bool {
    default_row_impl(ui, label, help, false, value, default, edit)
}

/// A [`default_row`] laid out as a [`block_row`]
pub fn default_block_row<T: Clone + PartialEq>(
    ui: &mut Ui,
    label: &str,
    help: &str,
    value: &mut Option<T>,
    default: T,
    edit: impl FnOnce(&mut Ui, &mut T) -> bool,
) -> bool {
    default_row_impl(ui, label, help, true, value, default, edit)
}

fn default_row_impl<T: Clone + PartialEq>(
    ui: &mut Ui,
    label: &str,
    help: &str,
    block: bool,
    value: &mut Option<T>,
    default: T,
    edit: impl FnOnce(&mut Ui, &mut T) -> bool,
) -> bool {
    row_impl(ui, label, help, block, |ui| {
        let mut v = value.clone().unwrap_or_else(|| default.clone());
        let changed = ui.vertical(|ui| edit(ui, &mut v)).inner;
        if changed {
            store_unless_default(value, v, &default);
        }
        changed
    })
}

/// `v`, or nothing when it is the default
pub fn store_unless_default<T: PartialEq>(value: &mut Option<T>, v: T, default: &T) {
    *value = (v != *default).then_some(v);
}

/// A [`row`] for an optional value that has no default, where leaving it out
/// means something, like no attenuation. Unchecked, `none_text` says what.
pub fn opt_row<T>(
    ui: &mut Ui,
    label: &str,
    help: &str,
    value: &mut Option<T>,
    none_text: &str,
    new: impl FnOnce() -> T,
    edit: impl FnOnce(&mut Ui, &mut T) -> bool,
) -> bool {
    row(ui, label, help, |ui| {
        ui.horizontal_top(|ui| {
            let mut set = value.is_some();
            let mut changed = false;
            if ui
                .checkbox(&mut set, "")
                .on_hover_text(if set {
                    "Use the default"
                } else {
                    "Set a value"
                })
                .changed()
            {
                *value = if set { Some(new()) } else { None };
                changed = true;
            }
            match value {
                Some(v) => changed |= ui.vertical(|ui| edit(ui, v)).inner,
                None => {
                    ui.label(RichText::new(none_text).weak());
                }
            }
            changed
        })
        .inner
    })
}

/// A combo box choosing the variant of a one-of value. Switching replaces the
/// value with a new one of the chosen variant.
pub fn variant_combo<T: OneOf>(ui: &mut Ui, value: &mut T) -> bool {
    let current = value.variant_index();
    let mut chosen = current;
    egui::ComboBox::from_id_salt("variant")
        .selected_text(label_for(value.variant_name()))
        .show_ui(ui, |ui| {
            for (i, name) in T::variants().iter().enumerate() {
                ui.selectable_value(&mut chosen, i, label_for(name));
            }
        });
    if chosen != current {
        *value = T::default_variant(chosen);
        true
    } else {
        false
    }
}

/// `half_screen` as `Half screen`
pub fn label_for(key: &str) -> String {
    let s = key.replace('_', " ");
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => s,
    }
}

/// A list of one-of values, each with its own editor, that can be added to,
/// reordered and removed from.
pub fn list_edit<T: OneOf + Edit>(ui: &mut Ui, list: &mut Vec<T>, cx: &mut EditCx) -> bool {
    enum Op {
        Up(usize),
        Down(usize),
        Remove(usize),
    }
    let mut op = None;
    let mut changed = false;
    let len = list.len();
    for (i, item) in list.iter_mut().enumerate() {
        ui.push_id(i, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label_for(item.variant_name())).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✖").on_hover_text("Remove").clicked() {
                            op = Some(Op::Remove(i));
                        }
                        if ui
                            .add_enabled(i + 1 < len, egui::Button::new("⏷").small())
                            .on_hover_text("Move down")
                            .clicked()
                        {
                            op = Some(Op::Down(i));
                        }
                        if ui
                            .add_enabled(i > 0, egui::Button::new("⏶").small())
                            .on_hover_text("Move up")
                            .clicked()
                        {
                            op = Some(Op::Up(i));
                        }
                    });
                });
                changed |= item.edit(ui, cx);
            });
        });
    }
    match op {
        Some(Op::Up(i)) => list.swap(i, i - 1),
        Some(Op::Down(i)) => list.swap(i, i + 1),
        Some(Op::Remove(i)) => {
            list.remove(i);
        }
        None => {}
    }
    changed |= op.is_some();

    ui.menu_button("+ Add", |ui| {
        for (i, name) in T::variants().iter().enumerate() {
            if ui.button(label_for(name)).clicked() {
                list.push(T::default_variant(i));
                changed = true;
                ui.close();
            }
        }
    });
    changed
}
