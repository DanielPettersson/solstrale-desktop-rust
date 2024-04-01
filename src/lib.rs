use eframe::egui::{TextureHandle, Vec2};
use image::RgbImage;
use once_cell::sync::Lazy;
use solstrale::renderer::RenderProgress;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
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

#[derive(Default)]
pub struct RenderedImage {
    pub texture_handle: Option<TextureHandle>,
    pub rgb_image: Option<RgbImage>,
    pub progress: f64,
    pub fps: f64,
    pub estimated_time_left: Duration,
    pub width: u32,
    pub height: u32,
}
