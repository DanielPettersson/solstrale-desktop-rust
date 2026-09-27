//! Tests of scenes using expressions, variables and repeat, built on a GPU device.

use solstrale::hittable::{Hittable, Hittables};
use solstrale::util::wgpu_util::get_wgpu_device_and_queue;

use crate::model::scope::Scope;
use crate::model::{Creator, CreatorContext, parse_scene};

const CAMERA: &str = "camera:\n  look_from: 0, 0, -10\n";

fn ctx_with<T>(scope: &Scope, f: impl FnOnce(&CreatorContext) -> T) -> T {
    let (device, queue) = get_wgpu_device_and_queue();
    f(&CreatorContext {
        screen_width: 100,
        screen_height: 100,
        device,
        queue,
        scope,
        refit_models: false,
    })
}

fn world(yaml: &str, frame: usize) -> Result<Vec<Hittables>, String> {
    let scene = parse_scene(&format!("{}{}", CAMERA, yaml)).map_err(|e| e.to_string())?;
    let scope = scene
        .scope(&Scope::builtin(frame))
        .map_err(|e| e.to_string())?;
    ctx_with(&scope, |ctx| scene.create_hittables(ctx)).map_err(|e| e.to_string())
}

fn full_scene_error(yaml: &str) -> String {
    let yaml = if yaml.contains("camera:") {
        yaml.to_string()
    } else {
        format!("{}{}", CAMERA, yaml)
    };
    let scene = parse_scene(&yaml).unwrap();
    match ctx_with(&Scope::builtin(0), |ctx| scene.create(ctx)) {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

fn centers_x(world: &[Hittables]) -> Vec<f64> {
    world
        .iter()
        .map(|h| {
            let b = h.bounding_box();
            (b.x.min + b.x.max) / 2.
        })
        .collect()
}

#[test]
fn repeat_expands_with_loop_variable() {
    let w = world(
        "world:
  - repeat:
      variable: i
      to: 3
      world:
        - sphere:
            center: i * 2, 0, 0
            radius: 0.5
",
        0,
    )
    .unwrap();
    assert_eq!(vec![0., 2., 4.], centers_x(&w));
}

#[test]
fn nested_repeats_see_outer_variables() {
    let w = world(
        "variables:
  n: 2
world:
  - repeat:
      variable: i
      to: n
      world:
        - repeat:
            variable: j
            from: i
            to: 3
            world:
              - sphere:
                  center: i * 10 + j, 0, 0
                  radius: 0.1
",
        0,
    )
    .unwrap();
    assert_eq!(vec![0., 1., 2., 11., 12.], centers_x(&w));
}

#[test]
fn frame_index_changes_values() {
    let yaml = "world:
  - sphere:
      center: frameIndex * 2, 0, 0
      radius: 1
";
    assert_eq!(vec![0.], centers_x(&world(yaml, 0).unwrap()));
    assert_eq!(vec![6.], centers_x(&world(yaml, 3).unwrap()));
}

#[test]
fn frame_index_use_is_found_anywhere() {
    let uses = |yaml: &str| {
        parse_scene(&format!("{}{}", CAMERA, yaml))
            .unwrap()
            .uses_frame_index()
    };
    assert!(!uses(
        "world:\n  - sphere:\n      center: 0, 0, 0\n      radius: 1\n"
    ));
    assert!(uses("variables:\n  t: frameIndex / 10\nworld: []\n"));
    assert!(uses(
        "world:\n  - repeat:\n      variable: i\n      to: frameIndex\n"
    ));
    assert!(uses(
        "world:\n  - sphere:\n      center: 0, 0, 0\n      radius: 1\n      material:\n        light:\n          color: frameIndex, 1, 1\n"
    ));
}

#[test]
fn transformations_and_materials_take_expressions() {
    let w = world(
        "variables:
  s: 2
world:
  - sphere:
      center: 0, 0, 0
      radius: 1
      material:
        metal:
          fuzz: s / 10
          albedo:
            color: s / 2, 0, 0
      transformations:
        - scale: s
        - translation: s * 5, 0, 0
",
        0,
    )
    .unwrap();
    let b = w[0].bounding_box();
    assert_eq!((8., 12.), (b.x.min, b.x.max));
}

#[test]
fn loop_variable_can_not_shadow() {
    let err = world(
        "variables:
  i: 1
world:
  - repeat:
      variable: i
      to: 3
",
        0,
    )
    .unwrap_err();
    assert!(err.contains("`i` is already defined"), "{}", err);
}

#[test]
fn errors_name_the_path_and_iteration() {
    let err = full_scene_error(
        "world:
  - sphere:
      center: 0, 0, 0
      radius: 1
  - repeat:
      variable: i
      to: 2
      world:
        - sphere:
            center: 0, 0, 0
            radius: 1 / (1 - i)
",
    );
    assert_eq!(
        "world[1] › repeat (i = 1) › world[0] › sphere: result inf is not a finite number in `1 / (1 - i)`",
        err
    );

    let err = full_scene_error(
        "world:
  - sphere:
      center: 0, 0, 0
      radius: j
",
    );
    assert_eq!("world[0] › sphere: unknown variable `j` in `j`", err);

    let err =
        full_scene_error("camera:\n  look_from: 0, 0, -10\n  vertical_fov_degrees: x\nworld: []\n");
    assert!(err.starts_with("camera: unknown variable `x`"), "{}", err);
}

#[test]
fn colors_can_not_be_negative() {
    let err = full_scene_error(
        "variables:
  t: -2
world:
  - sphere:
      center: 0, 0, 0
      radius: 1
      material:
        lambertian:
          albedo:
            color: 1, t * 0.25, 1
",
    );
    assert_eq!(
        "world[0] › sphere: a color can not be negative, but g is -0.5",
        err
    );
}

#[test]
fn obj_models_are_cached_per_evaluated_value() {
    let dir = std::env::temp_dir().join("solstrale_desktop_scene_tests");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tri.obj"), "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
    let w = world(
        &format!(
            "world:
  - repeat:
      variable: i
      to: 3
      world:
        - model:
            path: {}/
            name: tri.obj
            transformations:
              - translation: i * 10, 0, 0
",
            dir.display()
        ),
        0,
    )
    .unwrap();
    let mins: Vec<f64> = w.iter().map(|h| h.bounding_box().x.min).collect();
    assert_eq!(3, mins.len());
    assert!(
        (mins[0] - 0.).abs() < 1e-6 && (mins[1] - 10.).abs() < 1e-6 && (mins[2] - 20.).abs() < 1e-6,
        "{:?}",
        mins
    );
}
