//! Decides how edits to the scene reach the running render: which parts of the
//! scene they changed, and building those parts into an update to it.

use std::error::Error;

use solstrale::renderer::SceneUpdate;

use crate::editor::outline::Selection;
use crate::model::CreatorContext;
use crate::model::camera_config::CameraConfig;
use crate::model::hittable::Hittable;
use crate::model::num::VisitNums;
use crate::model::post_processor::PostProcessor;
use crate::model::render_config::RenderConfig;
use crate::model::rgb::Rgb;
use crate::model::scene::Scene;
use crate::model::scope::Scope;

/// The parts of a scene that a running render takes one by one
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Parts {
    pub camera: bool,
    pub background_color: bool,
    pub render_config: bool,
    pub post_processors: bool,
    pub world: bool,
}

impl Parts {
    pub const NONE: Parts = Parts {
        camera: false,
        background_color: false,
        render_config: false,
        post_processors: false,
        world: false,
    };
    pub const ALL: Parts = Parts {
        camera: true,
        background_color: true,
        render_config: true,
        post_processors: true,
        world: true,
    };
    pub const CAMERA: Parts = Parts {
        camera: true,
        ..Parts::NONE
    };

    pub fn is_empty(&self) -> bool {
        *self == Parts::NONE
    }
}

/// A part of the model, and the values of the variables it reads
#[derive(Clone, Debug, PartialEq)]
struct Input<T> {
    model: T,
    reads: String,
}

impl<T: VisitNums + Clone> Input<T> {
    fn new(model: &T, scope: &Scope) -> Input<T> {
        Input {
            model: model.clone(),
            reads: scope.fingerprint(&model.free_vars()),
        }
    }
}

/// What each part of a scene is evaluated from. A part whose input is the same
/// evaluates the same, so only the parts whose inputs changed are built again.
#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    camera: Input<CameraConfig>,
    background_color: Input<Option<Rgb>>,
    /// Less its post-processors, and with the screen size a size can be
    /// relative to
    render_config: (Input<Option<RenderConfig>>, (usize, usize)),
    post_processors: Input<Vec<PostProcessor>>,
    world: Input<Vec<Hittable>>,
}

impl Inputs {
    pub fn of(
        scene: &Scene,
        frame: usize,
        screen: (usize, usize),
    ) -> Result<Inputs, Box<dyn Error>> {
        let scope = scene.scope(&Scope::builtin(frame))?;
        let config = scene.render_configuration.as_ref();
        let config_alone = config.map(|c| RenderConfig {
            post_processors: Vec::new(),
            ..c.clone()
        });
        let post_processors = config.map_or(Vec::new(), |c| c.post_processors.clone());
        Ok(Inputs {
            camera: Input::new(&scene.camera, &scope),
            background_color: Input::new(&scene.background_color, &scope),
            render_config: (Input::new(&config_alone, &scope), screen),
            post_processors: Input::new(&post_processors, &scope),
            world: Input::new(&scene.world, &scope),
        })
    }

    /// The parts whose inputs differ
    pub fn changed(&self, other: &Inputs) -> Parts {
        Parts {
            camera: self.camera != other.camera,
            background_color: self.background_color != other.background_color,
            render_config: self.render_config != other.render_config,
            post_processors: self.post_processors != other.post_processors,
            world: self.world != other.world,
        }
    }

    /// Takes `parts` from `other`
    pub fn take(&mut self, other: &Inputs, parts: Parts) {
        if parts.camera {
            self.camera = other.camera.clone();
        }
        if parts.background_color {
            self.background_color = other.background_color.clone();
        }
        if parts.render_config {
            self.render_config = other.render_config.clone();
        }
        if parts.post_processors {
            self.post_processors = other.post_processors.clone();
        }
        if parts.world {
            self.world = other.world.clone();
        }
    }

    /// Takes the scene's camera
    pub fn take_camera(&mut self, scene: &Scene, frame: usize) -> Result<(), Box<dyn Error>> {
        self.camera = Input::new(&scene.camera, &scene.scope(&Scope::builtin(frame))?);
        Ok(())
    }
}

/// Builds `parts` of the scene into an update, with `ctx` giving the builtin
/// scope. A part that fails is left out. Returns the update, the parts in it
/// and the first error.
///
/// The camera reaches the render through the view instead, see
/// [`crate::RenderControl::schedule`], so it is only checked here.
pub fn create_update(
    scene: &Scene,
    ctx: &CreatorContext,
    parts: Parts,
) -> (SceneUpdate, Parts, Option<Box<dyn Error>>) {
    let mut update = SceneUpdate::default();
    let mut built = Parts::NONE;
    let scope = match scene.scope(ctx.scope) {
        Ok(scope) => scope,
        Err(err) => return (update, built, Some(err)),
    };
    let ctx = &CreatorContext {
        scope: &scope,
        ..*ctx
    };

    let mut error = None;
    fn part<T>(
        wanted: bool,
        create: impl FnOnce() -> Result<T, Box<dyn Error>>,
        built: &mut bool,
        error: &mut Option<Box<dyn Error>>,
    ) -> Option<T> {
        if !wanted {
            return None;
        }
        match create() {
            Ok(v) => {
                *built = true;
                Some(v)
            }
            Err(e) => {
                error.get_or_insert(e);
                None
            }
        }
    }

    if parts.camera
        && let Err(e) = scene.create_camera(ctx)
    {
        error = Some(e);
    }
    update.background_color = part(
        parts.background_color,
        || scene.create_background_color(ctx),
        &mut built.background_color,
        &mut error,
    );
    update.render_config = part(
        parts.render_config,
        || scene.create_render_config(ctx),
        &mut built.render_config,
        &mut error,
    );
    update.post_processors = part(
        parts.post_processors,
        || scene.create_post_processors(ctx),
        &mut built.post_processors,
        &mut error,
    );
    update.world = part(
        parts.world,
        || scene.create_world(ctx),
        &mut built.world,
        &mut error,
    );
    (update, built, error)
}

/// The part of the scene an error path points to, e.g.
/// `["world[1]", "repeat (i = 2)", "world[0]", "sphere"]` is the first
/// hittable in the repeat that is the second in the world.
pub fn error_selection(path: &[String]) -> Option<Selection> {
    match path.first().map(String::as_str) {
        Some("camera") => return Some(Selection::Camera),
        Some("render_configuration") => return Some(Selection::RenderConfig),
        Some("variables") | Some("background_color") => return Some(Selection::Scene),
        _ => {}
    }
    let indices: Vec<usize> = path
        .iter()
        .filter_map(|s| s.strip_prefix("world[")?.strip_suffix(']')?.parse().ok())
        .collect();
    (!indices.is_empty()).then_some(Selection::Hittable(indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::model::num::Num;
    use crate::model::parse_scene;
    use crate::model::pos::Pos;
    use crate::{Build, Built, RenderControl, RenderError};
    use solstrale::hittable::Hittable as _;
    use solstrale::util::wgpu_util::get_wgpu_device_and_queue;
    use std::sync::mpsc::channel;
    use std::time::Instant;

    const SCREEN: (usize, usize) = (400, 300);

    fn changed(a: &Scene, b: &Scene) -> Parts {
        changed_at(a, 0, b, 0)
    }

    fn changed_at(a: &Scene, frame_a: usize, b: &Scene, frame_b: usize) -> Parts {
        Inputs::of(a, frame_a, SCREEN)
            .unwrap()
            .changed(&Inputs::of(b, frame_b, SCREEN).unwrap())
    }

    fn only(f: impl FnOnce(&mut Parts)) -> Parts {
        let mut parts = Parts::NONE;
        f(&mut parts);
        parts
    }

    #[test]
    fn each_part_is_told_apart() {
        let scene = Document::default().scene;
        assert_eq!(Parts::NONE, changed(&scene, &scene));

        let mut s = scene.clone();
        s.camera.look_from = Pos::new(1., 2., 3.);
        assert_eq!(only(|p| p.camera = true), changed(&scene, &s));

        let mut s = scene.clone();
        s.background_color = Some(Rgb::new(0.1, 0.1, 0.1));
        assert_eq!(only(|p| p.background_color = true), changed(&scene, &s));

        let mut s = scene.clone();
        s.render_configuration.as_mut().unwrap().samples_per_pixel = Some(10);
        assert_eq!(only(|p| p.render_config = true), changed(&scene, &s));

        let mut s = scene.clone();
        s.render_configuration
            .as_mut()
            .unwrap()
            .post_processors
            .pop();
        assert_eq!(only(|p| p.post_processors = true), changed(&scene, &s));

        let mut s = scene.clone();
        s.world.pop();
        assert_eq!(only(|p| p.world = true), changed(&scene, &s));

        let inputs = Inputs::of(&scene, 0, SCREEN).unwrap();
        assert_eq!(
            only(|p| p.render_config = true),
            inputs.changed(&Inputs::of(&scene, 0, (800, 600)).unwrap())
        );
    }

    #[test]
    fn variables_and_the_frame_change_the_parts_that_read_them() {
        let scene = parse_scene(
            "variables:
  unused: 1
  x: 2
  t: frameIndex * 2
background_color: x / 10, 0, 0
camera:
  look_from: t, 0, -10
world:
  - sphere:
      center: 0, 0, 0
      radius: x
",
        )
        .unwrap();
        let mut unused = scene.clone();
        unused.variables.0[0].1 = Num::Lit(5.);
        assert_eq!(Parts::NONE, changed(&scene, &unused));

        let mut x = scene.clone();
        x.variables.0[1].1 = Num::Lit(3.);
        assert_eq!(
            only(|p| {
                p.background_color = true;
                p.world = true
            }),
            changed(&scene, &x)
        );

        assert_eq!(only(|p| p.camera = true), changed_at(&scene, 0, &scene, 1));
        assert!(
            Inputs::of(
                &parse_scene("variables:\n  a: b\ncamera:\n  look_from: 0, 0, 0\nworld: []\n")
                    .unwrap(),
                0,
                SCREEN
            )
            .is_err()
        );
    }

    #[test]
    fn take_copies_only_the_parts_asked_for() {
        let scene = Document::default().scene;
        let mut edited = scene.clone();
        edited.world.pop();
        edited.background_color = Some(Rgb::new(0.1, 0.1, 0.1));

        let mut inputs = Inputs::of(&scene, 0, SCREEN).unwrap();
        let edited_inputs = Inputs::of(&edited, 0, SCREEN).unwrap();
        inputs.take(&edited_inputs, only(|p| p.world = true));
        assert_eq!(
            only(|p| p.background_color = true),
            inputs.changed(&edited_inputs)
        );
    }

    /// A render of `scene` is running
    fn running(scene: &Scene) -> RenderControl {
        let (update_sender, _) = channel();
        RenderControl {
            update_sender: Some(update_sender),
            rendered: Some(Inputs::of(scene, 0, SCREEN).unwrap()),
            ..Default::default()
        }
    }

    fn requested_parts(rc: &mut RenderControl) -> Option<Parts> {
        rc.update_requested.take().map(|r| r.parts)
    }

    #[test]
    fn edits_go_to_the_running_render_at_once() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        rc.schedule(&scene, 0, SCREEN);
        assert!(rc.update_requested.is_none());

        let mut edited = scene.clone();
        edited.world.pop();
        rc.edited(Instant::now());
        rc.schedule(&edited, 0, SCREEN);
        assert_eq!(Some(only(|p| p.world = true)), requested_parts(&mut rc));
        assert!(!rc.render_requested);
        assert!(!rc.edit_pending);
    }

    fn build_in_flight(rc: &mut RenderControl, scene: &Scene) -> std::sync::mpsc::Sender<Built> {
        let (sender, receiver) = channel();
        rc.build = Some(Build {
            receiver,
            inputs: Some(Inputs::of(scene, 0, SCREEN).unwrap()),
            started: Instant::now(),
        });
        sender
    }

    #[test]
    fn edits_during_a_build_go_into_the_next() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        let mut first = scene.clone();
        first.world.pop();
        let built = build_in_flight(&mut rc, &first);

        let mut second = first.clone();
        second.background_color = Some(Rgb::new(0.1, 0.1, 0.1));
        rc.edited(Instant::now());
        rc.schedule(&second, 0, SCREEN);
        assert!(rc.update_requested.is_none());
        assert!(rc.edit_pending);

        built
            .send(Built {
                parts: only(|p| p.world = true),
                error: None,
            })
            .unwrap();
        rc.receive(&mut Default::default());
        assert!(rc.build.is_none());
        rc.schedule(&second, 0, SCREEN);
        assert_eq!(
            Some(only(|p| p.background_color = true)),
            requested_parts(&mut rc)
        );
    }

    #[test]
    fn a_failed_part_is_built_again_with_the_next_edit() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        let mut broken = scene.clone();
        broken.world.pop();
        broken.background_color = Some(Rgb::new(0.1, 0.1, 0.1));
        let built = build_in_flight(&mut rc, &broken);
        built
            .send(Built {
                parts: only(|p| p.background_color = true),
                error: Some(RenderError {
                    message: "broken".to_string(),
                    selection: None,
                }),
            })
            .unwrap();
        rc.receive(&mut Default::default());
        assert!(rc.render_error.is_some());

        let mut edited = broken.clone();
        edited
            .render_configuration
            .as_mut()
            .unwrap()
            .samples_per_pixel = Some(10);
        rc.edited(Instant::now());
        rc.schedule(&edited, 0, SCREEN);
        assert_eq!(
            Some(only(|p| {
                p.render_config = true;
                p.world = true
            })),
            requested_parts(&mut rc)
        );
    }

    #[test]
    fn undoing_a_failed_edit_clears_its_error() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        rc.render_error = Some(RenderError {
            message: "broken".to_string(),
            selection: None,
        });
        rc.edited(Instant::now());
        rc.schedule(&scene, 0, SCREEN);
        assert!(rc.update_requested.is_none());
        assert!(rc.render_error.is_none());
    }

    #[test]
    fn a_new_render_being_built_holds_edits() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        rc.loading_scene = true;
        let mut edited = scene.clone();
        edited.world.pop();
        rc.edited(Instant::now());
        rc.schedule(&edited, 0, SCREEN);
        assert!(rc.update_requested.is_none());
        assert!(rc.edit_pending);

        rc.loading_scene = false;
        rc.schedule(&edited, 0, SCREEN);
        assert_eq!(Some(only(|p| p.world = true)), requested_parts(&mut rc));
    }

    #[test]
    fn edits_without_a_render_start_one() {
        let scene = Document::default().scene;
        let mut rc = RenderControl::default();
        rc.edited(Instant::now());
        rc.schedule(&scene, 0, SCREEN);
        assert!(rc.render_requested);
        assert!(rc.update_requested.is_none());
    }

    #[test]
    fn a_render_that_ended_is_not_updated() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        let (sender, receiver) = channel();
        rc.render_receiver = Some(receiver);
        sender
            .send(crate::RenderMessage::Error(RenderError {
                message: "no lights".to_string(),
                selection: None,
            }))
            .unwrap();
        rc.receive(&mut Default::default());
        assert!(rc.update_sender.is_none());
        assert!(rc.render_error.is_some());

        let mut edited = scene.clone();
        edited.world.pop();
        rc.edited(Instant::now());
        rc.schedule(&edited, 0, SCREEN);
        assert!(rc.render_requested);
    }

    #[test]
    fn camera_edits_go_to_the_running_render_through_the_view() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        // Not held back by a build
        let _built = build_in_flight(&mut rc, &scene);

        let mut moved = scene.clone();
        moved.camera.look_from = Pos::new(1., 2., 3.);
        rc.edited(Instant::now());
        rc.schedule(&moved, 0, SCREEN);
        assert!(rc.update_requested.is_none());
        assert!(rc.camera_updated);
        assert_eq!(
            solstrale::geo::vec3::Vec3::new(1., 2., 3.),
            rc.applied_camera.unwrap().look_from
        );
        assert_eq!(
            Parts::NONE,
            rc.rendered
                .as_ref()
                .unwrap()
                .changed(&Inputs::of(&moved, 0, SCREEN).unwrap())
        );
    }

    #[test]
    fn a_camera_that_fails_goes_to_the_build_to_report() {
        let scene = Document::default().scene;
        let mut rc = running(&scene);
        let mut broken = scene.clone();
        broken.camera.look_from.x = Num::parse("nope").unwrap();
        rc.edited(Instant::now());
        rc.schedule(&broken, 0, SCREEN);
        assert!(!rc.camera_updated);
        assert_eq!(Some(Parts::CAMERA), requested_parts(&mut rc));
    }

    /// A view moved to `look_from`, with the rest from the scene's camera
    fn running_with_view(scene: &Scene, look_from: (f64, f64, f64)) -> RenderControl {
        let mut view = scene.camera_at(0).unwrap();
        view.look_from = solstrale::geo::vec3::Vec3::new(look_from.0, look_from.1, look_from.2);
        view.look_at = solstrale::geo::vec3::Vec3::new(0., 0., 0.);
        RenderControl {
            orbit_camera: Some(crate::model::orbit_camera::OrbitCamera::from_config(
                &view, 1.,
            )),
            ..running(scene)
        }
    }

    #[test]
    fn dragged_view_is_written_to_the_camera_without_a_restart() {
        let mut scene = Document::default().scene;
        let mut rc = running_with_view(&scene, (1.23456, 2., -10.));

        assert!(rc.write_view_to_camera(&mut scene, 0, false));
        assert_eq!(Pos::new(1.235, 2., -10.), scene.camera.look_from);
        assert_eq!(Some(Pos::new(0., 0., 0.)), scene.camera.look_at);
        // Already on screen, so nothing to render and the view has not moved
        assert_eq!(
            Parts::NONE,
            rc.rendered
                .as_ref()
                .unwrap()
                .changed(&Inputs::of(&scene, 0, SCREEN).unwrap())
        );
        assert!(!rc.view_moved());
    }

    #[test]
    fn dragging_leaves_camera_expressions_alone() {
        let mut scene = Document::default().scene;
        scene.camera.look_from.x = Num::parse("10 * 2").unwrap();
        let mut rc = running_with_view(&scene, (1., 2., -10.));

        assert!(!rc.write_view_to_camera(&mut scene, 0, false));
        assert_eq!("10 * 2", scene.camera.look_from.x.to_string());
        assert!(rc.write_view_to_camera(&mut scene, 0, true));
        assert_eq!(Pos::new(1., 2., -10.), scene.camera.look_from);
    }

    fn build(yaml: &str, parts: Parts) -> (SceneUpdate, Parts, Option<String>) {
        let scene = parse_scene(yaml).unwrap();
        let (device, queue) = get_wgpu_device_and_queue();
        let (update, built, error) = create_update(
            &scene,
            &CreatorContext {
                screen_width: SCREEN.0,
                screen_height: SCREEN.1,
                device,
                queue,
                scope: &Scope::builtin(0),
                refit_models: true,
            },
            parts,
        );
        (update, built, error.map(|e| e.to_string()))
    }

    const UPDATED: &str = "variables:
  r: 2
background_color: 0.5, 0, 0
render_configuration:
  samples_per_pixel: 7
  width_height:
    half_screen: {}
  post_processors:
    - bloom: {}
camera:
  look_from: 0, 0, -10
world:
  - sphere:
      center: 0, 0, 0
      radius: r
";

    #[test]
    fn an_update_holds_the_parts_asked_for() {
        let (update, built, error) = build(UPDATED, Parts::ALL);
        assert_eq!(None, error);
        assert_eq!(
            Parts {
                camera: false,
                ..Parts::ALL
            },
            built
        );
        assert!(update.camera.is_none(), "the camera goes through the view");
        assert_eq!(
            Some(solstrale::geo::vec3::Vec3::new(0.5, 0., 0.)),
            update.background_color
        );
        let config = update.render_config.unwrap();
        assert_eq!(
            (7, 200, 150),
            (config.samples_per_pixel, config.width, config.height)
        );
        assert_eq!(1, update.post_processors.unwrap().len());
        assert_eq!(2., update.world.unwrap().bounding_box().x.max);

        let (update, built, _) = build(UPDATED, only(|p| p.background_color = true));
        assert_eq!(only(|p| p.background_color = true), built);
        assert!(update.world.is_none() && update.render_config.is_none());
    }

    #[test]
    fn a_part_that_fails_is_left_out() {
        let yaml = UPDATED.replace("radius: r", "radius: q");
        let (update, built, error) = build(&yaml, Parts::ALL);
        assert_eq!(
            Some("world[0] › sphere: unknown variable `q` in `q`".to_string()),
            error
        );
        assert!(!built.world && update.world.is_none());
        assert!(built.background_color && update.background_color.is_some());

        let yaml = UPDATED.replace("look_from: 0, 0, -10", "look_from: q, 0, -10");
        let (_, built, error) = build(&yaml, Parts::CAMERA);
        assert!(error.unwrap().starts_with("camera:"));
        assert_eq!(Parts::NONE, built);

        let yaml = UPDATED.replace("r: 2", "r: q");
        let (_, built, error) = build(&yaml, Parts::ALL);
        assert!(error.unwrap().starts_with("variables › r:"));
        assert_eq!(Parts::NONE, built);
    }

    #[test]
    fn error_paths_point_into_the_scene() {
        let path = |p: &[&str]| p.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            Some(Selection::Hittable(vec![1, 0])),
            error_selection(&path(&["world[1]", "repeat (i = 2)", "world[0]", "sphere"]))
        );
        assert_eq!(Some(Selection::Camera), error_selection(&path(&["camera"])));
        assert_eq!(
            Some(Selection::Scene),
            error_selection(&path(&["variables", "a"]))
        );
        assert_eq!(None, error_selection(&[]));
    }
}
