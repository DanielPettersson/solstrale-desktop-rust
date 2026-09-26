use std::collections::HashMap;
use std::error::Error;

use derive_more::Display;
use eframe::wgpu;

use crate::model::pos::Pos;
use crate::model::scene::Scene;
use crate::model::template::apply_template;

mod blend;
mod bloom_post_processor;
mod r#box;
mod camera_config;
mod custom_width_height;
mod denoise_post_processor;
mod glass;
mod half_screen_width_height;
mod hittable;
mod image;
mod lambertian;
mod light;
mod material;
mod metal;
mod normal_texture;
mod obj_model;
mod one_of;
pub mod orbit_camera;
mod plastic;
mod pos;
mod post_processor;
mod quad;
mod quarter_screen_width_height;
mod render_config;
mod rgb;
mod saturation_post_processor;
pub mod scene;
mod screen_width_height;
mod sphere;
mod template;
mod texture;
mod transformation;
mod width_height;

#[derive(Clone, Debug, Display)]
struct ModelError {
    message: String,
}

impl ModelError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    fn new_from_err(err: Box<dyn Error>) -> Self {
        Self {
            message: format!("{}", err),
        }
    }
}

impl Error for ModelError {}

pub struct CreatorContext<'a> {
    pub screen_width: usize,
    pub screen_height: usize,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
}

pub trait Creator<T> {
    fn create(&self, ctx: &CreatorContext) -> Result<T, Box<dyn Error>>;
}

pub trait HelpDocumentation {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure;
}

#[derive(Clone)]
pub struct DocumentationStructure {
    pub description: String,
    pub fields: HashMap<String, FieldInfo>,
}

impl DocumentationStructure {
    pub fn new_simple(description: &str) -> DocumentationStructure {
        DocumentationStructure {
            description: description.to_string(),
            fields: Default::default(),
        }
    }
}

#[derive(Clone)]
pub enum FieldType {
    Normal,
    Optional,
    List,
    OptionalList,
}

#[derive(Clone)]
pub struct FieldInfo {
    pub description: String,
    pub field_type: FieldType,
    pub documentation_structure: DocumentationStructure,
}

impl FieldInfo {
    pub fn new(
        field_description: &str,
        field_type: FieldType,
        documentation_structure: DocumentationStructure,
    ) -> FieldInfo {
        FieldInfo {
            description: field_description.to_string(),
            field_type,
            documentation_structure,
        }
    }
    pub fn new_simple(
        field_description: &str,
        field_type: FieldType,
        description: &str,
    ) -> FieldInfo {
        FieldInfo {
            description: field_description.to_string(),
            field_type,
            documentation_structure: DocumentationStructure::new_simple(description),
        }
    }
}

pub fn get_documentation_structure_by_yaml_path(
    info: &DocumentationStructure,
    path: &[String],
) -> Option<DocumentationStructure> {
    if path.is_empty() {
        Some(info.clone())
    } else {
        match path.split_first() {
            None => None,
            Some((first, rest)) => match info.fields.get(first) {
                None => None,
                Some(child_info) => get_documentation_structure_by_yaml_path(
                    &child_info.documentation_structure,
                    rest,
                ),
            },
        }
    }
}

pub fn parse_scene_yaml(templated_yaml: &str, frame_index: usize) -> Result<Scene, Box<dyn Error>> {
    let yaml = apply_template(templated_yaml, frame_index)?;
    let scene: Scene = serde_yaml::from_str(&yaml)?;
    Ok(scene)
}

pub fn parse_option<'de, D>(a: Option<&str>, expected_field: &'static str) -> Result<f64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    a.ok_or(serde::de::Error::missing_field(expected_field))?
        .trim()
        .parse::<f64>()
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod test {
    use crate::model::blend::Blend;
    use crate::model::bloom_post_processor::BloomPostProcessor;
    use crate::model::camera_config::CameraConfig;
    use crate::model::custom_width_height::CustomWidthHeight;
    use crate::model::denoise_post_processor::{DenoiseGuide, DenoisePostProcessor};
    use crate::model::hittable::Hittable;
    use crate::model::lambertian::Lambertian;
    use crate::model::material::Material;
    use crate::model::metal::Metal;
    use crate::model::post_processor::PostProcessor;
    use crate::model::render_config::RenderConfig;
    use crate::model::rgb::Rgb;
    use crate::model::texture::Texture;
    use crate::model::transformation::Transformation;
    use crate::model::width_height::WidthHeight;
    use crate::model::*;

    #[test]
    fn serde() {
        let scene = Scene {
            world: vec![Hittable::Box(crate::model::r#box::Box {
                a: Pos {
                    x: 1.,
                    y: 2.,
                    z: 3.,
                },
                b: Pos {
                    x: 4.,
                    y: 5.,
                    z: 6.,
                },
                material: Some(Material::Blend(Box::new(Blend {
                    first: Material::Lambertian(Lambertian {
                        albedo: Some(Texture::Color(Rgb {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                        })),
                        normal: None,
                    }),
                    second: Material::Metal(Metal {
                        albedo: Some(Texture::Color(Rgb {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                        })),
                        normal: None,
                        fuzz: Some(0.1),
                    }),
                    blend_factor: Some(0.5),
                }))),
                transformations: vec![Transformation::RotationX(30.)],
            })],
            camera: CameraConfig {
                vertical_fov_degrees: Some(0.0),
                aperture_size: Some(0.0),
                look_from: Pos {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                look_at: Some(Pos {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
                up: Some(Pos {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }),
            },
            background_color: Some(Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            }),
            render_configuration: Some(RenderConfig {
                width_height: Some(WidthHeight::Custom(CustomWidthHeight {
                    width: 200,
                    height: 100,
                })),
                samples_per_pixel: Some(50),
                post_processors: vec![
                    PostProcessor::Denoise(DenoisePostProcessor {
                        strength: Some(1.0),
                        iterations: Some(5),
                        guide: Some(DenoiseGuide::ColorOnly),
                    }),
                    PostProcessor::Bloom(BloomPostProcessor {
                        kernel_size_fraction: Some(0.1),
                        threshold: Some(1.5),
                        max_intensity: None,
                    }),
                ],
                preview: None,
            }),
        };

        let yaml = serde_yaml::to_string(&scene).unwrap();
        assert_eq!(
            "render_configuration:
  width_height:
    custom:
      width: 200
      height: 100
  samples_per_pixel: 50
  post_processors:
  - denoise:
      strength: 1.0
      iterations: 5
      guide: color_only
  - bloom:
      kernel_size_fraction: 0.1
      threshold: 1.5
background_color: 0, 0, 0
camera:
  vertical_fov_degrees: 0.0
  aperture_size: 0.0
  look_from: 0, 0, 0
  look_at: 0, 0, 0
  up: 0, 1, 0
world:
- box:
    a: 1, 2, 3
    b: 4, 5, 6
    material:
      blend:
        first:
          lambertian:
            albedo:
              color: 1, 0, 0
        second:
          metal:
            albedo:
              color: 0, 1, 0
            fuzz: 0.1
        blend_factor: 0.5
    transformations:
    - rotation_x: 30.0
",
            yaml
        );

        let de_scene: Scene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene, de_scene);
    }

    /// The scene the app opens with, and resets to. Nothing else loads it
    /// until it is in front of a user.
    #[test]
    fn default_scene_parses() {
        parse_scene_yaml(include_str!("../../resources/scene.yaml"), 0).unwrap();
    }

    /// Every one-of variant, written the ways users write them, survives a
    /// write and a re-read.
    #[test]
    fn one_of_round_trip() {
        let yaml = "
render_configuration:
  width_height:
    half_screen: {}
  post_processors:
    - denoise: { }
    - bloom: { }
    - saturation:
        saturation_factor: 1.2
camera:
  look_from: 0, 0, -10
world:
  - sphere:
      center: 0, 0, 0
      radius: 1
      material:
        light: { }
  - quad:
      q: 0, 0, 0
      u: 1, 0, 0
      v: 0, 1, 0
      material:
        glass:
          albedo:
            image:
              file: tex.png
      transformations:
        - translation: 1, 2, 3
        - scale: 2
        - rotation_x: 10
        - rotation_y: 20
        - rotation_z: 30
  - box:
      a: 0, 0, 0
      b: 1, 1, 1
      material:
        blend:
          first:
            plastic: { }
          second:
            metal: { }
  - model:
      path: /tmp
      name: x.obj
";
        let scene: Scene = serde_yaml::from_str(yaml).unwrap();
        let written = serde_yaml::to_string(&scene).unwrap();
        let reread: Scene = serde_yaml::from_str(&written).unwrap();
        assert_eq!(scene, reread);
        assert!(written.contains("half_screen: {}"), "{}", written);
        assert!(written.contains("- denoise: {}"), "{}", written);
        assert!(written.contains("light: {}"), "{}", written);
        assert!(!written.contains("null"), "{}", written);
    }

    #[test]
    fn one_of_rejects_two_keys() {
        let err = serde_yaml::from_str::<Hittable>(
            "sphere:\n  center: 0, 0, 0\n  radius: 1\nquad:\n  q: 0, 0, 0\n  u: 1, 0, 0\n  v: 0, 1, 0\n",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains(
                "expected only one of `sphere`, `model`, `quad`, `box`, found `sphere`, `quad`"
            ),
            "{}",
            err
        );
    }

    #[test]
    fn one_of_rejects_no_key_unless_it_has_a_default() {
        let err = serde_yaml::from_str::<Hittable>("{}")
            .unwrap_err()
            .to_string();
        assert!(err.contains("expected one of `sphere`"), "{}", err);

        assert_eq!(
            Material::default(),
            serde_yaml::from_str::<Material>("{}").unwrap()
        );
        assert_eq!(
            Texture::Color(Rgb::new(0.8, 0.8, 0.8)),
            serde_yaml::from_str::<Texture>("{}").unwrap()
        );
    }

    #[test]
    fn one_of_rejects_unknown_key() {
        let err = serde_yaml::from_str::<Hittable>("spere:\n  radius: 1\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown field `spere`"), "{}", err);
    }

    #[test]
    fn one_of_error_has_location() {
        let err = serde_yaml::from_str::<Scene>(
            "camera:\n  look_from: 0, 0, 0\nworld:\n  - sphere:\n      center: 0, 0, 0\n      radius: 1\n    box:\n      a: 0, 0, 0\n      b: 1, 1, 1\n",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("line 4"), "{}", err);
        assert!(err.contains("expected only one of"), "{}", err);
    }
}
