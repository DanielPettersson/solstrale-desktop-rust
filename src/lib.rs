use crate::model::orbit_camera::OrbitCamera;
use eframe::egui::Vec2;
use eframe::wgpu;
use once_cell::sync::Lazy;
use solstrale::renderer::RenderProgress;
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use model::scene::Scene;
use std::sync::Mutex;

pub mod help;
pub mod keyboard;
pub mod load_scene;
pub mod loading_output;
pub mod model;
pub mod render_button;
pub mod render_output;
pub mod reset_confirm;
pub mod save_image;
pub mod save_scene;
pub mod yaml_editor;

pub static DEFAULT_SCENE: Lazy<String> =
    Lazy::new(|| include_str!("../resources/scene.yaml").to_owned());

/// The ray tracer binds ten storage buffers in a single compute stage, while
/// wgpu's default limit is eight. Ask for some headroom so that a new binding
/// in the library does not immediately break rendering here again.
pub const MIN_STORAGE_BUFFERS_PER_SHADER_STAGE: u32 = 12;

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
                .min(16)
                .max(MIN_STORAGE_BUFFERS_PER_SHADER_STAGE),
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
    pub scene: Option<Scene>,
    pub camera_updated: bool,
}

pub enum RenderMessage {
    SampleRendered(RenderProgress),
    Error(String),
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
    pub output_buffer: Arc<wgpu::Buffer>,
    pub width: u32,
    pub height: u32,
    pub bind_group: Arc<Mutex<Option<Arc<wgpu::BindGroup>>>>,
}

impl eframe::egui_wgpu::CallbackTrait for RenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.resources.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.resources.viewport_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.output_buffer.as_entire_binding(),
                },
            ],
        });
        *self.bind_group.lock().unwrap() = Some(Arc::new(bind_group));
        Vec::new()
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        if let Some(bind_group) = self.bind_group.lock().unwrap().as_ref() {
            render_pass.set_pipeline(&self.resources.pipeline);
            render_pass.set_bind_group(0, Some(bind_group.as_ref()), &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}

pub struct RenderedImage {
    pub output_buffer: Option<Arc<wgpu::Buffer>>,
    pub render_resources: Option<Arc<RenderResources>>,
    pub progress: f64,
    pub fps: f64,
    pub estimated_time_left: Duration,
    pub width: u32,
    pub height: u32,
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
        }
    }
}
