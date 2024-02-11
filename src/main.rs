use std::error::Error;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use dark_light::Mode;
use eframe::egui::Event::CompositionUpdate;
use eframe::egui::{
    Align, Button, Context, Layout, Margin, ProgressBar, SidePanel, TopBottomPanel, Vec2,
    ViewportBuilder, Visuals,
};
use eframe::epaint::TextureHandle;
use eframe::{egui, icon_data, run_native, App, Frame, NativeOptions, Storage};
use egui::{CentralPanel, ScrollArea, Window};
use egui_file::FileDialog;
use hhmmss::Hhmmss;
use image::RgbImage;
use once_cell::sync::Lazy;
use solstrale::renderer::RenderProgress;

use yaml_editor::yaml_editor;

use crate::keyboard::{is_ctrl_space, is_enter};
use crate::model::scene::Scene;
use crate::model::{
    get_documentation_structure_by_yaml_path, DocumentationStructure, HelpDocumentation,
};
use crate::render_output::render_output;
use crate::yaml_editor::create_layouter;

mod help;
mod keyboard;
mod load_scene;
mod loading_output;
mod model;
mod render_button;
mod render_output;
mod reset_confirm;
mod save_image;
mod save_scene;
mod yaml_editor;

static ROOT_DOCUMENTATION_STRUCTURE: Lazy<DocumentationStructure> =
    Lazy::new(|| Scene::get_documentation_structure(0));

pub static DEFAULT_SCENE: Lazy<String> =
    Lazy::new(|| include_str!("../resources/scene.yaml").to_owned());

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../resources/icon.png");
    let icon = icon_data::from_png_bytes(icon_bytes).expect("Failed to load application icon");

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size(Vec2 {
                x: 1400.0,
                y: 800.0,
            })
            .with_min_inner_size(Vec2 { x: 700., y: 300. })
            .with_icon(icon)
            .with_app_id("solstrale".to_string()),
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
    display_help: bool,
}

pub struct Dialogs {
    load_scene_dialog: FileDialog,
    save_scene_dialog: FileDialog,
    save_output_dialog: FileDialog,
    show_reset_confirm_dialog: bool,
}

impl Default for Dialogs {
    fn default() -> Self {
        Dialogs {
            load_scene_dialog: load_scene::create(),
            save_scene_dialog: save_scene::create(None),
            save_output_dialog: save_image::create(),
            show_reset_confirm_dialog: false,
        }
    }
}

#[derive(Default)]
pub struct RenderControl {
    pub abort_sender: Option<Sender<bool>>,
    pub render_receiver: Option<Receiver<RenderMessage>>,
    pub render_requested: bool,
    pub loading_scene: bool,
    initial_render_started: bool,
    previous_frame_render_size: Vec2,
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
    pub num_pixels: u32,
}

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

impl SolstraleApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let mut yaml = DEFAULT_SCENE.to_owned();

        let mode = dark_light::detect();
        match mode {
            Mode::Dark => ctx.egui_ctx.set_visuals(Visuals::dark()),
            Mode::Light => ctx.egui_ctx.set_visuals(Visuals::light()),
            Mode::Default => ctx.egui_ctx.set_visuals(Visuals::default()),
        }

        let mut display_help = true;
        if let Some(storage) = ctx.storage {
            if let Some(value) = storage.get_string("display_help") {
                display_help =
                    bool::from_str(&value).expect("Invalid app configuration for display help");
            }
            if let Some(value) = storage.get_string("scene_yaml") {
                yaml = value;
            }
        }

        SolstraleApp {
            scene_yaml: yaml,
            display_help,
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

                if ui.button("Reset").clicked() {
                    self.dialogs.show_reset_confirm_dialog = true;
                }

                let render_button_enabled = render_button::is_enabled(&self.render_control);
                let render_button_clicked = ui
                    .add_enabled(render_button_enabled, Button::new("Render"))
                    .clicked();

                if render_button_enabled {
                    render_button::handle_click(
                        render_button_clicked,
                        &mut self.render_control,
                        &mut self.error_info,
                        ui,
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.checkbox(&mut self.display_help, "Display help")
                });
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

        save_image::handle_dialog(
            &mut self.dialogs.save_output_dialog,
            &mut self.error_info,
            &self.rendered_image,
            ctx,
        );

        reset_confirm::dialog(
            &mut self.dialogs.show_reset_confirm_dialog,
            &mut self.scene_yaml,
            ctx,
        );

        TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            egui::Frame::side_top_panel(ui.style())
                .inner_margin(Margin {
                    top: 3.,
                    ..Margin::default()
                })
                .show(ui, |ui| {
                    ui.add(
                        ProgressBar::new(self.rendered_image.progress as f32).text(format!(
                            "{:.0}% {} {:.1}FPS {:.1}MPPS",
                            self.rendered_image.progress * 100.,
                            self.rendered_image.estimated_time_left.hhmmss(),
                            self.rendered_image.fps,
                            self.rendered_image.fps * self.rendered_image.num_pixels as f64
                                / 1_000_000.,
                        )),
                    )
                });
        });

        let documentation_structure = get_documentation_structure_by_yaml_path(
            &ROOT_DOCUMENTATION_STRUCTURE,
            &yaml_editor::get_yaml_path(&self.scene_yaml, ctx),
        );

        SidePanel::left("code-panel").show(ctx, |ui| {
            egui::Frame::side_top_panel(ui.style())
                .inner_margin(Margin::same(0.))
                .show(ui, |ui| {
                    ScrollArea::both().min_scrolled_width(300.).show(ui, |ui| {
                        ui.add(yaml_editor(
                            &mut self.scene_yaml,
                            &mut create_layouter(),
                            Vec2 {
                                x: 300.0,
                                y: ui.available_height(),
                            },
                        ));

                        if is_ctrl_space(ui) {
                            if let Some(doc) = &documentation_structure {
                                yaml_editor::autocomplete(&mut self.scene_yaml, doc, ctx);
                            }
                        }

                        if is_enter(ui) {
                            yaml_editor::indent_new_line(&mut self.scene_yaml, ctx);
                        }
                    })
                });
        });

        SidePanel::right("help-panel")
            .min_width(300.0)
            .show_animated(ctx, self.display_help, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::side_top_panel(ui.style())
                        .show(ui, |ui| help::show(ui, &documentation_structure));
                })
            });

        CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::same(0.),
                ..Default::default()
            })
            .show(ctx, |ui| {
                // When window is first displayed, the available size can change in the
                // first few frames. So here we wait until the layout stabilizes until kicking off
                // the initial rendering, as to get 1:1 match to display pixel size
                if !self.render_control.initial_render_started {
                    if self.render_control.previous_frame_render_size == ui.available_size() {
                        self.render_control.render_requested = true;
                        self.render_control.initial_render_started = true;
                    }
                    self.render_control.previous_frame_render_size = ui.available_size();
                }

                match render_output(
                    &mut self.render_control,
                    &mut self.rendered_image,
                    &mut self.error_info,
                    &self.scene_yaml,
                    ui.available_size(),
                    ui.ctx(),
                ) {
                    None => {
                        loading_output::show(ui, self.rendered_image.texture_handle.clone());
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

        // Work around suspected eframe bug that caused new frames to be requested
        // all the time when yaml editor had focused caused by there always being
        // and empty IME composition update event. So just delete it here seems to work
        // without any noticeable side effects.
        ctx.input_mut(|i| {
            i.events.retain(|e| {
                if let CompositionUpdate(s) = e {
                    !s.eq("")
                } else {
                    true
                }
            })
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        storage.set_string("display_help", self.display_help.to_string());
        storage.set_string("scene_yaml", self.scene_yaml.to_owned())
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
