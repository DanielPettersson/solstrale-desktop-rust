use eframe::egui::{self, Color32, DragValue, Id, RichText, Stroke, TextEdit, Ui};

use crate::editor::EditCx;
use crate::editor::asset_picker::AssetKind;
use crate::model::expr::Expr;
use crate::model::num::Num;
use crate::model::pos::Pos;
use crate::model::rgb::Rgb;
use crate::model::scope::Scope;

const EXPR_WIDTH: f32 = 110.;

/// Why `src` can not be used as an expression here, if it can not
pub fn expr_error(src: &str, cx: &EditCx) -> Result<Expr, String> {
    let expr = Expr::parse(src).map_err(|e| e.to_string())?;
    if let Some(unknown) = expr.free_vars().iter().find(|v| !cx.visible.contains(*v)) {
        return Err(format!("unknown variable `{}`", unknown));
    }
    Ok(expr)
}

/// Text edited in a field whose model value only takes valid text. The model
/// keeps its last valid value while the field shows what is being typed.
fn buffered_text(
    ui: &mut Ui,
    id: Id,
    model: &str,
    width: f32,
    validate: impl Fn(&str) -> Result<(), String>,
) -> Option<String> {
    let mut text = ui
        .data(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| model.to_string());
    let error = validate(&text).err();
    let mut response = ui.add(
        TextEdit::singleline(&mut text)
            .id(id)
            .desired_width(width)
            // Grid columns size to the last frame's content, so without a
            // minimum the field never grows past the first frame's width
            .min_size(egui::vec2(width, 0.))
            .font(egui::TextStyle::Monospace),
    );
    if let Some(err) = &error {
        ui.painter().rect_stroke(
            response.rect.expand(1.),
            2.,
            Stroke::new(1.5, Color32::from_rgb(220, 60, 60)),
            egui::StrokeKind::Outside,
        );
        response = response.on_hover_text(err);
    }
    let committed = (response.changed() && validate(&text).is_ok()).then(|| text.clone());
    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, text));
    } else {
        // Not being edited, so show the model, which may have been changed
        // elsewhere, e.g. by loading a scene
        ui.data_mut(|d| d.remove::<String>(id));
    }
    committed
}

/// A number, as a draggable value or, toggled with ƒx, as an expression.
pub fn num_edit(ui: &mut Ui, num: &mut Num, cx: &EditCx) -> bool {
    let id = ui.auto_id_with("num");
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.;
        match num {
            Num::Lit(v) => {
                let speed = (v.abs() * 0.01).max(0.01);
                changed |= ui
                    .add(DragValue::new(v).speed(speed).max_decimals(6))
                    .changed();
            }
            Num::Expr(e) => {
                let src = e.src().to_string();
                // Grows with the expression, up to a point
                let width = (src.chars().count() as f32 * 7.5 + 16.).clamp(EXPR_WIDTH, 320.);
                if let Some(text) =
                    buffered_text(ui, id, &src, width, |t| expr_error(t, cx).map(|_| ()))
                    && let Ok(expr) = expr_error(&text, cx)
                {
                    *num = Num::Expr(expr);
                    changed = true;
                }
            }
        }

        let is_expr = matches!(num, Num::Expr(_));
        if ui
            .selectable_label(is_expr, RichText::new("ƒx").small())
            .on_hover_text(if is_expr {
                "Use a plain number"
            } else {
                "Use an expression, e.g. i * 2 or sin(frameIndex * 0.1)"
            })
            .clicked()
        {
            *num = match num {
                Num::Lit(v) => Num::Expr(Expr::parse(&format!("{}", v)).expect("numbers parse")),
                Num::Expr(_) => Num::Lit(
                    num.eval_scope(cx.preview.unwrap_or(&Scope::builtin(0)))
                        .unwrap_or(0.),
                ),
            };
            changed = true;
        }

        if let (Num::Expr(_), Some(scope)) = (&*num, cx.preview)
            && let Ok(v) = num.eval_scope(scope)
        {
            ui.label(RichText::new(format!("= {}", round_for_display(v))).weak())
                .on_hover_text(
                    "Value at the current frame, with loop variables at their first value",
                );
        }
    });
    changed
}

fn round_for_display(v: f64) -> f64 {
    (v * 1e4).round() / 1e4
}

/// Three numbers side by side, or one per line when any is an expression,
/// which is too wide to fit three of
fn triple_edit(ui: &mut Ui, nums: [&mut Num; 3], labels: [&str; 3], cx: &EditCx) -> bool {
    let vertical = nums.iter().any(|n| matches!(n, Num::Expr(_)));
    let mut changed = false;
    let components = |ui: &mut Ui| {
        for (n, l) in nums.into_iter().zip(labels) {
            ui.horizontal(|ui| {
                ui.label(RichText::new(l).weak().small());
                changed |= num_edit(ui, n, cx);
            });
        }
    };
    if vertical {
        ui.vertical(components);
    } else {
        ui.horizontal(components);
    }
    changed
}

pub fn pos_edit(ui: &mut Ui, pos: &mut Pos, cx: &EditCx) -> bool {
    triple_edit(
        ui,
        [&mut pos.x, &mut pos.y, &mut pos.z],
        ["x", "y", "z"],
        cx,
    )
}

/// Three numbers, plus a colour picker while they are plain numbers from 0 to 1
pub fn rgb_edit(ui: &mut Ui, rgb: &mut Rgb, cx: &EditCx) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        if let (Num::Lit(r), Num::Lit(g), Num::Lit(b)) = (&rgb.r, &rgb.g, &rgb.b)
            && [*r, *g, *b].iter().all(|c| (0. ..=1.).contains(c))
        {
            let mut color = [*r as f32, *g as f32, *b as f32];
            if ui.color_edit_button_rgb(&mut color).changed() {
                *rgb = Rgb::new(color[0] as f64, color[1] as f64, color[2] as f64);
                changed = true;
            }
        }
        changed |= triple_edit(
            ui,
            [&mut rgb.r, &mut rgb.g, &mut rgb.b],
            ["r", "g", "b"],
            cx,
        );
    });
    changed
}

/// A file path, typed or picked with a file dialog
pub fn path_edit(ui: &mut Ui, path: &mut String, kind: AssetKind, cx: &mut EditCx) -> bool {
    let id = ui.auto_id_with("path");
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui
            .add(TextEdit::singleline(path).desired_width(160.))
            .changed();
        if ui.button("…").on_hover_text("Choose a file").clicked() {
            cx.assets.request(id, kind, path);
        }
        if let Some(picked) = cx.assets.take(id) {
            *path = picked.display().to_string();
            changed = true;
        }
    });
    changed
}

/// A name that only changes to valid names
pub fn name_edit(
    ui: &mut Ui,
    name: &mut String,
    validate: impl Fn(&str) -> Result<(), String>,
) -> bool {
    let id = ui.auto_id_with("name");
    match buffered_text(ui, id, name, 80., validate) {
        Some(text) => {
            *name = text;
            true
        }
        None => false,
    }
}
