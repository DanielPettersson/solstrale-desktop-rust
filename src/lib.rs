use eframe::egui::{TextureHandle, Vec2};
use eframe::wgpu;
use once_cell::sync::Lazy;
use solstrale::renderer::RenderProgress;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

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
}

pub enum RenderMessage {
    SampleRendered(RenderProgress),
    Error(String),
}

pub struct RenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub viewport_size_buffer: wgpu::Buffer,
}

impl eframe::egui_wgpu::CallbackTrait for RenderResources {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        Vec::new()
    }

    fn paint(
        &self,
        _info: eframe::egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'static>,
        _callback_resources: &eframe::egui_wgpu::CallbackResources,
    ) {
    }
}

pub struct RenderedImage {
    pub texture_handle: Option<TextureHandle>,
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
            texture_handle: None,
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
