use std::collections::HashMap;
use std::error::Error;

use derive_more::Display;

use crate::model::pos::Pos;
use crate::model::scene::Scene;

mod camera_config;
pub mod scene;
mod shader;
mod path_tracing_shader;
mod simple_shader;
mod albedo_shader;
mod normal_shader;
mod post_processor;
mod bloom_post_processor;
mod denoise_post_processor;
mod render_config;
mod pos;
mod hittable;
mod sphere;
mod obj_model;
mod quad;
mod r#box;
mod constant_medium;
mod material;
mod lambertian;
mod glass;
mod metal;
mod light;
mod texture;
mod image;
mod rgb;
mod transformation;

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

pub trait Creator<T> {
    fn create(&self) -> Result<T, Box<dyn Error>>;
}

pub trait HelpDocumentation {
    fn get_documentation_structure() -> DocumentationStructure;
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
}

#[derive(Clone)]
pub struct FieldInfo {
    pub description: String,
    pub field_type: FieldType,
    pub documentation_structure: DocumentationStructure
}

impl FieldInfo {
    pub fn new(field_description: &str, field_type: FieldType, documentation_structure: DocumentationStructure) -> FieldInfo {
        FieldInfo {
            description: field_description.to_string(),
            field_type,
            documentation_structure,
        }
    }
    pub fn new_simple(field_description: &str, field_type: FieldType, description: &str) -> FieldInfo {
        FieldInfo {
            description: field_description.to_string(),
            field_type,
            documentation_structure: DocumentationStructure::new_simple(description),
        }
    }
}

pub fn get_documentation_structure_by_yaml_path(info: &DocumentationStructure, path: &[String]) -> Option<DocumentationStructure> {
    if path.is_empty() {
        Some(info.clone())
    } else {
        match path.split_first() {
            None => None,
            Some((first, rest)) => {
                match info.fields.get(first) {
                    None => None,
                    Some(child_info) => get_documentation_structure_by_yaml_path(&child_info.documentation_structure, rest)
                }
            }
        }
    }
}

pub fn parse_scene_yaml(yaml: &str) -> Result<Scene, Box<dyn Error>> {
    let scene: Scene = serde_yaml::from_str(yaml)?;
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
    use crate::model::*;
    use crate::model::bloom_post_processor::BloomPostProcessor;
    use crate::model::camera_config::CameraConfig;
    use crate::model::denoise_post_processor::DenoisePostProcessor;
    use crate::model::hittable::Hittable;
    use crate::model::lambertian::Lambertian;
    use crate::model::material::Material;
    use crate::model::path_tracing_shader::PathTracingShader;
    use crate::model::post_processor::PostProcessor;
    use crate::model::render_config::RenderConfig;
    use crate::model::rgb::Rgb;
    use crate::model::shader::Shader;
    use crate::model::texture::Texture;
    use crate::model::transformation::Transformation;

    #[test]
    fn serde() {
        let scene = Scene {
            world: vec![Hittable {
                sphere: None,
                model: None,
                quad: None,
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
                    material: Material {
                        lambertian: Some(Lambertian {
                            albedo: Texture {
                                color: Some(Rgb {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                }),
                                image: None,
                            },
                            normal: None,
                        }),
                        glass: None,
                        metal: None,
                        light: None,
                    },
                    transformations: vec![Transformation {
                        translation: None,
                        scale: None,
                        rotation_x: Some(30.),
                        rotation_y: None,
                        rotation_z: None,
                    }],
                }),
                constant_medium: None,
            }],
            camera: CameraConfig {
                vertical_fov_degrees: 0.0,
                aperture_size: 0.0,
                look_from: Pos {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                look_at: Pos {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                up: Pos {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
            background_color: Rgb {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
            render_configuration: RenderConfig {
                samples_per_pixel: 50,
                shader: Shader {
                    path_tracing: Some(PathTracingShader { max_depth: 50 }),
                    simple: None,
                    albedo: None,
                    normal: None,
                },
                post_processors: vec![
                    PostProcessor {
                        bloom: Some(BloomPostProcessor {
                            kernel_size_fraction: 0.1,
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
                preview_interval_ms: 1000,
            },
        };

        let yaml = serde_yaml::to_string(&scene).unwrap();
        assert_eq!(
            "render_configuration:
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
      lambertian:
        albedo:
          color: 0, 0, 0
    transformations:
    - rotation_x: 30.0
",
            yaml
        );

        let de_scene: Scene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene, de_scene);
    }
}
