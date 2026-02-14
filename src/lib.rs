use crate::model::orbit_camera::OrbitCamera;
use eframe::egui::Vec2;
use eframe::wgpu;
use once_cell::sync::Lazy;
use solstrale::renderer::RenderProgress;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

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
    pub render_requested: bool,
    pub loading_scene: bool,
    pub initial_render_started: bool,
    pub previous_frame_render_size: Vec2,
    pub orbit_camera: Option<OrbitCamera>,
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
