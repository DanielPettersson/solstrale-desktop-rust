use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use eframe::{App, egui, Frame, IconData, NativeOptions, run_native};
use eframe::egui::{
    Color32, ColorImage, Context, ProgressBar, SidePanel, TextureOptions,
    TopBottomPanel, Vec2,
};
use eframe::epaint::TextureHandle;
use egui::CentralPanel;
use solstrale::ray_trace;

use yaml_editor::yaml_editor;

use crate::scene_model::{create_scene, Creator, SceneModel};
use crate::yaml_editor::create_layouter;

mod scene_model;
mod yaml_editor;
mod render_button;

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("icon.png");
    let icon = IconData::try_from_png_bytes(icon_bytes).expect("Failed to load application icon");

    let native_options = NativeOptions {
        resizable: true,
        initial_window_size: Some(Vec2 {
            x: 1000.0,
            y: 600.0,
        }),
        icon_data: Some(icon),
        app_id: Some("solstrale".to_string()),
        ..Default::default()
    };

    run_native(
        "Solstrale",
        native_options,
        Box::new(|cc| Box::new(SolstraleApp::new(cc))),
    )
}

struct SolstraleApp {
    render_control: RenderControl,
    render_info: Arc<Mutex<RenderInfo>>,
    texture_handle: TextureHandle,
    scene_yaml: String,
    show_error: bool,
    error_message: String,
}

pub struct RenderControl {
    pub abort_sender: Option<Sender<bool>>,
    pub render_requested: bool,
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
            render_control: RenderControl {
                abort_sender: None,
                render_requested: true,
            },
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
        }
    }
}

impl App for SolstraleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        let mut render_info = self.render_info.lock().unwrap();

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let render_button_clicked = ui
                    .add_enabled(!self.show_error, render_button::render_button())
                    .clicked();
                render_button::handle_click(render_button_clicked, &mut self.render_control, ui);

                ui.add(ProgressBar::new(render_info.progress as f32));
            });
        });

        SidePanel::left("code-panel").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(yaml_editor(
                    &mut self.scene_yaml,
                    &mut create_layouter(),
                    Vec2 { x: 300.0, y: 0.0 }
                ));
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if self.render_control.abort_sender.is_none() && self.render_control.render_requested {

                let res = create_scene(&self.scene_yaml).and_then(|scene_model| render(
                    self.render_info.clone(),
                    &scene_model,
                    ui.available_size(),
                    ui.ctx().clone(),
                ));

                match res {
                    Ok(abort_sender) => self.render_control.abort_sender = Some(abort_sender),
                    Err(err) => {
                        self.render_control.render_requested = false;
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

fn render(
    render_info: Arc<Mutex<RenderInfo>>,
    scene_model: &SceneModel,
    render_size: Vec2,
    ctx: Context,
) -> Result<Sender<bool>, Box<dyn Error>> {
    let (output_sender, output_receiver) = channel();
    let (abort_sender, abort_receiver) = channel();

    let scene = scene_model.create()?;

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
