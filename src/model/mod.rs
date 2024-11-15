use std::collections::HashMap;
use std::error::Error;

use derive_more::Display;

use crate::model::pos::Pos;
use crate::model::scene::Scene;
use crate::model::template::apply_template;

mod albedo_shader;
mod blend;
mod bloom_post_processor;
mod r#box;
mod camera_config;
mod constant_medium;
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
mod normal_shader;
mod normal_texture;
mod obj_model;
mod path_tracing_shader;
mod plastic;
mod pos;
mod post_processor;
mod quad;
mod quarter_screen_width_height;
mod render_config;
mod rgb;
pub mod scene;
mod screen_width_height;
mod shader;
mod simple_shader;
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

pub struct CreatorContext {
    pub screen_width: usize,
    pub screen_height: usize,
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
    use crate::model::denoise_post_processor::DenoisePostProcessor;
    use crate::model::hittable::Hittable;
    use crate::model::lambertian::Lambertian;
    use crate::model::material::Material;
    use crate::model::metal::Metal;
    use crate::model::path_tracing_shader::PathTracingShader;
    use crate::model::post_processor::PostProcessor;
    use crate::model::render_config::RenderConfig;
    use crate::model::rgb::Rgb;
    use crate::model::shader::Shader;
    use crate::model::texture::Texture;
    use crate::model::transformation::Transformation;
    use crate::model::width_height::WidthHeight;
    use crate::model::*;

    #[test]
    fn serde() {
        let scene = Scene {
            world: vec![Hittable {
                r#box: Some(crate::model::r#box::Box {
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
                    material: Some(Material {
                        blend: Some(Box::new(Blend {
                            first: Material {
                                lambertian: Some(Lambertian {
                                    albedo: Some(Texture {
                                        color: Some(Rgb {
                                            r: 1.0,
                                            g: 0.0,
                                            b: 0.0,
                                        }),
                                        image: None,
                                    }),
                                    normal: None,
                                }),
                                ..Default::default()
                            },
                            second: Material {
                                metal: Some(Metal {
                                    albedo: Some(Texture {
                                        color: Some(Rgb {
                                            r: 0.0,
                                            g: 1.0,
                                            b: 0.0,
                                        }),
                                        image: None,
                                    }),
                                    normal: None,
                                    fuzz: Some(0.1),
                                }),
                                ..Default::default()
                            },
                            blend_factor: Some(0.5),
                        })),
                        ..Default::default()
                    }),
                    transformations: vec![Transformation {
                        rotation_x: Some(30.),
                        ..Default::default()
                    }],
                }),
                ..Default::default()
            }],
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
                width_height: Some(WidthHeight {
                    screen: None,
                    half_screen: None,
                    quarter_screen: None,
                    custom: Some(CustomWidthHeight {
                        width: 200,
                        height: 100,
                    }),
                }),
                samples_per_pixel: Some(50),
                shader: Some(Shader {
                    path_tracing: Some(PathTracingShader {
                        max_depth: Some(50),
                    }),
                    simple: None,
                    albedo: None,
                    normal: None,
                }),
                post_processors: vec![
                    PostProcessor {
                        bloom: Some(BloomPostProcessor {
                            kernel_size_fraction: Some(0.1),
                            threshold: Some(1.5),
                            max_intensity: None,
                        }),
                        denoise: None,
                    },
                    PostProcessor {
                        bloom: None,
                        denoise: Some(DenoisePostProcessor {}),
                    },
                ],
                preview_interval_ms: Some(1000),
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
  shader:
    path_tracing:
      max_depth: 50
  post_processors:
  - bloom:
      kernel_size_fraction: 0.1
      threshold: 1.5
  - denoise: {}
  preview_interval_ms: 1000
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
}
