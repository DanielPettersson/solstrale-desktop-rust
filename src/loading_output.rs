use std::f64::consts::TAU;

use eframe::egui::{Align2, Color32, FontId, Margin, Pos2, Rect, Stroke, Ui, Vec2};
use eframe::{egui, emath, epaint};
use emath::RectTransform;
use epaint::Shape;

pub fn show(ui: &mut Ui) {
    egui::Frame::canvas(ui.style())
        .inner_margin(Margin::same(0.))
        .show(ui, |ui| {
            ui.ctx().request_repaint();
            let time = ui.input(|i| i.time);
            let (_id, rect) = ui.allocate_space(ui.available_size());
            let (shadow_color, text_color) = if ui.visuals().dark_mode {
                (Color32::from_rgb(50, 50, 50), Color32::WHITE)
            } else {
                (Color32::from_rgb(200, 200, 200), Color32::BLACK)
            };

            let to_screen =
                RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

            let mut shapes = vec![];

            for &(mode, color) in &[
                (2., Color32::from_rgba_unmultiplied(200, 0, 0, 15)),
                (3., Color32::from_rgba_unmultiplied(0, 200, 0, 15)),
                (4., Color32::from_rgba_unmultiplied(0, 0, 200, 15)),
                (5., Color32::from_rgba_unmultiplied(200, 200, 0, 15)),
                (6., Color32::from_rgba_unmultiplied(0, 200, 200, 15)),
                (7., Color32::from_rgba_unmultiplied(200, 0, 200, 15)),
            ] {
                let n = 120;

                let points: Vec<Pos2> = (0..=n)
                    .map(|i| {
                        let t = i as f64 / (n as f64);
                        let amp = (time * mode).sin() / mode;
                        let y = amp * (t * TAU / 2.0 * mode).sin();
                        to_screen * Pos2::new(t as f32, y as f32)
                    })
                    .collect();

                let thickness = 40.0 / mode as f32;
                shapes.push(Shape::line(points, Stroke::new(thickness, color)));
            }
            ui.fonts(|fonts| {
                for &offset in &[
                    Vec2::new(0.002, 0.002),
                    Vec2::new(0.002, -0.002),
                    Vec2::new(-0.002, 0.002),
                    Vec2::new(-0.002, -0.002),
                ] {
                    shapes.push(Shape::text(
                        fonts,
                        to_screen * (Pos2::new(0.5, 0.0) + offset),
                        Align2::CENTER_CENTER,
                        "Preparing scene...",
                        FontId::monospace(to_screen.scale().x * 0.04),
                        shadow_color,
                    ));
                }

                shapes.push(Shape::text(
                    fonts,
                    to_screen * Pos2::new(0.5, 0.0),
                    Align2::CENTER_CENTER,
                    "Preparing scene...",
                    FontId::monospace(to_screen.scale().x * 0.04),
                    text_color,
                ));
            });

            ui.painter().extend(shapes);
        });
}
