use crate::editor::outline::Selection;
use crate::model::ModelError;
use crate::model::orbit_camera::{CameraSnapshot, OrbitCamera};
use crate::model::pos::Pos;
use crate::model::scene::Scene;
use crate::render_scheduler::{Inputs, Parts, error_selection};
use eframe::egui::Vec2;
use eframe::wgpu;
use once_cell::sync::Lazy;
use solstrale::renderer::{RenderProgress, SceneUpdate};
use solstrale::util::tone_map::ToneMapper;
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

pub mod document;
pub mod editor;
pub mod help;
pub mod keyboard;
pub mod load_scene;
pub mod loading_output;
pub mod model;
pub mod render_button;
pub mod render_output;
pub mod render_scheduler;
#[cfg(test)]
mod render_tests;
pub mod save_image;
pub mod save_scene;

/// The tone mapping curve every display path in the app uses.
///
/// There are three of them -- the live viewport's blit shader, Save Image, and
/// the batch render binary -- and they have to agree, or the preview shows
/// something the saved file does not. Declared once here so changing the curve
/// changes all three; the viewport gets it as WGSL from
/// [`ToneMapper::wgsl`](solstrale::util::tone_map::ToneMapper::wgsl), the other
/// two hand it to `buffer_to_image`.
pub const DISPLAY_TONE_MAPPER: ToneMapper = ToneMapper::Aces;

pub static DEFAULT_SCENE: Lazy<String> =
    Lazy::new(|| include_str!("../resources/scene.yaml").to_owned());

/// The ray tracer binds twelve storage buffers in a single compute stage, while
/// wgpu's default limit is eight. This is the floor the library itself refuses
/// to start below, so it cannot be raised for headroom without turning adapters
/// the library supports into a startup failure here. The headroom is in the
/// ceiling instead: up to sixteen are requested wherever the adapter has them.
pub const MIN_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 12;

/// Ceiling on the request. Above this the count buys nothing, and asking for
/// more than an adapter has is what fails device creation.
const MAX_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 16;

/// Describes the device that both egui and the ray tracer render on.
///
/// The limits have to satisfy both: egui needs a texture large enough for a 4k+
/// surface, the ray tracer needs the raised storage buffer count. Anything the
/// adapter cannot provide fails device creation with a message naming the limit,
/// rather than panicking later when a bind group layout is created.
pub fn device_descriptor(adapter: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    let base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
        wgpu::Limits::downlevel_webgl2_defaults()
    } else {
        wgpu::Limits::default()
    };
    let adapter_limits = adapter.limits();

    wgpu::DeviceDescriptor {
        label: Some("solstrale device"),
        required_limits: wgpu::Limits {
            max_texture_dimension_2d: 8192,
            max_storage_buffers_per_shader_stage: adapter_limits
                .max_storage_buffers_per_shader_stage
                .clamp(
                    MIN_STORAGE_BUFFERS_PER_SHADER_STAGE,
                    MAX_STORAGE_BUFFERS_PER_SHADER_STAGE,
                ),
            // Scene geometry and the output buffer both outgrow the defaults
            // (128 MiB per binding, 256 MiB per buffer) on large meshes and
            // high resolutions, so take whatever the adapter offers.
            max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
            max_buffer_size: adapter_limits.max_buffer_size,
            ..base_limits
        },
        ..Default::default()
    }
}

#[derive(Default)]
pub struct ErrorInfo {
    pub show_error: bool,
    pub error_message: String,
}

impl ErrorInfo {
    pub fn handle(&mut self, err: Box<dyn Error>) {
        self.show_error = true;

        let mut err_msg = format!("{}", err);
        if let Some(s) = err.source() {
            err_msg = err_msg + &format!("\n{}", s);
        }
        self.error_message = err_msg;
    }
    pub fn handle_str(&mut self, err: &str) {
        self.show_error = true;
        self.error_message = err.to_string();
    }
}

#[derive(Default)]
pub struct RenderControl {
    pub abort_sender: Option<Sender<bool>>,
    pub render_receiver: Option<Receiver<RenderMessage>>,
    /// Takes changes to the scene into the running render
    pub update_sender: Option<Sender<SceneUpdate>>,
    /// Start a new render, building the whole scene
    pub render_requested: bool,
    /// A new render is building the whole scene, and has not produced a sample yet
    pub loading_scene: bool,
    pub initial_render_started: bool,
    pub previous_frame_render_size: Vec2,
    pub orbit_camera: Option<OrbitCamera>,
    /// The scene camera the orbit camera was last set from
    pub applied_camera: Option<CameraSnapshot>,
    /// Makes the next render move the view to the scene's camera
    pub reset_view: bool,
    pub camera_updated: bool,
    /// What each part of the running render was built from
    pub rendered: Option<Inputs>,
    /// The scene was edited, and the edit is not acted on yet
    pub edit_pending: bool,
    /// Parts of the scene to build into an update to the running render
    pub update_requested: Option<UpdateRequest>,
    /// The update being built. There is only ever one, and edits made while it
    /// builds go into the next.
    pub build: Option<Build>,
    /// Cover the viewport while the next render loads, as for a new scene
    /// rather than an edit
    pub overlay_next_render: bool,
    /// Cover the viewport while the running render loads
    pub overlay: bool,
    pub render_error: Option<RenderError>,
    /// The user moved the view in the viewport since this was last cleared
    pub view_dragged: bool,
    pub edit_timer: EditTimer,
}

/// Why the scene could not be rendered
#[derive(Debug)]
pub struct RenderError {
    pub message: String,
    /// Where in the scene the error is, when known
    pub selection: Option<Selection>,
}

impl RenderError {
    pub fn new(err: &(dyn Error + 'static)) -> RenderError {
        let mut message = format!("{}", err);
        if let Some(s) = err.source() {
            message = message + &format!("\n{}", s);
        }
        let path = err
            .downcast_ref::<ModelError>()
            .map_or(Vec::new(), |e| e.path.clone());
        RenderError {
            message,
            selection: error_selection(&path),
        }
    }
}

/// An update for [`render_output`](crate::render_output::render_output) to build
pub struct UpdateRequest {
    pub scene: Scene,
    pub frame: usize,
    pub screen: (usize, usize),
    pub parts: Parts,
    /// What the parts are built from, unless the scene's variables fail
    pub inputs: Option<Inputs>,
}

pub struct Build {
    pub receiver: Receiver<Built>,
    pub inputs: Option<Inputs>,
    pub started: Instant,
}

/// What a build sent to the render
pub struct Built {
    pub parts: Parts,
    pub error: Option<RenderError>,
}

/// An update that has taken this long shows that it is building
pub const SLOW_BUILD: Duration = Duration::from_millis(300);

impl RenderControl {
    pub fn edited(&mut self, now: Instant) {
        self.edit_pending = true;
        self.edit_timer.edited(now);
    }

    /// Acts on edits: a camera change goes straight to the running render
    /// through the view, the other parts that changed are built into an
    /// update to it. With no render to update, a new one is started.
    pub fn schedule(&mut self, scene: &Scene, frame: usize, screen: (usize, usize)) {
        // A new render takes every edit, and one being built can not be
        // updated until it runs
        if !self.edit_pending || self.render_requested || self.loading_scene {
            return;
        }
        if screen.0 == 0 || screen.1 == 0 {
            return;
        }
        let running = self.update_sender.is_some();
        let Some(rendered) = self.rendered.as_mut().filter(|_| running) else {
            self.render_requested = true;
            self.edit_pending = false;
            return;
        };

        let inputs = Inputs::of(scene, frame, screen);
        let mut parts = match &inputs {
            Ok(inputs) => rendered.changed(inputs),
            // The build reports it
            Err(_) => Parts::ALL,
        };
        if parts.camera
            && let Ok(inputs) = &inputs
            && let Ok(camera) = scene.camera_at(frame)
        {
            self.orbit_camera = Some(OrbitCamera::from_config(&camera, 1.));
            self.applied_camera = Some((&camera).into());
            self.camera_updated = true;
            rendered.take(inputs, Parts::CAMERA);
            parts.camera = false;
        }

        // Folded into the next build once this one is done
        if self.build.is_some() {
            return;
        }
        self.edit_pending = false;
        if parts.is_empty() {
            // Everything rendered is as the scene has it, so there is no error
            self.render_error = None;
            if !self.camera_updated {
                self.edit_timer.discard();
            }
            return;
        }
        self.update_requested = Some(UpdateRequest {
            scene: scene.clone(),
            frame,
            screen,
            parts,
            inputs: inputs.ok(),
        });
    }

    /// Takes in what the build and the render have sent since last called
    pub fn receive(&mut self, image: &mut RenderedImage) {
        if let Some(build) = self.build.take() {
            match build.receiver.try_recv() {
                Ok(built) => {
                    if let (Some(rendered), Some(inputs)) = (&mut self.rendered, &build.inputs) {
                        rendered.take(inputs, built.parts);
                    }
                    if !built.parts.is_empty() {
                        self.edit_timer.sent();
                    }
                    self.render_error = built.error;
                }
                Err(TryRecvError::Empty) => self.build = Some(build),
                Err(TryRecvError::Disconnected) => {}
            }
        }

        while let Some(receiver) = &self.render_receiver {
            match receiver.try_recv() {
                Ok(RenderMessage::SampleRendered(progress)) => {
                    self.edit_timer.frame(progress.progress);
                    image.show(progress);
                    self.loading_scene = false;
                }
                Ok(RenderMessage::Error(error)) => {
                    self.render_error = Some(error);
                    self.end_render();
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => self.end_render(),
            }
        }
    }

    /// Lets go of the running render, which leaves the next edit to start a new one
    pub fn end_render(&mut self) {
        self.abort_sender = None;
        self.render_receiver = None;
        self.update_sender = None;
        self.rendered = None;
        self.build = None;
        self.update_requested = None;
        self.loading_scene = false;
    }

    /// Whether to show that the scene is building: while a new render builds
    /// it, or once an update has taken long enough to notice
    pub fn building(&self, now: Instant) -> bool {
        (self.loading_scene && !self.overlay)
            || self
                .build
                .as_ref()
                .is_some_and(|b| now.duration_since(b.started) >= SLOW_BUILD)
    }

    /// Moves the view back to the scene's camera, without restarting the render
    pub fn reset_view(&mut self) {
        if let Some(applied) = &self.applied_camera {
            self.orbit_camera = Some(OrbitCamera::from_config(&applied.into(), 1.));
            self.camera_updated = true;
        }
    }

    /// Writes the view into the scene's camera, so the camera's settings
    /// follow the view. A camera placed by expressions is left alone unless
    /// `replace_expressions`. Returns whether it wrote.
    pub fn write_view_to_camera(
        &mut self,
        scene: &mut Scene,
        frame: usize,
        replace_expressions: bool,
    ) -> bool {
        let Some(orbit) = &self.orbit_camera else {
            return false;
        };
        if !replace_expressions && scene.camera.position_uses_expressions() {
            return false;
        }
        let view = solstrale::camera::CameraConfig::from(orbit);
        let round = |v: f64| (v * 1000.).round() / 1000.;
        let pos = |v: solstrale::geo::vec3::Vec3| Pos::new(round(v.x), round(v.y), round(v.z));
        scene.camera.look_from = pos(view.look_from);
        scene.camera.look_at = Some(pos(view.look_at));

        // The render already shows this view, so the edit is recorded as
        // rendered rather than sent to it again
        if let Some(rendered) = &mut self.rendered {
            rendered.take_camera(scene, frame).ok();
        }
        self.applied_camera = scene.camera_at(frame).ok().map(|c| (&c).into());
        true
    }

    /// Whether the view has been moved away from the scene's camera
    pub fn view_moved(&self) -> bool {
        match (&self.orbit_camera, &self.applied_camera) {
            (Some(orbit), Some(applied)) => {
                !CameraSnapshot::from(&solstrale::camera::CameraConfig::from(orbit))
                    .approx_eq(applied)
            }
            _ => false,
        }
    }
}

/// Prints the time from each edit to the first frame it shows in
static TIME_EDITS: Lazy<bool> = Lazy::new(|| std::env::var_os("SOLSTRALE_TIME_EDITS").is_some());

/// Times an edit from when it is made to the first frame it shows in, which is
/// printed when `SOLSTRALE_TIME_EDITS` is set. That frame is told by the
/// accumulation restarting, which an edit that shows at all does.
#[derive(Default)]
pub struct EditTimer {
    /// The first edit not yet sent to the render
    edited: Option<Instant>,
    /// The first edit sent, and not yet shown
    sent: Option<Instant>,
    last_progress: f64,
    /// The time the last edit took to show
    pub shown_after: Option<Duration>,
}

impl EditTimer {
    fn edited(&mut self, now: Instant) {
        self.edited.get_or_insert(now);
    }

    /// The edits so far changed nothing
    fn discard(&mut self) {
        self.edited = None;
    }

    /// The edits so far have gone to the render
    pub fn sent(&mut self) {
        if let Some(edited) = self.edited.take() {
            self.sent.get_or_insert(edited);
        }
    }

    /// For a new render, whose first frame shows every edit
    pub fn new_render(&mut self) {
        self.sent();
        self.last_progress = f64::INFINITY;
    }

    fn frame(&mut self, progress: f64) {
        let restarted = progress < self.last_progress;
        self.last_progress = progress;
        if restarted && let Some(sent) = self.sent.take() {
            let shown_after = sent.elapsed();
            if *TIME_EDITS {
                eprintln!(
                    "Edit shown after {:.1} ms",
                    shown_after.as_secs_f64() * 1000.
                );
            }
            self.shown_after = Some(shown_after);
        }
    }
}

pub enum RenderMessage {
    SampleRendered(RenderProgress),
    Error(RenderError),
}

pub struct RenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub image_size_buffer: wgpu::Buffer,
    pub target_format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct RenderCallback {
    pub resources: Arc<RenderResources>,
    pub bind_group: Arc<wgpu::BindGroup>,
    pub width: u32,
    pub height: u32,
}

impl eframe::egui_wgpu::CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        queue.write_buffer(
            &self.resources.image_size_buffer,
            0,
            bytemuck::cast_slice(&[self.width as f32, self.height as f32]),
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        render_pass.set_pipeline(&self.resources.pipeline);
        render_pass.set_bind_group(0, Some(self.bind_group.as_ref()), &[]);
        render_pass.draw(0..3, 0..1);
    }
}

/// The bind group the blit was last painted with, and what it was built for.
struct CachedBlitBindGroup {
    bind_group: Arc<wgpu::BindGroup>,
    output_buffer: wgpu::Buffer,
    width: u32,
    height: u32,
}

pub struct RenderedImage {
    pub output_buffer: Option<wgpu::Buffer>,
    pub render_resources: Option<Arc<RenderResources>>,
    pub progress: f64,
    pub fps: f64,
    pub estimated_time_left: Duration,
    /// Of the image in `output_buffer`, which can be another size than the viewport
    pub width: u32,
    pub height: u32,
    blit: Option<CachedBlitBindGroup>,
}

impl RenderedImage {
    /// Takes in a frame from the renderer
    pub fn show(&mut self, progress: RenderProgress) {
        // The renderer mostly reports the same buffer. Only swapping the
        // handle when it really changed keeps the cached blit bind group valid.
        if self.output_buffer.as_ref() != Some(&progress.output_buffer) {
            self.output_buffer = Some(progress.output_buffer);
        }
        self.width = progress.width;
        self.height = progress.height;
        self.progress = progress.progress;
        if let Some(fps) = progress.fps {
            self.fps = fps;
        }
        self.estimated_time_left = progress.estimated_time_left;
    }

    /// Bind group for painting the current output buffer.
    ///
    /// The bindings only change when the renderer hands over a different buffer
    /// or the image is resized, so the bind group is built once and then
    /// reused. Building one per frame is pure churn on the UI thread, and a
    /// drag repaints continuously.
    pub fn blit_bind_group(&mut self) -> Option<Arc<wgpu::BindGroup>> {
        let resources = self.render_resources.clone()?;
        let output_buffer = self.output_buffer.clone()?;
        let (width, height) = (self.width, self.height);

        let up_to_date = self.blit.as_ref().is_some_and(|cached| {
            cached.output_buffer == output_buffer
                && cached.width == width
                && cached.height == height
        });

        if !up_to_date {
            let bind_group = resources
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Blit Bind Group"),
                    layout: &resources.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: resources.image_size_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: output_buffer.as_entire_binding(),
                        },
                    ],
                });

            self.blit = Some(CachedBlitBindGroup {
                bind_group: Arc::new(bind_group),
                output_buffer,
                width,
                height,
            });
        }

        self.blit.as_ref().map(|cached| cached.bind_group.clone())
    }
}

impl Default for RenderedImage {
    fn default() -> Self {
        Self {
            output_buffer: None,
            render_resources: None,
            progress: 0.0,
            fps: 0.0,
            estimated_time_left: Duration::default(),
            width: 0,
            height: 0,
            blit: None,
        }
    }
}
