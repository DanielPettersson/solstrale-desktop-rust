use std::str::FromStr;

use dark_light::Mode;
use eframe::egui::{
    Align, Button, Context, Direction, Layout, Margin, ProgressBar, SidePanel, TopBottomPanel,
    Vec2, ViewportBuilder, Visuals,
};
use eframe::{egui, icon_data, run_native, App, Frame, NativeOptions, Storage};
use egui::{CentralPanel, ScrollArea, Window};
use egui_file::FileDialog;
use hhmmss::Hhmmss;
use once_cell::sync::Lazy;

use solstrale_desktop_rust::keyboard::{is_ctrl_space, is_enter};
use solstrale_desktop_rust::model::scene::Scene;
use solstrale_desktop_rust::model::{
    get_documentation_structure_by_yaml_path, DocumentationStructure, HelpDocumentation,
};
use solstrale_desktop_rust::render_output::render_output;
use solstrale_desktop_rust::yaml_editor::{create_layouter, yaml_editor};
use solstrale_desktop_rust::{
    help, load_scene, loading_output, render_button, reset_confirm, save_image, save_scene,
    yaml_editor, ErrorInfo, RenderControl, RenderedImage, DEFAULT_SCENE,
};

static ROOT_DOCUMENTATION_STRUCTURE: Lazy<DocumentationStructure> =
    Lazy::new(|| Scene::get_documentation_structure(0));

fn main() -> eframe::Result<()> {
    let icon_bytes = include_bytes!("../../resources/icon.png");
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
        Box::new(|cc| Ok(Box::new(SolstraleApp::new(cc)))),
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
    dark_mode: bool,
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

impl SolstraleApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let mut yaml = DEFAULT_SCENE.to_owned();

        let mut dark_mode = match dark_light::detect() {
            Mode::Dark => Some(true),
            Mode::Light => Some(false),
            Mode::Default => None,
        };

        let mut display_help = true;
        if let Some(storage) = ctx.storage {
            if let Some(value) = storage.get_string("display_help") {
                display_help =
                    bool::from_str(&value).expect("Invalid app configuration for display help");
            }
            if let Some(value) = storage.get_string("dark_mode") {
                dark_mode =
                    Some(bool::from_str(&value).expect("Invalid app configuration for dark mode"));
            }
            if let Some(value) = storage.get_string("scene_yaml") {
                yaml = value;
            }
        }

        if let Some(d) = dark_mode {
            ctx.egui_ctx
                .set_visuals(if d { Visuals::dark() } else { Visuals::light() });
        }

        SolstraleApp {
            scene_yaml: yaml,
            display_help,
            dark_mode: dark_mode.unwrap_or(false),
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
                        if ui
                            .button("Load")
                            .on_hover_text("Load scene configuration from a file")
                            .clicked()
                        {
                            ui.close_menu();
                            self.dialogs.load_scene_dialog.open();
                        }
                        if ui
                            .button("Save")
                            .on_hover_text("Save the current scene configuration to a file")
                            .clicked()
                        {
                            ui.close_menu();
                            self.dialogs.save_scene_dialog.open();
                        }
                    });
                    let save_output_button = ui
                        .add_enabled(self.rendered_image.progress > 0., Button::new("Save image"))
                        .on_hover_text(
                            "Saves the currently showing render output to an image file",
                        );
                    let save_output_button_clicked = save_output_button.clicked();
                    if save_output_button_clicked {
                        ui.close_menu();
                        self.dialogs.save_output_dialog.open();
                    }
                });

                let reset_button = ui.button("Reset");
                if reset_button.clicked() {
                    self.dialogs.show_reset_confirm_dialog = true;
                }
                reset_button.on_hover_text("Resets the scene configuration to the default example");

                let render_button_enabled = render_button::is_enabled(&self.render_control);
                let render_button = ui.add_enabled(render_button_enabled, Button::new("Render"));
                let render_button_clicked = render_button.clicked();
                render_button.on_hover_text("Restart the image rendering");

                if render_button_enabled {
                    render_button::handle_click(
                        render_button_clicked,
                        &mut self.render_control,
                        &mut self.error_info,
                        ui,
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.checkbox(&mut self.display_help, "Display help");
                    if ui.checkbox(&mut self.dark_mode, "Dark mode").changed() {
                        ctx.set_visuals(if self.dark_mode {
                            Visuals::dark()
                        } else {
                            Visuals::light()
                        })
                    }
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
                            self.rendered_image.fps
                                * self.rendered_image.width as f64
                                * self.rendered_image.height as f64
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
                        loading_output::show(ui);
                    }
                    Some(im) => {
                        let ri_aspect =
                            self.rendered_image.width as f32 / self.rendered_image.height as f32;
                        let ui_aspect = ui.available_size().x / ui.available_size().y;

                        ui.with_layout(
                            Layout::from_main_dir_and_cross_align(
                                if ri_aspect > ui_aspect {
                                    Direction::LeftToRight
                                } else {
                                    Direction::TopDown
                                },
                                Align::Center,
                            ),
                            |ui| {
                                ui.add(im.maintain_aspect_ratio(true).shrink_to_fit());
                            },
                        );
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

    fn save(&mut self, storage: &mut dyn Storage) {
        storage.set_string("display_help", self.display_help.to_string());
        storage.set_string("dark_mode", self.dark_mode.to_string());
        storage.set_string("scene_yaml", self.scene_yaml.to_owned())
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
