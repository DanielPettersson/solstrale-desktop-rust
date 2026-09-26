//! Drives the outline and inspector headless. Renders are written to
//! target/editor-previews for looking at, not compared.

use eframe::egui::{self, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use crate::document::Document;
use crate::editor::asset_picker::AssetPicker;
use crate::editor::inspector::inspector;
use crate::editor::outline::{OutlineCx, Selection, apply, outline};
use crate::model::hittable::Hittable;
use crate::model::parse_scene;
use crate::model::scene::Scene;

struct State {
    scene: Scene,
    selection: Selection,
    assets: AssetPicker,
    changed: bool,
}

fn editor_ui(ui: &mut egui::Ui, s: &mut State) {
    ui.horizontal_top(|ui| {
        ui.allocate_ui(Vec2::new(260., ui.available_height()), |ui| {
            ui.vertical(|ui| {
                let vars = Default::default();
                let mut cx = OutlineCx {
                    selection: &mut s.selection,
                    error: None,
                    scene_variables: &vars,
                    ops: Vec::new(),
                };
                outline(ui, &s.scene, &mut cx);
                for op in cx.ops {
                    if let Some(sel) = apply(&mut s.scene.world, op) {
                        s.selection = sel;
                    }
                    s.changed = true;
                }
            });
        });
        ui.separator();
        ui.vertical(|ui| {
            s.changed |= inspector(ui, &mut s.scene, &s.selection, 0, &mut s.assets);
        });
    });
}

fn harness(scene: Scene, selection: Selection) -> Harness<'static, State> {
    Harness::builder()
        .with_size(Vec2::new(760., 640.))
        .wgpu()
        .build_ui_state(
            editor_ui,
            State {
                scene,
                selection,
                assets: AssetPicker::default(),
                changed: false,
            },
        )
}

fn save_preview(h: &mut Harness<State>, name: &str) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/editor-previews");
    std::fs::create_dir_all(&dir).unwrap();
    h.render()
        .unwrap()
        .save(dir.join(format!("{}.png", name)))
        .unwrap();
}

const EXPRESSIONS_SCENE: &str = "variables:
  spacing: 2.5
  wobble: sin(frameIndex * 0.1)
camera:
  look_from: 0, 1, -10
  vertical_fov_degrees: 40 + wobble
world:
  - repeat:
      variable: i
      from: -2
      to: 3
      world:
        - sphere:
            center: i * spacing, wobble, 0
            radius: 0.5
            material:
              metal:
                fuzz: 0.1
                albedo:
                  color: 0.9, 0.5, 0.2
            transformations:
              - rotation_y: i * 10
              - scale: 1.5
";

#[test]
fn every_selection_renders() {
    let default = Document::default().scene;
    let expressions = parse_scene(EXPRESSIONS_SCENE).unwrap();
    for (name, scene, selection) in [
        ("scene", expressions.clone(), Selection::Scene),
        ("camera", expressions.clone(), Selection::Camera),
        ("render_config", default.clone(), Selection::RenderConfig),
        ("repeat", expressions.clone(), Selection::Hittable(vec![0])),
        (
            "sphere_in_repeat",
            expressions.clone(),
            Selection::Hittable(vec![0, 0]),
        ),
        ("quad", default.clone(), Selection::Hittable(vec![0])),
        ("box", default.clone(), Selection::Hittable(vec![6])),
    ] {
        let mut h = harness(scene, selection);
        h.run();
        save_preview(&mut h, name);
        assert!(
            !h.state().changed,
            "{} changed the scene just by showing it",
            name
        );
    }
}

#[test]
fn clicking_the_outline_selects() {
    let mut h = harness(Document::default().scene, Selection::Scene);
    h.run();
    h.get_by_label("Camera").click();
    h.run();
    assert_eq!(Selection::Camera, h.state().selection);

    h.get_by_label("Render configuration").click();
    h.run();
    assert_eq!(Selection::RenderConfig, h.state().selection);
}

fn text_field(value: &str) -> impl Fn(&egui_kittest::kittest::AccessKitNode<'_>) -> bool + '_ {
    move |n| n.role() == egui::accesskit::Role::TextInput && n.value().as_deref() == Some(value)
}

fn center_x(s: &State) -> String {
    match crate::editor::outline::get(&s.scene.world, &[0, 0]) {
        Some(Hittable::Sphere(sphere)) => sphere.center.x.to_string(),
        other => panic!("{:?}", other),
    }
}

#[test]
fn typing_an_expression_updates_the_scene_only_when_valid() {
    let mut h = harness(
        parse_scene(EXPRESSIONS_SCENE).unwrap(),
        Selection::Hittable(vec![0, 0]),
    );
    h.run();

    let field = h.get_by(text_field("i * spacing"));
    field.focus();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    field.type_text("i * 3");
    h.run();
    assert_eq!("i * 3", center_x(h.state()));
    assert!(h.state().changed);

    // `q` is not a variable here, so the scene keeps the last valid value
    let field = h.get_by(text_field("i * 3"));
    field.type_text(" * q");
    h.run();
    assert_eq!("i * 3", center_x(h.state()));
    save_preview(&mut h, "invalid_expression");
}

#[test]
fn adding_from_the_world_menu() {
    let mut h = harness(Document::default().scene, Selection::Scene);
    h.run();
    let before = h.state().scene.world.len();
    h.get_by_label("+").click();
    h.run();
    h.get_by_label("Sphere").click();
    h.run();
    let s = h.state();
    assert_eq!(before + 1, s.scene.world.len());
    assert!(matches!(s.scene.world.last(), Some(Hittable::Sphere(_))));
    assert_eq!(Selection::Hittable(vec![before]), s.selection);
    assert!(s.changed);
    save_preview(&mut h, "added_sphere");
}
