use std::error::Error;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui::Event::Key;
use eframe::egui::{
    Color32, ColorImage, Context, Modifiers, ProgressBar, SidePanel, TextureOptions,
    TopBottomPanel, Vec2,
};
use eframe::epaint::TextureHandle;
use eframe::{egui, run_native, App, Frame, NativeOptions};

use egui::CentralPanel;
use solstrale::ray_trace;

mod scene_model;
mod yaml_highlighter;

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
    show_error: bool,
    error_message: String,
    render_requested: bool,
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
            show_error: false,
            error_message: "".to_string(),
            render_requested: true,
        }
    }
}

impl App for SolstraleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let mut render_info = self.render_info.lock().unwrap();

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let render_button_clicked = ui
                    .add_enabled(!self.show_error, egui::Button::new("Render"))
                    .clicked();

                if render_button_clicked || is_ctrl_r(ui) {
                    if let Some(abort_sender) = &self.abort_sender {
                        abort_sender.send(true).ok();
                        self.abort_sender = None;
                    }
                    self.render_requested = true;
                }
                ui.add(ProgressBar::new(render_info.progress as f32));
            });
        });

        SidePanel::left("code-panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let layout_job = yaml_highlighter::highlight(ui.ctx(), string);
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                ui.add(
                    egui::TextEdit::multiline(&mut self.scene_yaml)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .min_size(Vec2 { x: 300.0, y: 0.0 })
                        .layouter(&mut layouter),
                );
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if self.abort_sender.is_none() && self.render_requested && !self.show_error {
                let res = render(
                    self.render_info.clone(),
                    &self.scene_yaml,
                    ui.available_size(),
                    ui.ctx().clone(),
                );
                match res {
                    Ok(abort_sender) => self.abort_sender = Some(abort_sender),
                    Err(err) => {
                        self.render_requested = false;
                        self.show_error = true;
                        self.error_message = format!("{}", err)
                    }
                }
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

        if self.show_error {
            egui::Window::new("Error")
                .open(&mut self.show_error)
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                });
        }
    }
}

fn is_ctrl_r(ui: &mut egui::Ui) -> bool {
    ui.input(|input| {
        for event in input.events.clone() {
            if match event {
                Key {
                    key: egui::Key::R,
                    pressed: _pressed,
                    repeat: false,
                    modifiers,
                } => modifiers.matches(Modifiers::CTRL),
                _ => false,
            } {
                return true;
            }
        }
        false
    })
}

fn render(
    render_info: Arc<Mutex<RenderInfo>>,
    scene_yaml: &str,
    render_size: Vec2,
    ctx: Context,
) -> Result<Sender<bool>, Box<dyn Error>> {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();

    let scene = scene_model::create_scene(scene_yaml)?;

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

    Ok(abort_sender)
}
