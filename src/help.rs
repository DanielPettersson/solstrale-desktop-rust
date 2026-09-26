use eframe::egui::{RichText, Ui};

use crate::model::expr::FUNCTIONS;

fn section(ui: &mut Ui, title: &str, body: &str) {
    ui.label(RichText::new(title).strong());
    ui.label(body);
    ui.add_space(8.);
}

pub fn show(ui: &mut Ui) {
    section(
        ui,
        "Editing",
        "Select a part of the scene in the outline to the left and change it in the inspector to the right. \
         Add hittables with +, and right click one for more, like duplicating or wrapping it in a repeat. \
         Drag hittables in the outline to reorder them. Hover a label in the inspector for what it does.",
    );
    section(
        ui,
        "Expressions",
        &format!(
            "Toggle ƒx next to a number to write it as an expression, e.g. sqrt(2) / 2 or sin(frameIndex * 0.1) * 3. \
             Expressions support + - * / % ^ and parentheses, and the functions {}. \
             They can read frameIndex (the frame number when batch rendering), pi, e, \
             the scene variables and the loop variables of the repeats around them.",
            FUNCTIONS
                .iter()
                .map(|(f, _, _)| *f)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    section(
        ui,
        "Variables and repeat",
        "Scene variables, under Scene in the outline, are named values any expression can use, each able to use the ones above it. \
         A repeat creates the hittables in it once for each value of its loop variable, from its from value up to but not including to. \
         E.g. a sphere in a repeat with variable i and to 10, with center i * 2, 0, 0, gives ten spheres in a row.",
    );
    section(
        ui,
        "Viewport",
        "Drag to orbit the camera, drag with the right or middle button to pan, and scroll to zoom. \
         Reset view goes back to the scene's camera. Ctrl+R restarts the render and Ctrl+S saves the scene.",
    );
    section(
        ui,
        "Progress bar",
        "Percentage completed, remaining time, FPS (frames rendered per second) and MPPS (million pixel samples rendered per second).",
    );
}
