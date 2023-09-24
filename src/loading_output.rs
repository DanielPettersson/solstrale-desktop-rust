use eframe::egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Ui};
use eframe::{egui, emath, epaint};
use emath::RectTransform;
use epaint::Shape;
use std::f64::consts::TAU;

pub fn show(ui: &mut Ui) {
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let time = ui.input(|i| i.time);
        let (_id, rect) = ui.allocate_space(ui.available_size());
        let (line_color, text_color) = if ui.visuals().dark_mode {
            (Color32::from_rgb(50, 50, 50), Color32::WHITE)
        } else {
            (Color32::from_rgb(200, 200, 200), Color32::BLACK)
        };

        let to_screen = RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

        let mut shapes = vec![];

        for &mode in &[2, 3, 5] {
            let mode = mode as f64;
            let n = 120;

            let points: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f64 / (n as f64);
                    let amp = (time * mode).sin() / mode;
                    let y = amp * (t * TAU / 2.0 * mode).sin();
                    to_screen * Pos2::new(t as f32, y as f32)
                })
                .collect();

            let thickness = 20.0 / mode as f32;
            shapes.push(Shape::line(points, Stroke::new(thickness, line_color)));
        }
        ui.fonts(|fonts| {
            shapes.push(Shape::text(
                fonts,
                to_screen * Pos2::new(0.5, 0.0),
                Align2::CENTER_CENTER,
                "Preparing scene...",
                FontId::monospace(to_screen.scale().x * 0.04),
                text_color,
            ))
        });

        ui.painter().extend(shapes);
    });
}
