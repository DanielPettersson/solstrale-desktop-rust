use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};

use eframe::egui::{Button, Context, Margin, ProgressBar, SidePanel, TopBottomPanel, Vec2};
use eframe::epaint::TextureHandle;
use eframe::{egui, run_native, App, Frame, IconData, NativeOptions};
use egui::{CentralPanel, ScrollArea, Window};
use egui_file::FileDialog;
use image::RgbImage;
use solstrale::renderer::RenderProgress;

use yaml_editor::yaml_editor;

use crate::render_output::render_output;
use crate::yaml_editor::create_layouter;

mod load_scene;
mod loading_output;
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
    rendered_image: RenderedImage,
    scene_yaml: String,
    error_info: ErrorInfo,
    dialogs: Dialogs,
}

pub struct Dialogs {
    load_scene_dialog: FileDialog,
    save_scene_dialog: FileDialog,
    save_output_dialog: FileDialog,
}

impl Default for Dialogs {
    fn default() -> Self {
        Dialogs {
            load_scene_dialog: load_scene::create(),
            save_scene_dialog: save_scene::create(None),
            save_output_dialog: save_output::create(),
        }
    }
}

pub struct RenderControl {
    pub abort_sender: Option<Sender<bool>>,
    pub render_receiver: Option<Receiver<RenderMessage>>,
    pub render_requested: bool,
    pub loading_scene: bool,
}

impl Default for RenderControl {
    fn default() -> Self {
        Self {
            abort_sender: None,
            render_receiver: None,
            render_requested: true,
            loading_scene: true,
        }
    }
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
    pub fn handle_str(&mut self, err: &str) {
        self.show_error = true;
        self.error_message = err.to_string();
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
                ui.menu_button("File", |ui| {
                    ui.menu_button("Scene", |ui| {
                        if ui.button("Load").clicked() {
                            ui.close_menu();
                            self.dialogs.load_scene_dialog.open();
                        }
                        if ui.button("Save").clicked() {
                            ui.close_menu();
                            self.dialogs.save_scene_dialog.open();
                        }
                    });
                    let save_output_button_clicked = ui
                        .add_enabled(self.rendered_image.progress > 0., Button::new("Save image"))
                        .clicked();
                    if save_output_button_clicked {
                        ui.close_menu();
                        self.dialogs.save_output_dialog.open();
                    }
                });

                let render_button_clicked = ui
                    .add_enabled(
                        render_button::is_enabled(&self.render_control),
                        Button::new("Render"),
                    )
                    .clicked();

                render_button::handle_click(
                    render_button_clicked,
                    &mut self.render_control,
                    &mut self.error_info,
                    ui,
                );
            });
        });

        load_scene::handle_dialog(
            &mut self.dialogs.load_scene_dialog,
            &mut self.dialogs.save_scene_dialog,
            &mut self.error_info,
            &mut self.scene_yaml,
            &mut self.render_control,
            ctx,
        );

        save_scene::handle_dialog(
            &mut self.dialogs.save_scene_dialog,
            &mut self.error_info,
            &self.scene_yaml,
            ctx,
        );

        save_output::handle_dialog(
            &mut self.dialogs.save_output_dialog,
            &mut self.error_info,
            &self.rendered_image,
            ctx,
        );

        TopBottomPanel::bottom("bottom-panel")
            .frame(egui::Frame {
                inner_margin: Margin {
                    left: 0.0,
                    right: 0.0,
                    top: 4.0,
                    bottom: 0.0,
                },
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.add(ProgressBar::new(self.rendered_image.progress as f32));
            });

        SidePanel::left("code-panel")
            .frame(egui::Frame {
                inner_margin: Margin::same(2.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add(yaml_editor(
                        &mut self.scene_yaml,
                        &mut create_layouter(),
                        Vec2 {
                            x: 300.0,
                            y: ui.available_height(),
                        },
                    ));
                });
            });

        CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::same(0.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                match render_output(
                    &mut self.render_control,
                    &mut self.rendered_image,
                    &mut self.error_info,
                    &self.scene_yaml,
                    ui.available_size(),
                    ui.ctx().clone(),
                ) {
                    None => {
                        loading_output::show(ui);
                    }
                    Some(im) => {
                        ui.add(im);
                    }
                };
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
