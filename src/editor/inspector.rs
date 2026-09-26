//! Shows the editor for what is selected in the outline.

use std::collections::BTreeSet;

use eframe::egui::{self, RichText, Ui};

use crate::editor::asset_picker::AssetPicker;
use crate::editor::outline::{Selection, enclosing_loops, get_mut};
use crate::editor::widgets::{name_edit, num_edit};
use crate::editor::{Edit, EditCx, form, opt_row};
use crate::model::num::Num;
use crate::model::render_config::RenderConfig;
use crate::model::rgb::Rgb;
use crate::model::scene::Scene;
use crate::model::scope::{BUILTINS, Scope, check_name};
use crate::model::variables::Variables;

fn builtins() -> BTreeSet<String> {
    BUILTINS.iter().map(|s| s.to_string()).collect()
}

/// Names expressions at `selection` can read, and a scope to preview them in.
/// A repeat's own loop variable is not visible in its from, to and step.
pub fn scope_at(
    scene: &Scene,
    selection: &Selection,
    frame: usize,
) -> (BTreeSet<String>, Option<Scope>) {
    let mut visible = builtins();
    visible.extend(scene.variables.0.iter().map(|(n, _)| n.clone()));
    let mut preview = scene.scope(&Scope::builtin(frame)).ok();

    if let Selection::Hittable(path) = selection {
        for r in enclosing_loops(&scene.world, path) {
            visible.insert(r.variable.clone());
            preview = preview.and_then(|s| {
                let from = r.from.as_ref().map_or(Ok(0.), |f| f.eval_scope(&s)).ok()?;
                Some(s.with(&r.variable, from))
            });
        }
    }
    (visible, preview)
}

/// What to do about a view that has moved away from the scene's camera
#[derive(Debug, PartialEq)]
pub enum ViewRequest {
    /// Move the view back to the camera
    Reset,
    /// Write the view into the camera, replacing its expressions
    UseView,
}

/// Returns whether the scene changed. `view_moved` is whether the viewport
/// shows something else than the scene's camera, which happens when the
/// camera is placed by expressions that dragging does not overwrite.
pub fn inspector(
    ui: &mut Ui,
    scene: &mut Scene,
    selection: &Selection,
    frame: usize,
    assets: &mut AssetPicker,
    view_moved: bool,
    view_request: &mut Option<ViewRequest>,
) -> bool {
    let (visible, preview) = scope_at(scene, selection, frame);
    let mut cx = EditCx {
        visible: &visible,
        preview: preview.as_ref(),
        assets,
    };

    ui.push_id(selection, |ui| match selection {
        Selection::Scene => {
            ui.heading("Scene");
            ui.add_space(4.);
            let mut changed = form(ui, "scene", |ui| {
                opt_row(
                    ui,
                    "Background",
                    "The color where a ray hits nothing",
                    &mut scene.background_color,
                    "Black",
                    || Rgb::new(0., 0., 0.),
                    |ui, c| crate::editor::widgets::rgb_edit(ui, c, &cx),
                )
            });
            ui.add_space(8.);
            ui.label(RichText::new("Variables").strong()).on_hover_text(
                "Named values that expressions in the scene can use. Each can use the ones above it",
            );
            changed |= variables_edit(ui, &mut scene.variables, frame, cx.assets);
            changed
        }
        Selection::Camera => {
            ui.heading("Camera");
            ui.add_space(4.);
            if view_moved {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(
                        "The view has been moved, but the camera is placed by expressions, \
                         which moving the view does not change.",
                    );
                    ui.horizontal(|ui| {
                        if ui
                            .button("Reset view")
                            .on_hover_text("Move the view back to where the expressions place the camera")
                            .clicked()
                        {
                            *view_request = Some(ViewRequest::Reset);
                        }
                        if ui
                            .button("Replace with view")
                            .on_hover_text("Replace the expressions for look from and look at with the view's position")
                            .clicked()
                        {
                            *view_request = Some(ViewRequest::UseView);
                        }
                    });
                });
                ui.add_space(4.);
            }
            scene.camera.edit(ui, &mut cx)
        }
        Selection::RenderConfig => {
            ui.heading("Render configuration");
            ui.add_space(4.);
            let mut config = scene.render_configuration.clone().unwrap_or_default();
            let changed = config.edit(ui, &mut cx);
            if changed {
                scene.render_configuration =
                    (config != RenderConfig::default()).then_some(config);
            }
            changed
        }
        Selection::Hittable(path) => match get_mut(&mut scene.world, path) {
            Some(h) => h.edit(ui, &mut cx),
            None => {
                ui.label(RichText::new("Nothing selected").weak());
                false
            }
        },
    })
    .inner
}

fn variables_edit(
    ui: &mut Ui,
    vars: &mut Variables,
    frame: usize,
    assets: &mut AssetPicker,
) -> bool {
    let mut changed = false;
    let mut remove = None;
    let mut visible = builtins();
    let mut scope = Some(Scope::builtin(frame));
    let names: Vec<String> = vars.0.iter().map(|(n, _)| n.clone()).collect();

    egui::Grid::new("variables")
        .num_columns(3)
        .spacing([8., 6.])
        .show(ui, |ui| {
            for (i, (name, value)) in vars.0.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    let others: Vec<&String> = names
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i)
                        .map(|(_, n)| n)
                        .collect();
                    changed |= name_edit(ui, name, |n| {
                        check_name(n)?;
                        if others.iter().any(|o| *o == n) {
                            return Err(format!("`{}` is already a variable", n));
                        }
                        Ok(())
                    });
                });
                ui.push_id(("value", i), |ui| {
                    let cx = EditCx {
                        visible: &visible,
                        preview: scope.as_ref(),
                        assets,
                    };
                    changed |= num_edit(ui, value, &cx);
                });
                if ui.small_button("✖").on_hover_text("Remove").clicked() {
                    remove = Some(i);
                }
                ui.end_row();

                visible.insert(name.clone());
                scope = scope.and_then(|s| {
                    let v = value.eval_scope(&s).ok()?;
                    Some(s.with(name, v))
                });
            }
        });

    if let Some(i) = remove {
        vars.0.remove(i);
        changed = true;
    }
    if ui.button("+ Add variable").clicked() {
        let name = ('a'..='z')
            .map(|c| c.to_string())
            .chain((1..).map(|n| format!("v{}", n)))
            .find(|n| !names.contains(n) && check_name(n).is_ok())
            .expect("an unbounded list has a free name");
        vars.0.push((name, Num::Lit(0.)));
        changed = true;
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parse_scene;

    #[test]
    fn scope_includes_enclosing_loops_but_not_own() {
        let scene = parse_scene(
            "variables:
  n: 3
camera:
  look_from: 0, 0, -10
world:
  - repeat:
      variable: i
      from: n
      to: 10
      world:
        - repeat:
            variable: j
            to: 2
",
        )
        .unwrap();
        let (visible, preview) = scope_at(&scene, &Selection::Hittable(vec![0, 0]), 2);
        assert!(visible.contains("i") && visible.contains("n") && visible.contains("frameIndex"));
        assert!(!visible.contains("j"));
        let preview = preview.unwrap();
        assert_eq!(Some(3.), preview.get("i"));
        assert_eq!(Some(2.), preview.get("frameIndex"));

        let (visible, _) = scope_at(&scene, &Selection::Camera, 0);
        assert!(!visible.contains("i"));
    }
}
