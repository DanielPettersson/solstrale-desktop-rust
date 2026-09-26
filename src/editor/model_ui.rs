//! Inspector editors for the scene model types.

use eframe::egui::{self, DragValue, Ui};

use crate::editor::asset_picker::AssetKind;
use crate::editor::widgets::{num_edit, path_edit, pos_edit, rgb_edit};
use crate::editor::{EditCx, form, list_edit, opt_row, row, variant_combo};
use crate::model::blend::Blend;
use crate::model::bloom_post_processor::BloomPostProcessor;
use crate::model::r#box::Box;
use crate::model::camera_config::CameraConfig;
use crate::model::custom_width_height::CustomWidthHeight;
use crate::model::denoise_post_processor::{DenoiseGuide, DenoisePostProcessor};
use crate::model::glass::Glass;
use crate::model::hittable::Hittable;
use crate::model::image::Image;
use crate::model::lambertian::Lambertian;
use crate::model::light::Light;
use crate::model::material::Material;
use crate::model::metal::Metal;
use crate::model::normal_texture::NormalTexture;
use crate::model::num::Num;
use crate::model::obj_model::ObjModel;
use crate::model::plastic::Plastic;
use crate::model::pos::Pos;
use crate::model::post_processor::PostProcessor;
use crate::model::quad::Quad;
use crate::model::render_config::RenderConfig;
use crate::model::repeat::Repeat;
use crate::model::rgb::Rgb;
use crate::model::saturation_post_processor::SaturationPostProcessor;
use crate::model::sphere::Sphere;
use crate::model::texture::Texture;
use crate::model::transformation::Transformation;
use crate::model::width_height::WidthHeight;

pub use crate::editor::Edit;

const NORMAL_HELP: &str = "Texture for the material's normals. Used to give the illusion of fine structure of the hittable";
const ALBEDO_HELP: &str = "Texture for the material's albedo color";
const TRANSFORMATIONS_HELP: &str =
    "Transformations applied to the position and size, in order from the top";

fn num_row(ui: &mut Ui, label: &str, help: &str, n: &mut Num, cx: &EditCx) -> bool {
    row(ui, label, help, |ui| num_edit(ui, n, cx))
}

fn opt_num_row(
    ui: &mut Ui,
    label: &str,
    help: &str,
    n: &mut Option<Num>,
    default: f64,
    cx: &EditCx,
) -> bool {
    opt_row(
        ui,
        label,
        help,
        n,
        &format!("Default {}", default),
        || Num::Lit(default),
        |ui, n| num_edit(ui, n, cx),
    )
}

fn pos_row(ui: &mut Ui, label: &str, help: &str, p: &mut Pos, cx: &EditCx) -> bool {
    row(ui, label, help, |ui| pos_edit(ui, p, cx))
}

fn opt_pos_row(
    ui: &mut Ui,
    label: &str,
    help: &str,
    p: &mut Option<Pos>,
    default: Pos,
    cx: &EditCx,
) -> bool {
    let text = format!("Default {}, {}, {}", default.x, default.y, default.z);
    opt_row(
        ui,
        label,
        help,
        p,
        &text,
        || default,
        |ui, p| pos_edit(ui, p, cx),
    )
}

fn opt_rgb_row(
    ui: &mut Ui,
    label: &str,
    help: &str,
    c: &mut Option<Rgb>,
    default: Rgb,
    cx: &EditCx,
) -> bool {
    let text = format!("Default {}, {}, {}", default.r, default.g, default.b);
    opt_row(
        ui,
        label,
        help,
        c,
        &text,
        || default,
        |ui, c| rgb_edit(ui, c, cx),
    )
}

fn opt_texture_row(
    ui: &mut Ui,
    label: &str,
    help: &str,
    t: &mut Option<Texture>,
    default_text: &str,
    cx: &mut EditCx,
) -> bool {
    opt_row(
        ui,
        label,
        help,
        t,
        default_text,
        Texture::default,
        |ui, t| t.edit(ui, cx),
    )
}

fn opt_normal_row(ui: &mut Ui, n: &mut Option<NormalTexture>, cx: &mut EditCx) -> bool {
    opt_row(
        ui,
        "Normal",
        NORMAL_HELP,
        n,
        "None",
        NormalTexture::default,
        |ui, n| n.edit(ui, cx),
    )
}

fn opt_material_row(ui: &mut Ui, m: &mut Option<Material>, cx: &mut EditCx) -> bool {
    opt_row(
        ui,
        "Material",
        "What the surface looks like",
        m,
        "Default grey lambertian",
        Material::default,
        |ui, m| m.edit(ui, cx),
    )
}

fn transformations_row(ui: &mut Ui, t: &mut Vec<Transformation>, cx: &mut EditCx) -> bool {
    row(ui, "Transformations", TRANSFORMATIONS_HELP, |ui| {
        ui.vertical(|ui| list_edit(ui, t, cx)).inner
    })
}

impl Edit for Hittable {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label("Type");
            let before = self.clone();
            if variant_combo(ui, self) {
                carry_over(&before, self);
                changed = true;
            }
        });
        ui.add_space(4.);
        changed
            | match self {
                Hittable::Sphere(s) => s.edit(ui, cx),
                Hittable::Model(m) => m.edit(ui, cx),
                Hittable::Quad(q) => q.edit(ui, cx),
                Hittable::Box(b) => b.edit(ui, cx),
                Hittable::Repeat(r) => r.edit(ui, cx),
            }
    }
}

/// Keeps the material and transformations when switching between shapes
fn carry_over(from: &Hittable, to: &mut Hittable) {
    let parts = |h: &Hittable| match h {
        Hittable::Sphere(s) => Some((s.material.clone(), s.transformations.clone())),
        Hittable::Model(m) => Some((m.material.clone(), m.transformations.clone())),
        Hittable::Quad(q) => Some((q.material.clone(), q.transformations.clone())),
        Hittable::Box(b) => Some((b.material.clone(), b.transformations.clone())),
        Hittable::Repeat(_) => None,
    };
    let Some((material, transformations)) = parts(from) else {
        return;
    };
    let (m, t) = match to {
        Hittable::Sphere(s) => (&mut s.material, &mut s.transformations),
        Hittable::Model(o) => (&mut o.material, &mut o.transformations),
        Hittable::Quad(q) => (&mut q.material, &mut q.transformations),
        Hittable::Box(b) => (&mut b.material, &mut b.transformations),
        Hittable::Repeat(_) => return,
    };
    *m = material;
    *t = transformations;
}

impl Edit for Sphere {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "sphere", |ui| {
            pos_row(
                ui,
                "Center",
                "Position of the sphere's center",
                &mut self.center,
                cx,
            ) | num_row(ui, "Radius", "Radius of the sphere", &mut self.radius, cx)
                | opt_material_row(ui, &mut self.material, cx)
                | transformations_row(ui, &mut self.transformations, cx)
        })
    }
}

impl Edit for Quad {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "quad", |ui| {
            pos_row(ui, "Q", "Position of a corner of the quad", &mut self.q, cx)
                | pos_row(
                    ui,
                    "U",
                    "Direction of the first edge from Q",
                    &mut self.u,
                    cx,
                )
                | pos_row(
                    ui,
                    "V",
                    "Direction of the other edge from Q",
                    &mut self.v,
                    cx,
                )
                | opt_material_row(ui, &mut self.material, cx)
                | transformations_row(ui, &mut self.transformations, cx)
        })
    }
}

impl Edit for Box {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "box", |ui| {
            pos_row(ui, "A", "Position of a corner of the box", &mut self.a, cx)
                | pos_row(
                    ui,
                    "B",
                    "Position of the corner opposite to A",
                    &mut self.b,
                    cx,
                )
                | opt_material_row(ui, &mut self.material, cx)
                | transformations_row(ui, &mut self.transformations, cx)
        })
    }
}

impl Edit for ObjModel {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "model", |ui| {
            let mut changed = row(ui, "File", "The .obj file to load", |ui| {
                let mut file = format!("{}{}", self.path, self.name);
                if path_edit(ui, &mut file, AssetKind::Obj, cx) {
                    let (path, name) = split_obj_path(&file);
                    self.path = path;
                    self.name = name;
                    true
                } else {
                    false
                }
            });
            changed |= opt_row(
                ui,
                "Material",
                "Used for the parts of the model that have no material in the file",
                &mut self.material,
                "Default white lambertian",
                Material::default,
                |ui, m| m.edit(ui, cx),
            );
            changed | transformations_row(ui, &mut self.transformations, cx)
        })
    }
}

/// Splits a file path into the folder, with a trailing separator, and the file
/// name, which is how the obj loader takes them
pub fn split_obj_path(file: &str) -> (String, String) {
    match file.rfind(['/', '\\']) {
        Some(i) => (file[..=i].to_string(), file[i + 1..].to_string()),
        None => (String::new(), file.to_string()),
    }
}

impl Edit for Repeat {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "repeat", |ui| {
            let mut changed = row(
                ui,
                "Variable",
                "Name of the loop variable, which expressions in the repeated hittables can use",
                |ui| {
                    crate::editor::widgets::name_edit(
                        ui,
                        &mut self.variable,
                        crate::model::scope::check_name,
                    )
                },
            );
            changed |= opt_num_row(
                ui,
                "From",
                "First value of the loop variable",
                &mut self.from,
                0.,
                cx,
            );
            changed |= num_row(
                ui,
                "To",
                "The loop stops before reaching this value",
                &mut self.to,
                cx,
            );
            changed |= opt_num_row(
                ui,
                "Step",
                "How much the loop variable changes each iteration. Negative counts down",
                &mut self.step,
                1.,
                cx,
            );
            ui.label("");
            ui.label(
                egui::RichText::new(format!(
                    "{} hittables repeated, edit them in the outline",
                    self.world.len()
                ))
                .weak(),
            );
            ui.end_row();
            changed
        })
    }
}

impl Edit for Material {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        let changed = variant_combo(ui, self);
        changed
            | match self {
                Material::Lambertian(m) => m.edit(ui, cx),
                Material::Glass(m) => m.edit(ui, cx),
                Material::Metal(m) => m.edit(ui, cx),
                Material::Plastic(m) => m.edit(ui, cx),
                Material::Light(m) => m.edit(ui, cx),
                Material::Blend(m) => m.edit(ui, cx),
            }
    }
}

impl Edit for Lambertian {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "lambertian", |ui| {
            opt_texture_row(
                ui,
                "Albedo",
                ALBEDO_HELP,
                &mut self.albedo,
                "Default grey",
                cx,
            ) | opt_normal_row(ui, &mut self.normal, cx)
        })
    }
}

impl Edit for Glass {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "glass", |ui| {
            opt_texture_row(
                ui,
                "Albedo",
                "Fraction of light transmitted per world unit travelled inside the glass",
                &mut self.albedo,
                "Default clear",
                cx,
            ) | opt_normal_row(ui, &mut self.normal, cx)
                | opt_num_row(
                    ui,
                    "Index of refraction",
                    "How much the path of light is bent when entering the material",
                    &mut self.index_of_refraction,
                    1.5,
                    cx,
                )
                | opt_num_row(
                    ui,
                    "Roughness",
                    "The roughness of the glass surfaces",
                    &mut self.roughness,
                    0.,
                    cx,
                )
        })
    }
}

impl Edit for Metal {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "metal", |ui| {
            opt_texture_row(
                ui,
                "Albedo",
                ALBEDO_HELP,
                &mut self.albedo,
                "Default grey",
                cx,
            ) | opt_normal_row(ui, &mut self.normal, cx)
                | opt_num_row(
                    ui,
                    "Fuzz",
                    "How rough the reflection is",
                    &mut self.fuzz,
                    0.05,
                    cx,
                )
        })
    }
}

impl Edit for Plastic {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "plastic", |ui| {
            opt_texture_row(
                ui,
                "Albedo",
                ALBEDO_HELP,
                &mut self.albedo,
                "Default grey",
                cx,
            ) | opt_normal_row(ui, &mut self.normal, cx)
                | opt_num_row(
                    ui,
                    "Glossiness",
                    "0 is matte and 1 is metal",
                    &mut self.glossiness,
                    0.1,
                    cx,
                )
        })
    }
}

impl Edit for Light {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "light", |ui| {
            opt_rgb_row(
                ui,
                "Color",
                "The color of the light emitted. The intensity is normally way over 1",
                &mut self.color,
                Rgb::new(15., 15., 15.),
                cx,
            ) | opt_row(
                ui,
                "Attenuation half length",
                "The distance at which the light has lost half its intensity",
                &mut self.attenuation_half_length,
                "No attenuation",
                || Num::Lit(1.),
                |ui, n| num_edit(ui, n, cx),
            )
        })
    }
}

impl Edit for Blend {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "blend", |ui| {
            row(ui, "First", "The first material that is blended", |ui| {
                ui.vertical(|ui| self.first.edit(ui, cx)).inner
            }) | row(ui, "Second", "The second material that is blended", |ui| {
                ui.vertical(|ui| self.second.edit(ui, cx)).inner
            }) | opt_num_row(
                ui,
                "Blend factor",
                "How much of the second material is used, from 0 to 1",
                &mut self.blend_factor,
                0.5,
                cx,
            )
        })
    }
}

impl Edit for Texture {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        let mut changed = variant_combo(ui, self);
        changed |= match self {
            Texture::Color(c) => rgb_edit(ui, c, cx),
            Texture::Image(i) => i.edit(ui, cx),
        };
        changed
    }
}

impl Edit for Image {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        path_edit(ui, &mut self.file, AssetKind::Image, cx)
    }
}

impl Edit for NormalTexture {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        path_edit(ui, &mut self.file, AssetKind::Image, cx)
    }
}

impl Edit for Transformation {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        match self {
            Transformation::Translation(p) => pos_edit(ui, p, cx),
            Transformation::Scale(n)
            | Transformation::RotationX(n)
            | Transformation::RotationY(n)
            | Transformation::RotationZ(n) => num_edit(ui, n, cx),
        }
    }
}

impl Edit for CameraConfig {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "camera", |ui| {
            pos_row(
                ui,
                "Look from",
                "Position where the camera is located",
                &mut self.look_from,
                cx,
            ) | opt_pos_row(
                ui,
                "Look at",
                "Position the camera is pointed at",
                &mut self.look_at,
                Pos::default(),
                cx,
            ) | opt_pos_row(
                ui,
                "Up",
                "The direction that is up for the camera",
                &mut self.up,
                Pos::new(0., 1., 0.),
                cx,
            ) | opt_num_row(
                ui,
                "Field of view",
                "Vertical field of view in degrees",
                &mut self.vertical_fov_degrees,
                60.,
                cx,
            ) | opt_num_row(
                ui,
                "Aperture size",
                "Size of the opening light enters the camera through. Larger gives a shallower depth of field",
                &mut self.aperture_size,
                0.,
                cx,
            )
        })
    }
}

impl Edit for RenderConfig {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "render_config", |ui| {
            let mut changed = opt_row(
                ui,
                "Size",
                "Width and height in pixels of the rendered image",
                &mut self.width_height,
                "Same as the window",
                WidthHeight::default,
                |ui, w| w.edit(ui, cx),
            );
            changed |= opt_row(
                ui,
                "Samples per pixel",
                "Number of rays shot for each pixel. More rays gives a less noisy image but takes longer",
                &mut self.samples_per_pixel,
                "Default 200",
                || 200,
                |ui, n| ui.add(DragValue::new(n).range(1..=1_000_000)).changed(),
            );
            changed |= opt_row(
                ui,
                "Preview",
                "Run the denoise and saturation post processors on every batch rather than only the last, so the image is filtered while it renders and while the camera moves. Costs render time",
                &mut self.preview,
                "Off",
                || true,
                |ui, b| ui.checkbox(b, "").changed(),
            );
            changed |= row(
                ui,
                "Post processors",
                "Applied to the image after rendering, in order from the top",
                |ui| {
                    ui.vertical(|ui| list_edit(ui, &mut self.post_processors, cx))
                        .inner
                },
            );
            changed
        })
    }
}

impl Edit for WidthHeight {
    fn edit(&mut self, ui: &mut Ui, _cx: &mut EditCx) -> bool {
        let mut changed = variant_combo(ui, self);
        if let WidthHeight::Custom(c) = self {
            changed |= c.edit(ui, _cx);
        }
        changed
    }
}

impl Edit for CustomWidthHeight {
    fn edit(&mut self, ui: &mut Ui, _: &mut EditCx) -> bool {
        ui.horizontal(|ui| {
            let w = ui
                .add(
                    DragValue::new(&mut self.width)
                        .range(1..=8000)
                        .suffix(" px"),
                )
                .changed();
            ui.label("×");
            let h = ui
                .add(
                    DragValue::new(&mut self.height)
                        .range(1..=8000)
                        .suffix(" px"),
                )
                .changed();
            w | h
        })
        .inner
    }
}

impl Edit for PostProcessor {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        match self {
            PostProcessor::Denoise(p) => p.edit(ui, cx),
            PostProcessor::Bloom(p) => p.edit(ui, cx),
            PostProcessor::Saturation(p) => p.edit(ui, cx),
        }
    }
}

impl Edit for DenoisePostProcessor {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "denoise", |ui| {
            opt_num_row(
                ui,
                "Strength",
                "How hard the filter is allowed to blur",
                &mut self.strength,
                1.,
                cx,
            ) | opt_row(
                ui,
                "Iterations",
                "Number of filter passes, each one reaching twice as far as the last",
                &mut self.iterations,
                "Default",
                || 5,
                |ui, n| ui.add(DragValue::new(n).range(1..=10)).changed(),
            ) | opt_row(
                ui,
                "Guide",
                "What about the surface in each pixel is used to avoid blurring across edges",
                &mut self.guide,
                "Default full",
                || DenoiseGuide::Full,
                |ui, g| {
                    let before = *g;
                    egui::ComboBox::from_id_salt("guide")
                        .selected_text(match g {
                            DenoiseGuide::Full => "Full",
                            DenoiseGuide::ColorOnly => "Color only",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(g, DenoiseGuide::Full, "Full");
                            ui.selectable_value(g, DenoiseGuide::ColorOnly, "Color only");
                        });
                    *g != before
                },
            )
        })
    }
}

impl Edit for BloomPostProcessor {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "bloom", |ui| {
            opt_num_row(
                ui,
                "Kernel size",
                "Size of the filter creating the bloom, as a fraction of the image",
                &mut self.kernel_size_fraction,
                0.1,
                cx,
            ) | opt_row(
                ui,
                "Threshold",
                "Brightness a pixel needs for the bloom to apply to it",
                &mut self.threshold,
                "Default",
                || Num::Lit(1.),
                |ui, n| num_edit(ui, n, cx),
            ) | opt_row(
                ui,
                "Max intensity",
                "Limits the intensity of the bloom",
                &mut self.max_intensity,
                "No limit",
                || Num::Lit(2.),
                |ui, n| num_edit(ui, n, cx),
            )
        })
    }
}

impl Edit for SaturationPostProcessor {
    fn edit(&mut self, ui: &mut Ui, cx: &mut EditCx) -> bool {
        form(ui, "saturation", |ui| {
            opt_num_row(
                ui,
                "Factor",
                "The amount of saturation applied to the image",
                &mut self.saturation_factor,
                0.5,
                cx,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obj_paths_split_into_folder_and_name() {
        assert_eq!(
            ("/models/".to_string(), "bunny.obj".to_string()),
            split_obj_path("/models/bunny.obj")
        );
        assert_eq!(
            (String::new(), "bunny.obj".to_string()),
            split_obj_path("bunny.obj")
        );
    }
}
