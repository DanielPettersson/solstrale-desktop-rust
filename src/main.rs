mod scene;

use crate::scene::create_scene;
use eframe::egui::{
    Color32, ColorImage, Context, ProgressBar, TextureOptions, TopBottomPanel, Vec2,
};
use eframe::epaint::TextureHandle;
use eframe::{egui, run_native, App, Frame, NativeOptions};
use egui::CentralPanel;
use solstrale::post::OidnPostProcessor;
use solstrale::ray_trace_arc;
use solstrale::renderer::shader::PathTracingShader;
use solstrale::renderer::{RenderConfig, Scene};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

const SAMPLES_PER_PIXEL: u32 = 50;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        resizable: true,
        ..Default::default()
    };

    run_native(
        "Solstrale",
        native_options,
        Box::new(|cc| Box::new(SolstraleApp::new(cc))),
    )
}

struct SolstraleApp {
    scene: Arc<Scene>,
    rendering: bool,
    render_info: Arc<Mutex<RenderInfo>>,
    texture_handle: TextureHandle,
}

struct RenderInfo {
    color_image: ColorImage,
    image_updated: bool,
    progress: f64,
}

impl SolstraleApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        SolstraleApp {
            scene: Arc::new(create_scene(RenderConfig {
                samples_per_pixel: SAMPLES_PER_PIXEL,
                shader: PathTracingShader::create(50),
                post_processor: Some(OidnPostProcessor::create()),
            })),
            rendering: false,
            render_info: Arc::new(Mutex::new(RenderInfo {
                color_image: ColorImage::new([1, 1], Color32::BLACK),
                image_updated: false,
                progress: 0.0,
            })),
            texture_handle: cc.egui_ctx.load_texture(
                "",
                ColorImage::new([1, 1], Color32::BLACK),
                TextureOptions::default(),
            ),
        }
    }
}

impl App for SolstraleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let mut render_info = self.render_info.lock().unwrap();

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.add(ProgressBar::new(render_info.progress as f32));
        });
        CentralPanel::default().show(ctx, |ui| {
            if !self.rendering {
                render(
                    self.render_info.clone(),
                    self.scene.clone(),
                    ui.available_size(),
                    ui.ctx().clone(),
                );
                self.rendering = true;
            }

            if render_info.image_updated {
                self.texture_handle = ctx.load_texture(
                    "render_texture",
                    render_info.color_image.to_owned(),
                    TextureOptions::default(),
                );
                render_info.image_updated = false;
            }

            ui.image(&self.texture_handle, ui.available_size())
        });
    }
}

fn render(render_info: Arc<Mutex<RenderInfo>>, scene: Arc<Scene>, render_size: Vec2, ctx: Context) {
    let (output_sender, output_receiver) = channel();
    let (_, abort_receiver) = channel();

    thread::spawn(move || {
        ray_trace_arc(
            render_size.x as u32,
            render_size.y as u32,
            scene,
            &output_sender,
            &abort_receiver,
        )
        .unwrap();
    });

    thread::spawn(move || {
        for render_output in output_receiver {
            let image = render_output.render_image;
            let fs = image.as_flat_samples();
            let color_image = ColorImage::from_rgb(
                [image.width() as usize, image.height() as usize],
                fs.as_slice(),
            );
            let mut render_info = render_info.lock().unwrap();
            render_info.image_updated = true;
            render_info.progress = render_output.progress;
            render_info.color_image = color_image;

            ctx.request_repaint();
        }
    });
}
