use crate::editor::outline::Selection;
use crate::model::orbit_camera::{CameraSnapshot, OrbitCamera};
use crate::model::scene::Scene;
use crate::render_scheduler::{Change, classify, debounce_remaining};
use eframe::egui::Vec2;
use eframe::wgpu;
use once_cell::sync::Lazy;
use solstrale::renderer::RenderProgress;
use solstrale::util::tone_map::ToneMapper;
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
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
    pub camera_config_sender: Option<Sender<solstrale::camera::CameraConfig>>,
    pub render_requested: bool,
    pub loading_scene: bool,
    pub initial_render_started: bool,
    pub previous_frame_render_size: Vec2,
    pub orbit_camera: Option<OrbitCamera>,
    /// The scene camera the orbit camera was last set from
    pub applied_camera: Option<CameraSnapshot>,
    /// Makes the next render move the view to the scene's camera
    pub reset_view: bool,
    pub camera_updated: bool,
    /// The scene and frame the running render was built from
    pub last_dispatched: Option<(Scene, usize)>,
    /// When the scene was last edited, while that edit is not rendered yet
    pub last_edit: Option<Instant>,
    /// A render is building its scene and has not produced a sample yet
    pub build_in_flight: bool,
    /// Cover the viewport while the next render loads, as for a new scene
    /// rather than an edit
    pub overlay_next_render: bool,
    /// Cover the viewport while the running render loads
    pub overlay: bool,
    pub render_error: Option<RenderError>,
}

/// Why the scene could not be rendered
pub struct RenderError {
    pub message: String,
    /// Where in the scene the error is, when known
    pub selection: Option<Selection>,
}

impl RenderControl {
    pub fn edited(&mut self, now: Instant) {
        self.last_edit = Some(now);
    }

    /// Acts on edits: a camera change goes straight to the running render,
    /// anything else restarts it once the edits pause. Returns when to be
    /// called again.
    pub fn schedule(&mut self, scene: &Scene, frame: usize, now: Instant) -> Option<Duration> {
        let last_edit = self.last_edit?;
        let change = classify(self.last_dispatched.as_ref(), scene, frame);
        if change == Change::Camera
            && self.abort_sender.is_some()
            && let Ok(camera) = scene.camera_at(frame)
        {
            self.orbit_camera = Some(OrbitCamera::from_config(&camera, 1.));
            self.applied_camera = Some((&camera).into());
            self.camera_updated = true;
            self.last_dispatched = Some((scene.clone(), frame));
            self.last_edit = None;
            return None;
        }
        if change == Change::None {
            self.last_edit = None;
            return None;
        }
        if let Some(remaining) = debounce_remaining(last_edit, now) {
            return Some(remaining);
        }
        // A build can not be aborted, so a new one waits for it rather than
        // piling up behind it
        if self.build_in_flight || self.render_requested {
            return Some(Duration::from_millis(50));
        }
        self.render_requested = true;
        self.last_edit = None;
        None
    }

    /// Moves the view back to the scene's camera, without restarting the render
    pub fn reset_view(&mut self) {
        if let Some(applied) = &self.applied_camera {
            self.orbit_camera = Some(OrbitCamera::from_config(&applied.into(), 1.));
            self.camera_updated = true;
        }
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

pub enum RenderMessage {
    SampleRendered(RenderProgress),
    Error {
        message: String,
        /// Where in the scene, as in [`model::ModelError::path`]
        path: Vec<String>,
    },
}

pub struct RenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub viewport_size_buffer: wgpu::Buffer,
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
            &self.resources.viewport_size_buffer,
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
    pub width: u32,
    pub height: u32,
    blit: Option<CachedBlitBindGroup>,
}

impl RenderedImage {
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
                            resource: resources.viewport_size_buffer.as_entire_binding(),
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
