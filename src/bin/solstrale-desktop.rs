use std::str::FromStr;

use dark_light::Mode;
use eframe::egui::{
    Align, Button, Id, Layout, Margin, Modal, Panel, ProgressBar, RichText, Ui, Vec2,
    ViewportBuilder, ViewportCommand, Visuals,
};
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::{App, Frame, NativeOptions, Storage, egui, icon_data, run_native};
use egui::{CentralPanel, ScrollArea, Window};
use egui_file_dialog::{DialogState, FileDialog};
use hhmmss::Hhmmss;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use solstrale_desktop_rust::document::Document;
use solstrale_desktop_rust::editor::asset_picker::AssetPicker;
use solstrale_desktop_rust::editor::inspector::inspector;
use solstrale_desktop_rust::editor::outline::{ERROR_COLOR, OutlineCx, Selection, apply, outline};
use solstrale_desktop_rust::keyboard::is_ctrl_s;
use solstrale_desktop_rust::model::num::Num;
use solstrale_desktop_rust::model::pos::Pos;
use solstrale_desktop_rust::render_output::render_output;
use solstrale_desktop_rust::{
    ErrorInfo, RenderControl, RenderedImage, device_descriptor, help, load_scene, loading_output,
    render_button, save_image, save_scene,
};

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
        // The ray tracer renders on egui's device, so it has to be created with
        // limits that fit the ray tracer's bindings as well.
        wgpu_options: WgpuConfiguration {
            wgpu_setup: WgpuSetup::CreateNew(WgpuSetupCreateNew {
                device_descriptor: Arc::new(device_descriptor),
                ..WgpuSetupCreateNew::without_display_handle()
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    run_native(
        "Solstrale",
        native_options,
        Box::new(|cc| Ok(Box::new(SolstraleApp::new(cc)))),
    )
}

/// Something that would discard unsaved changes
#[derive(Clone, Copy, Debug, PartialEq)]
enum PendingAction {
    New,
    Load,
}

struct SolstraleApp {
    render_control: RenderControl,
    rendered_image: RenderedImage,
    doc: Document,
    selection: Selection,
    assets: AssetPicker,
    error_info: ErrorInfo,
    dialogs: Dialogs,
    /// Waiting on the user to decide what to do with unsaved changes
    pending: Option<PendingAction>,
    /// Asking before "Use current view" replaces camera expressions
    confirm_use_view: bool,
    /// To do once the Save As dialog, opened to keep unsaved changes, is done
    after_save: Option<PendingAction>,
    /// Scene text from the last session that could not be loaded, kept in storage
    unparsed_scene: Option<String>,
    show_help: bool,
    dark_mode: bool,
    title: String,
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
            save_output_dialog: save_image::create(),
        }
    }
}

impl SolstraleApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let mut dark_mode = dark_light::detect().map_or(None, |m| Some(m == Mode::Dark));
        let mut error_info = ErrorInfo::default();
        let mut doc = Document::default();
        let mut unparsed_scene = None;

        if let Some(storage) = ctx.storage {
            if let Some(value) = storage.get_string("dark_mode") {
                dark_mode =
                    Some(bool::from_str(&value).expect("Invalid app configuration for dark mode"));
            }
            let (restored, lost) = Document::restore(
                storage.get_string("scene_yaml"),
                storage.get_string("scene_path"),
                storage.get_string("scene_dirty").as_deref() == Some("true"),
            );
            doc = restored;
            if let Some((yaml, err)) = lost {
                error_info.handle_str(&format!(
                    "The scene from the last session could not be loaded, so the default scene is open. \
                     Its text is kept in the app settings under scene_yaml_unparsed.\n\n{}",
                    err
                ));
                unparsed_scene = Some(yaml);
            }
            unparsed_scene = unparsed_scene.or(storage.get_string("scene_yaml_unparsed"));
        }

        if let Some(d) = dark_mode {
            ctx.egui_ctx
                .set_visuals(if d { Visuals::dark() } else { Visuals::light() });
        }

        let mut rendered_image = RenderedImage::default();
        if let Some(render_state) = &ctx.wgpu_render_state {
            rendered_image.render_resources = Some(Arc::new(
                solstrale_desktop_rust::render_output::create_render_resources(
                    &render_state.device,
                    &render_state.queue,
                    render_state.target_format,
                ),
            ));
        }

        let dialogs = Dialogs {
            save_scene_dialog: save_scene::create(doc.path.clone()),
            ..Dialogs::default()
        };

        SolstraleApp {
            render_control: RenderControl::default(),
            rendered_image,
            doc,
            selection: Selection::Scene,
            assets: AssetPicker::default(),
            error_info,
            dialogs,
            pending: None,
            confirm_use_view: false,
            after_save: None,
            unparsed_scene,
            show_help: false,
            dark_mode: dark_mode.unwrap_or(false),
            title: String::new(),
        }
    }

    /// Does `action` now, or asks first if it would discard unsaved changes
    fn request(&mut self, action: PendingAction) {
        if self.doc.dirty {
            self.pending = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn perform(&mut self, action: PendingAction) {
        match action {
            PendingAction::New => self.open(Document::default()),
            PendingAction::Load => self.dialogs.load_scene_dialog.pick_file(),
        }
    }

    fn open(&mut self, doc: Document) {
        self.dialogs.save_scene_dialog = save_scene::create(doc.path.clone());
        self.doc = doc;
        self.selection = Selection::Scene;
        self.error_info.show_error = false;
        self.render_control.render_requested = true;
        self.render_control.reset_view = true;
        self.render_control.overlay_next_render = true;
        self.render_control.last_edit = None;
    }

    /// Saves to the scene's file, or asks for one. Returns whether it saved now.
    fn save(&mut self) -> bool {
        match self.doc.path.clone() {
            Some(path) => match self.doc.save_to(&path) {
                Ok(()) => true,
                Err(err) => {
                    self.error_info.handle(err);
                    false
                }
            },
            None => {
                self.dialogs.save_scene_dialog.save_file();
                false
            }
        }
    }

    fn handle_dialogs(&mut self, ctx: &egui::Context) {
        self.dialogs.load_scene_dialog.update(ctx);
        if let Some(path) = self.dialogs.load_scene_dialog.take_picked() {
            match Document::load(&path) {
                Ok(doc) => self.open(doc),
                // The scene that was open stays open
                Err(err) => self.error_info.handle(err),
            }
        }

        self.dialogs.save_scene_dialog.update(ctx);
        if let Some(path) = self.dialogs.save_scene_dialog.take_picked() {
            match self.doc.save_to(&path) {
                Ok(()) => {
                    self.dialogs.save_scene_dialog = save_scene::create(Some(path));
                    if let Some(action) = self.after_save.take() {
                        self.perform(action);
                    }
                }
                Err(err) => self.error_info.handle(err),
            }
        }
        if *self.dialogs.save_scene_dialog.state() == DialogState::Cancelled {
            self.after_save = None;
        }

        save_image::handle_dialog(
            &mut self.dialogs.save_output_dialog,
            &mut self.error_info,
            &self.rendered_image,
            ctx,
        );

        self.assets.update(ctx);
    }

    fn edited(&mut self) {
        self.doc.dirty = true;
        self.render_control.edited(Instant::now());
    }

    /// Writes the orbited view into the scene's camera. Asks first, unless
    /// `confirmed`, when that would replace expressions.
    fn use_current_view(&mut self, confirmed: bool) {
        let Some(orbit) = &self.render_control.orbit_camera else {
            return;
        };
        let camera = &mut self.doc.scene.camera;
        let has_expr = |p: &Pos| [&p.x, &p.y, &p.z].iter().any(|n| matches!(n, Num::Expr(_)));
        if !confirmed
            && (has_expr(&camera.look_from) || camera.look_at.as_ref().is_some_and(has_expr))
        {
            self.confirm_use_view = true;
            return;
        }
        let view = solstrale::camera::CameraConfig::from(orbit);
        let round = |v: f64| (v * 1000.).round() / 1000.;
        let pos = |v: solstrale::geo::vec3::Vec3| Pos::new(round(v.x), round(v.y), round(v.z));
        camera.look_from = pos(view.look_from);
        camera.look_at = Some(pos(view.look_at));
        self.selection = Selection::Camera;
        self.edited();
    }

    fn use_view_modal(&mut self, ctx: &egui::Context) {
        if !self.confirm_use_view {
            return;
        }
        let mut choice = None;
        let modal = Modal::new(Id::new("use-current-view")).show(ctx, |ui| {
            ui.heading("Replace expressions?");
            ui.label(
                "The camera's position uses expressions. Replace them with the view's numbers?",
            );
            ui.add_space(8.);
            ui.horizontal(|ui| {
                if ui.button("Replace").clicked() {
                    choice = Some(true);
                }
                if ui.button("Cancel").clicked() {
                    choice = Some(false);
                }
            });
        });
        match choice {
            Some(replace) => {
                self.confirm_use_view = false;
                if replace {
                    self.use_current_view(true);
                }
            }
            None if modal.should_close() => self.confirm_use_view = false,
            None => {}
        }
    }

    fn unsaved_changes_modal(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending else {
            return;
        };
        let name = self
            .doc
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or("the scene".to_string(), |n| n.to_string_lossy().to_string());
        let mut choice = None;
        let modal = Modal::new(Id::new("unsaved-changes")).show(ctx, |ui| {
            ui.heading("Unsaved changes");
            ui.label(format!("Save the changes to {} first?", name));
            ui.add_space(8.);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    choice = Some("save");
                }
                if ui.button("Discard").clicked() {
                    choice = Some("discard");
                }
                if ui.button("Cancel").clicked() {
                    choice = Some("cancel");
                }
            });
        });
        match choice {
            Some("save") => {
                self.pending = None;
                if self.save() {
                    self.perform(action);
                } else if self.doc.path.is_none() {
                    self.after_save = Some(action);
                }
            }
            Some("discard") => {
                self.pending = None;
                self.perform(action);
            }
            Some(_) => self.pending = None,
            None if modal.should_close() => self.pending = None,
            None => {}
        }
    }

    fn top_panel(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        Panel::top("top-panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .button("New")
                        .on_hover_text("Start from the example scene")
                        .clicked()
                    {
                        ui.close();
                        self.request(PendingAction::New);
                    }
                    if ui
                        .button("Load…")
                        .on_hover_text("Load a scene file")
                        .clicked()
                    {
                        ui.close();
                        self.request(PendingAction::Load);
                    }
                    if ui
                        .add(Button::new("Save").shortcut_text("Ctrl+S"))
                        .on_hover_text("Save the scene to its file")
                        .clicked()
                    {
                        ui.close();
                        self.save();
                    }
                    if ui
                        .button("Save as…")
                        .on_hover_text("Save the scene to a new file")
                        .clicked()
                    {
                        ui.close();
                        self.dialogs.save_scene_dialog.save_file();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.rendered_image.progress > 0.,
                            Button::new("Save image…"),
                        )
                        .on_hover_text("Save the render output to an image file")
                        .clicked()
                    {
                        ui.close();
                        self.dialogs.save_output_dialog.save_file();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("How to use").clicked() {
                        ui.close();
                        self.show_help = true;
                    }
                });

                let render_button_enabled = render_button::is_enabled(&self.render_control);
                let render_button = ui.add_enabled(render_button_enabled, Button::new("Render"));
                let render_button_clicked = render_button.clicked();
                render_button.on_hover_text("Restart the image rendering (Ctrl+R)");
                if render_button_enabled {
                    render_button::handle_click(
                        render_button_clicked,
                        &mut self.render_control,
                        &mut self.error_info,
                        ui,
                    );
                }

                if ui
                    .add_enabled(self.render_control.view_moved(), Button::new("Reset view"))
                    .on_hover_text("Move the view back to the scene's camera")
                    .clicked()
                {
                    self.render_control.reset_view();
                }
                if ui
                    .add_enabled(
                        self.render_control.view_moved(),
                        Button::new("Use current view"),
                    )
                    .on_hover_text("Set the scene's camera to the view")
                    .clicked()
                {
                    self.use_current_view(false);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
    }

    fn bottom_panel(&mut self, ui: &mut Ui) {
        Panel::bottom("bottom-panel").show(ui, |ui| {
            egui::Frame::side_top_panel(ui.style())
                .inner_margin(Margin {
                    top: 3,
                    ..Margin::default()
                })
                .show(ui, |ui| {
                    if let Some(err) = &self.render_control.render_error {
                        let text = RichText::new(format!(
                            "⚠ {}",
                            err.message.lines().next().unwrap_or("")
                        ))
                        .color(ERROR_COLOR);
                        let mut response = ui
                            .add(
                                egui::Label::new(text)
                                    .truncate()
                                    .sense(egui::Sense::click()),
                            )
                            .on_hover_text(&err.message);
                        if err.selection.is_some() {
                            response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                        }
                        if response.clicked()
                            && let Some(selection) = &err.selection
                        {
                            self.selection = selection.clone();
                        }
                    }
                    ui.horizontal(|ui| {
                        if self.render_control.loading_scene && !self.render_control.overlay {
                            ui.spinner().on_hover_text("Building the scene");
                        }
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
        });
    }

    fn outline_panel(&mut self, ui: &mut Ui) {
        Panel::left("outline-panel")
            .min_size(220.)
            .default_size(260.)
            .show(ui, |ui| {
                ui.add_space(4.);
                ScrollArea::vertical().show(ui, |ui| {
                    let scene_variables: BTreeSet<String> = self
                        .doc
                        .scene
                        .variables
                        .0
                        .iter()
                        .map(|(n, _)| n.clone())
                        .collect();
                    let error = self
                        .render_control
                        .render_error
                        .as_ref()
                        .and_then(|e| e.selection.as_ref());
                    let mut cx = OutlineCx {
                        selection: &mut self.selection,
                        error,
                        scene_variables: &scene_variables,
                        ops: Vec::new(),
                    };
                    outline(ui, &self.doc.scene, &mut cx);
                    let ops = cx.ops;
                    for op in ops {
                        if let Some(selection) = apply(&mut self.doc.scene.world, op) {
                            self.selection = selection;
                        }
                        self.edited();
                    }
                });
            });
    }

    fn inspector_panel(&mut self, ui: &mut Ui) {
        Panel::right("inspector-panel")
            .min_size(340.)
            .default_size(400.)
            .show(ui, |ui| {
                ui.add_space(4.);
                ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    if inspector(
                        ui,
                        &mut self.doc.scene,
                        &self.selection,
                        0,
                        &mut self.assets,
                    ) {
                        self.edited();
                    }
                });
            });
    }
}

impl App for SolstraleApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;

        if is_ctrl_s(ui) {
            self.save();
        }

        self.top_panel(ui);
        self.handle_dialogs(ctx);
        self.unsaved_changes_modal(ctx);
        self.use_view_modal(ctx);
        self.bottom_panel(ui);
        self.outline_panel(ui);
        self.inspector_panel(ui);

        CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: Margin::same(0),
                ..Default::default()
            })
            .show(ui, |ui| {
                let available_size = ui.available_size();

                // When window is first displayed, the available size can change in the
                // first few frames. So here we wait until the layout stabilizes until kicking off
                // the initial rendering, as to get 1:1 match to display pixel size
                if !self.render_control.initial_render_started {
                    if self.render_control.previous_frame_render_size == available_size
                        && available_size.x > 0.
                        && available_size.y > 0.
                    {
                        self.render_control.render_requested = true;
                        self.render_control.overlay_next_render = true;
                        self.render_control.initial_render_started = true;
                    }
                    self.render_control.previous_frame_render_size = available_size;
                }

                let rc = &self.render_control;
                if (rc.loading_scene && rc.overlay)
                    || (rc.render_requested && rc.overlay_next_render)
                {
                    loading_output::show(ui);
                }

                if let Some(wait) = self
                    .render_control
                    .schedule(&self.doc.scene, 0, Instant::now())
                {
                    ui.ctx().request_repaint_after(wait);
                }

                render_output(
                    ui,
                    &mut self.render_control,
                    &mut self.rendered_image,
                    Some(&self.doc.scene),
                    0,
                    available_size,
                );
            });

        if self.show_help {
            Window::new("How to use")
                .open(&mut self.show_help)
                .default_width(420.)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, help::show);
                });
        }

        if self.error_info.show_error {
            Window::new("Error")
                .open(&mut self.error_info.show_error)
                .show(ctx, |ui| {
                    ui.label(RichText::new(&self.error_info.error_message).monospace());
                });
        }

        let title = self.doc.title();
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        storage.set_string("dark_mode", self.dark_mode.to_string());
        match solstrale_desktop_rust::model::scene_to_yaml(&self.doc.scene) {
            Ok(yaml) => storage.set_string("scene_yaml", yaml),
            Err(err) => eprintln!("Could not store the scene: {}", err),
        }
        storage.set_string(
            "scene_path",
            self.doc
                .path
                .as_ref()
                .map_or(String::new(), |p| p.display().to_string()),
        );
        storage.set_string("scene_dirty", self.doc.dirty.to_string());
        if let Some(yaml) = &self.unparsed_scene {
            storage.set_string("scene_yaml_unparsed", yaml.clone());
        }
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
