use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use eframe::egui::{Button, Context, ProgressBar, SidePanel, TopBottomPanel, Vec2};
use eframe::epaint::TextureHandle;
use eframe::{egui, run_native, App, Frame, IconData, NativeOptions};
use egui::{CentralPanel, ScrollArea, Window};
use egui_file::FileDialog;
use image::RgbImage;

use crate::render_output::render_output;
use yaml_editor::yaml_editor;

use crate::yaml_editor::create_layouter;

mod load_scene;
mod render_button;
mod render_output;
mod save_output;
mod save_scene;
mod scene_model;
mod yaml_editor;

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("icon.png");
    let icon = IconData::try_from_png_bytes(icon_bytes).expect("Failed to load application icon");

    let native_options = NativeOptions {
        resizable: true,
        initial_window_size: Some(Vec2 {
            x: 1100.0,
            y: 800.0,
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

#[derive(Default)]
struct SolstraleApp {
    render_control: RenderControl,
    render_info: Arc<Mutex<RenderInfo>>,
    scene_yaml: String,
    error_info: ErrorInfo,
    dialogs: Dialogs,
}

#[derive(Default)]
pub struct Dialogs {
    load_scene_dialog: Option<FileDialog>,
    save_scene_dialog: Option<FileDialog>,
    save_output_dialog: Option<FileDialog>,
}

pub struct RenderControl {
    pub abort_sender: Option<Sender<bool>>,
    pub render_requested: bool,
}

impl Default for RenderControl {
    fn default() -> Self {
        Self {
            abort_sender: None,
            render_requested: true,
        }
    }
}

#[derive(Default)]
pub struct RenderInfo {
    pub texture_handle: Option<TextureHandle>,
    pub rgb_image: Option<RgbImage>,
    pub progress: f64,
}

#[derive(Default)]
pub struct ErrorInfo {
    pub show_error: bool,
    pub error_message: String,
}

impl ErrorInfo {
    pub fn handle(&mut self, err: Box<dyn Error>) {
        self.show_error = true;
        self.error_message = format!("{}", err);
    }
}

impl SolstraleApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let yaml = include_str!("scene.yaml");

        SolstraleApp {
            scene_yaml: yaml.parse().unwrap(),
            ..Default::default()
        }
    }
}

impl App for SolstraleApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        TopBottomPanel::top("top-panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Scene", |ui| {
                    if ui.button("Load").clicked() {
                        ui.close_menu();
                        load_scene::show(&mut self.dialogs);
                    }
                    if ui.button("Save").clicked() {
                        ui.close_menu();
                        save_scene::show(&mut self.dialogs);
                    }
                });

                let render_button_clicked = ui
                    .add_enabled(!self.error_info.show_error, Button::new("Render"))
                    .clicked();
                render_button::handle_click(render_button_clicked, &mut self.render_control, ui);

                let render_info = self.render_info.lock().unwrap();
                let save_output_button_clicked = ui
                    .add_enabled(render_info.rgb_image.is_some(), Button::new("Save output"))
                    .clicked();
                if save_output_button_clicked {
                    save_output::show(&mut self.dialogs);
                }
            });
        });

        load_scene::handle_dialog(
            &mut self.dialogs,
            &mut self.error_info,
            &mut self.scene_yaml,
            &ctx,
        );

        save_scene::handle_dialog(
            &mut self.dialogs,
            &mut self.error_info,
            &self.scene_yaml,
            &ctx,
        );

        if self.dialogs.save_output_dialog.is_some() {
            let render_info = self.render_info.lock().unwrap();
            save_output::handle_dialog(&mut self.dialogs, &mut self.error_info, &render_info, &ctx);
        }

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            let render_info = self.render_info.lock().unwrap();
            ui.add(ProgressBar::new(render_info.progress as f32));
        });

        SidePanel::left("code-panel").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add(yaml_editor(
                    &mut self.scene_yaml,
                    &mut create_layouter(),
                    Vec2 { x: 300.0, y: 0.0 },
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
                ui.ctx().clone(),
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
