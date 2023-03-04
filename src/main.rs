mod scene;

use crate::scene::create_scene;
use eframe::egui::{Color32, ColorImage, ProgressBar, TextureOptions, TopBottomPanel, Vec2};
use eframe::epaint::TextureHandle;
use eframe::{egui, run_native, NativeOptions};
use egui::CentralPanel;
use solstrale::post::OidnPostProcessor;
use solstrale::ray_trace;
use solstrale::renderer::shader::PathTracingShader;
use solstrale::renderer::RenderConfig;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

const WIDTH: usize = 800;
const HEIGHT: usize = 400;
const SAMPLES_PER_PIXEL: u32 = 50;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        initial_window_size: Some(Vec2::new(WIDTH as f32, HEIGHT as f32)),
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
        let (output_sender, output_receiver) = channel();
        let (_, abort_receiver) = channel();

        thread::spawn(move || {
            let scene = create_scene(RenderConfig {
                samples_per_pixel: SAMPLES_PER_PIXEL,
                shader: PathTracingShader::create(50),
                post_processor: Some(OidnPostProcessor::create()),
            });
            ray_trace(
                WIDTH as u32,
                HEIGHT as u32,
                scene,
                &output_sender,
                &abort_receiver,
            )
            .unwrap();
        });

        let render_info = Arc::new(Mutex::new(RenderInfo {
            color_image: ColorImage::new([WIDTH, HEIGHT], Color32::BLACK),
            image_updated: false,
            progress: 0.0,
        }));
        let render_info_clone = render_info.clone();

        let app = SolstraleApp {
            render_info,
            texture_handle: cc.egui_ctx.load_texture(
                "",
                ColorImage::new([WIDTH, HEIGHT], Color32::BLACK),
                TextureOptions::default(),
            ),
        };

        let clone_ctx = cc.egui_ctx.clone();

        thread::spawn(move || {
            for render_output in output_receiver {
                let image = render_output.render_image;
                let fs = image.as_flat_samples();
                let color_image = ColorImage::from_rgb(
                    [image.width() as usize, image.height() as usize],
                    fs.as_slice(),
                );
                let mut render_info = render_info_clone.lock().unwrap();
                render_info.image_updated = true;
                render_info.progress = render_output.progress;
                render_info.color_image = color_image;

                clone_ctx.request_repaint();
            }
        });

        app
    }
}

impl eframe::App for SolstraleApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut render_info = self.render_info.lock().unwrap();

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.add(ProgressBar::new(render_info.progress as f32));
        });
        CentralPanel::default().show(ctx, |ui| {
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
