//! Drives edits through to a real render, as the app's frame loop does, but
//! without a window.

use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{Context, Vec2};
use eframe::wgpu;
use solstrale::util::wgpu_util::get_wgpu_device_and_queue;

use crate::model::hittable::Hittable;
use crate::model::lambertian::Lambertian;
use crate::model::material::Material;
use crate::model::num::Num;
use crate::model::parse_scene;
use crate::model::pos::Pos;
use crate::model::rgb::Rgb;
use crate::model::scene::Scene;
use crate::model::sphere::Sphere;
use crate::model::texture::Texture;
use crate::model::transformation::Transformation;
use crate::render_output::{create_render_resources, dispatch};
use crate::{RenderControl, RenderResources, RenderedImage};

struct App {
    rc: RenderControl,
    image: RenderedImage,
    scene: Scene,
    viewport: Vec2,
    ctx: Context,
    resources: Arc<RenderResources>,
}

impl App {
    fn new(scene: Scene, viewport: Vec2) -> App {
        let (device, queue) = get_wgpu_device_and_queue();
        App::on(device, queue, scene, viewport)
    }

    fn on(device: &wgpu::Device, queue: &wgpu::Queue, scene: Scene, viewport: Vec2) -> App {
        let resources = Arc::new(create_render_resources(
            device,
            queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ));
        let mut app = App {
            rc: RenderControl {
                render_requested: true,
                ..Default::default()
            },
            image: RenderedImage::default(),
            scene,
            viewport,
            ctx: Context::default(),
            resources,
        };
        app.until("the first frame", |app| app.image.progress > 0.);
        app
    }

    /// One frame of the app
    fn frame(&mut self) {
        self.rc.receive(&mut self.image);
        let screen = (self.viewport.x as usize, self.viewport.y as usize);
        self.rc.schedule(&self.scene, 0, screen);
        dispatch(
            &mut self.rc,
            &self.scene,
            0,
            self.viewport,
            &self.ctx,
            &self.resources,
        );
    }

    fn until(&mut self, what: &str, done: impl Fn(&App) -> bool) {
        let start = Instant::now();
        while !done(self) {
            assert!(
                start.elapsed() < Duration::from_secs(60),
                "no {}: building {}, running {}, progress {}, error {:?}",
                what,
                self.rc.build.is_some(),
                self.rc.update_sender.is_some(),
                self.image.progress,
                self.rc.render_error
            );
            std::thread::sleep(Duration::from_micros(500));
            self.frame();
        }
    }

    fn edit(&mut self, edit: impl FnOnce(&mut Scene)) {
        self.rc.edit_timer.shown_after = None;
        edit(&mut self.scene);
        self.rc.edited(Instant::now());
        self.frame();
    }

    /// Until the edits are built and shown
    fn shown(&mut self) -> Duration {
        self.until("frame with the edit", |app| {
            app.rc.edit_timer.shown_after.is_some()
        });
        self.until("end of the build", |app| app.rc.build.is_none());
        self.rc.edit_timer.shown_after.unwrap()
    }
}

const SCENE: &str = "camera:
  look_from: 0, 0, -10
world:
  - sphere:
      center: 0, 0, 0
      radius: 1
      material:
        light: {}
";

#[test]
fn an_image_of_another_size_than_the_viewport_is_shown_at_its_own() {
    let scene = format!(
        "render_configuration:\n  samples_per_pixel: 4\n  width_height:\n    half_screen: {{}}\n{}",
        SCENE
    );
    let app = App::new(parse_scene(&scene).unwrap(), Vec2::new(64., 48.));
    assert_eq!((32, 24), (app.image.width, app.image.height));
}

#[test]
fn an_edit_updates_the_running_render() {
    let mut app = App::new(parse_scene(SCENE).unwrap(), Vec2::new(32., 24.));
    let buffer = app.image.output_buffer.clone();

    app.edit(|s| s.background_color = Some(Rgb::new(0.2, 0.2, 0.2)));
    app.shown();
    app.edit(|s| {
        if let Hittable::Sphere(sphere) = &mut s.world[0] {
            sphere.radius = Num::Lit(2.);
        }
    });
    app.shown();

    assert!(!app.rc.render_requested && !app.rc.loading_scene);
    assert_eq!(
        buffer, app.image.output_buffer,
        "a new render has a new buffer"
    );
    assert!(app.rc.render_error.is_none());
}

#[test]
fn an_error_leaves_the_render_running() {
    let mut app = App::new(parse_scene(SCENE).unwrap(), Vec2::new(32., 24.));
    let buffer = app.image.output_buffer.clone();

    app.edit(|s| {
        if let Hittable::Sphere(sphere) = &mut s.world[0] {
            sphere.radius = Num::parse("nope").unwrap();
        }
    });
    app.until("build", |app| app.rc.build.is_none());
    let error = app.rc.render_error.as_ref().expect("an error");
    assert_eq!(
        Some(crate::editor::outline::Selection::Hittable(vec![0])),
        error.selection
    );
    assert!(app.rc.update_sender.is_some());

    app.edit(|s| {
        if let Hittable::Sphere(sphere) = &mut s.world[0] {
            sphere.radius = Num::Lit(1.5);
        }
    });
    app.shown();
    assert!(app.rc.render_error.is_none());
    assert_eq!(buffer, app.image.output_buffer);
}

#[test]
fn deleting_the_last_light_leaves_the_render_running() {
    let mut app = App::new(parse_scene(SCENE).unwrap(), Vec2::new(32., 24.));
    app.edit(|s| s.world.clear());
    app.shown();
    assert!(app.rc.render_error.is_none());
    assert!(app.rc.update_sender.is_some());
}

/// A device on the fastest adapter, as the app gets
fn fast_device() -> (wgpu::Device, wgpu::Queue) {
    pollster::block_on(async {
        let adapter = wgpu::Instance::default()
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .unwrap();
        adapter
            .request_device(&crate::device_descriptor(&adapter))
            .await
            .unwrap()
    })
}

fn median(mut times: Vec<Duration>) -> Duration {
    times.sort();
    times[times.len() / 2]
}

/// Time from an edit to the first frame it shows in, on the scene in
/// `SOLSTRALE_EDIT_SCENE`, e.g. sponza, which needs a model and a light.
/// Run with `cargo test --lib edit_latency -- --ignored --nocapture`.
///
/// Frames are polled for every 0.5 ms, rather than painted on vsync as in the
/// app.
#[test]
#[ignore]
fn edit_latency() {
    let path = std::env::var("SOLSTRALE_EDIT_SCENE").expect("SOLSTRALE_EDIT_SCENE");
    let mut scene = parse_scene(&std::fs::read_to_string(path).unwrap()).unwrap();
    let camera = scene.camera_at(0).unwrap();
    scene.world.push(Hittable::Sphere(Sphere {
        center: Pos::new(camera.look_at.x, camera.look_at.y, camera.look_at.z),
        radius: Num::Lit(10.),
        material: None,
        transformations: Vec::new(),
    }));
    let sphere = scene.world.len() - 1;
    let model = scene
        .world
        .iter()
        .position(|h| matches!(h, Hittable::Model(_)))
        .expect("a model");

    let (device, queue) = fast_device();
    let start = Instant::now();
    let mut app = App::on(&device, &queue, scene, Vec2::new(800., 600.));
    println!("first frame after {:.1} ms", ms(start.elapsed()));

    type Edit = fn(&mut Scene, usize, usize, f64);
    let edits: [(&str, Edit); 4] = [
        ("camera", |s, _, _, v| {
            s.camera.look_at = Some(Pos::new(280. + v, 130., 80.))
        }),
        ("background", |s, _, _, v| {
            s.background_color = Some(Rgb::new(v, v, v))
        }),
        ("sphere albedo", |s, sphere, _, v| {
            if let Hittable::Sphere(sphere) = &mut s.world[sphere] {
                sphere.material = Some(Material::Lambertian(Lambertian {
                    albedo: Some(Texture::Color(Rgb::new(v, 0.5, 0.5))),
                    normal: None,
                }));
            }
        }),
        ("model translation", |s, _, model, v| {
            if let Hittable::Model(model) = &mut s.world[model] {
                model.transformations =
                    vec![Transformation::Translation(Pos::new(v * 10., 0., 0.))];
            }
        }),
    ];
    // Taken in turns, so a change in the GPU's clocks falls on every kind
    let mut times = vec![Vec::new(); edits.len()];
    for i in 0..20 {
        for (k, (_, edit)) in edits.iter().enumerate() {
            // Mid accumulation, as an edit usually finds a render
            std::thread::sleep(Duration::from_millis(50));
            app.frame();
            app.edit(|s| edit(s, sphere, model, 0.5 + i as f64 * 0.01));
            times[k].push(app.shown());
        }
    }
    for ((name, _), times) in edits.iter().zip(times) {
        println!("{}: median {:.1} ms", name, ms(median(times)));
    }

    let times = (0..5)
        .map(|_| {
            let start = Instant::now();
            app.rc.render_requested = true;
            app.until("the new render's first frame", |app| {
                !app.rc.render_requested && !app.rc.loading_scene
            });
            start.elapsed()
        })
        .collect();
    println!("a new render: median {:.1} ms", ms(median(times)));
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.
}
