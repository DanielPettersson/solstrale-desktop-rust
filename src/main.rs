use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui::{
    Color32, ColorImage, Context, ProgressBar, SidePanel, TextureOptions, TopBottomPanel, Vec2,
};
use eframe::epaint::TextureHandle;
use eframe::{egui, run_native, App, Frame, NativeOptions};
use egui::CentralPanel;
use solstrale::ray_trace;

mod scene_model;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        resizable: true,
        initial_window_size: Some(Vec2 {
            x: 1000.0,
            y: 600.0,
        }),
        ..Default::default()
    };

    run_native(
        "Solstrale",
        native_options,
        Box::new(|cc| Box::new(SolstraleApp::new(cc))),
    )
}

struct SolstraleApp {
    abort_sender: Option<Sender<bool>>,
    render_info: Arc<Mutex<RenderInfo>>,
    texture_handle: TextureHandle,
    scene_yaml: String,
}

struct RenderInfo {
    color_image: ColorImage,
    image_updated: bool,
    progress: f64,
}

impl SolstraleApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let yaml = include_str!("scene.yaml");

        SolstraleApp {
            abort_sender: None,
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
            scene_yaml: yaml.parse().unwrap(),
        }
    }
}

impl App for SolstraleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let mut render_info = self.render_info.lock().unwrap();

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Render").clicked() {
                    if let Some(abort_sender) = &self.abort_sender {
                        abort_sender.send(true).ok();
                        self.abort_sender = None;
                    }
                }
                ui.add(ProgressBar::new(render_info.progress as f32));
            });
        });

        SidePanel::left("code-panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.scene_yaml)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .min_size(Vec2 { x: 300.0, y: 0.0 }),
                );
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if self.abort_sender.is_none() {
                self.abort_sender = Some(render(
                    self.render_info.clone(),
                    &self.scene_yaml,
                    ui.available_size(),
                    ui.ctx().clone(),
                ));
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

fn render(
    render_info: Arc<Mutex<RenderInfo>>,
    scene_yaml: &str,
    render_size: Vec2,
    ctx: Context,
) -> Sender<bool> {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();

    let scene = Arc::new(scene_model::create_scene(scene_yaml).unwrap());

    thread::spawn(move || {
        ray_trace(
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

    abort_sender
}
