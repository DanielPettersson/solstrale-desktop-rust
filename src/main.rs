use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender};

use eframe::{App, egui, Frame, IconData, NativeOptions, run_native};
use eframe::egui::{
    Color32, ColorImage, Context, ProgressBar, SidePanel, TextureOptions,
    TopBottomPanel, Vec2,
};
use eframe::epaint::TextureHandle;
use egui::{CentralPanel, ScrollArea, Window};

use yaml_editor::yaml_editor;
use crate::render_output::render_output;

use crate::yaml_editor::create_layouter;

mod scene_model;
mod yaml_editor;
mod render_button;
mod render_output;

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
    scene_yaml: String,
    error_info: ErrorInfo,
}

pub struct RenderControl {
    pub abort_sender: Option<Sender<bool>>,
    pub render_requested: bool,
}

pub struct RenderInfo {
    pub texture_handle: TextureHandle,
    pub progress: f64,
}

pub struct ErrorInfo {
    pub show_error: bool,
    pub error_message: String,
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
                texture_handle: cc.egui_ctx.load_texture(
                    "",
                    ColorImage::new([1, 1], Color32::BLACK),
                    TextureOptions::default(),
                ),
                progress: 0.0,
            })),
            scene_yaml: yaml.parse().unwrap(),
            error_info: ErrorInfo {
                show_error: false,
                error_message: "".to_string(),
            },
        }
    }
}

impl App for SolstraleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let render_button_clicked = ui
                    .add_enabled(!self.error_info.show_error, render_button::render_button())
                    .clicked();
                render_button::handle_click(render_button_clicked, &mut self.render_control, ui);

                let render_info = self.render_info.lock().unwrap();
                ui.add(ProgressBar::new(render_info.progress as f32));
            });
        });

        SidePanel::left("code-panel").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add(yaml_editor(
                    &mut self.scene_yaml,
                    &mut create_layouter(),
                    Vec2 { x: 300.0, y: 0.0 }
                ));
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add(render_output(
                &mut self.render_control,
                &self.render_info,
                &mut self.error_info,
                &self.scene_yaml,
                ui.available_size(),
                ui.ctx().clone()
            ));
        });

        if self.error_info.show_error {
            Window::new("Error")
                .open(&mut self.error_info.show_error)
                .show(ctx, |ui| {
                    ui.label(&self.error_info.error_message);
                });
        }
    }
}
