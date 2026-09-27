use std::error::Error;
use std::fmt;

use eframe::wgpu;

use crate::model::scene::Scene;
use crate::model::scope::Scope;

pub mod blend;
pub mod bloom_post_processor;
pub mod r#box;
pub mod camera_config;
pub mod custom_width_height;
pub mod denoise_post_processor;
pub mod expr;
pub mod glass;
pub mod half_screen_width_height;
pub mod hittable;
pub mod image;
pub mod lambertian;
pub mod light;
pub mod material;
pub mod metal;
pub mod normal_texture;
pub mod num;
pub mod obj_model;
pub mod one_of;
pub mod orbit_camera;
pub mod plastic;
pub mod pos;
pub mod post_processor;
pub mod quad;
pub mod quarter_screen_width_height;
pub mod render_config;
pub mod repeat;
pub mod rgb;
pub mod saturation_post_processor;
pub mod scene;
pub mod scope;
pub mod screen_width_height;
pub mod sphere;
pub mod texture;
pub mod texture_cache;
pub mod transformation;
pub mod variables;
pub mod width_height;

#[cfg(test)]
mod scene_tests;

/// An error in the scene, with the path to where in the scene it is.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelError {
    /// Outermost first, e.g. `["world[2]", "repeat (i = 3)", "world[0]", "sphere"]`
    pub path: Vec<String>,
    pub message: String,
}

impl ModelError {
    pub fn new(message: &str) -> Self {
        Self {
            path: Vec::new(),
            message: message.to_string(),
        }
    }

    fn new_from_err(err: Box<dyn Error>) -> Self {
        match err.downcast::<ModelError>() {
            Ok(e) => *e,
            Err(e) => ModelError::new(&e.to_string()),
        }
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.path.join(" › "), self.message)
        }
    }
}

impl Error for ModelError {}

/// Adds where in the scene an error happened as it passes up through the model.
pub trait ErrorPath<T> {
    fn at(self, segment: impl FnOnce() -> String) -> Result<T, Box<dyn Error>>;
}

impl<T> ErrorPath<T> for Result<T, Box<dyn Error>> {
    fn at(self, segment: impl FnOnce() -> String) -> Result<T, Box<dyn Error>> {
        self.map_err(|e| {
            let mut e = ModelError::new_from_err(e);
            e.path.insert(0, segment());
            Box::new(e) as Box<dyn Error>
        })
    }
}

pub struct CreatorContext<'a> {
    pub screen_width: usize,
    pub screen_height: usize,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    /// Variables that expressions in the scene can read
    pub scope: &'a Scope,
    /// Leave a model turned off its axes with its tree refitted rather than
    /// built again: quicker to move, up to 23% slower to render
    pub refit_models: bool,
}

pub trait Creator<T> {
    fn create(&self, ctx: &CreatorContext) -> Result<T, Box<dyn Error>>;
}

/// Parses a scene file
pub fn parse_scene(yaml: &str) -> Result<Scene, Box<dyn Error>> {
    serde_yaml::from_str(yaml).map_err(|e| {
        let mut msg = e.to_string();
        if yaml.contains("{%") || yaml.contains("{{") {
            msg.push_str(
                "\nTera templates are no longer supported. Use variables, repeat and expressions instead.",
            );
        }
        Box::new(ModelError::new(&msg)) as Box<dyn Error>
    })
}

/// Writes a scene file
pub fn scene_to_yaml(scene: &Scene) -> Result<String, Box<dyn Error>> {
    Ok(serde_yaml::to_string(scene)?)
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
    use crate::model::num::Num;
    use crate::model::pos::Pos;
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
            variables: Default::default(),
            world: vec![Hittable::Box(crate::model::r#box::Box {
                a: Pos::new(1., 2., 3.),
                b: Pos::new(4., 5., 6.),
                material: Some(Material::Blend(Box::new(Blend {
                    first: Material::Lambertian(Lambertian {
                        albedo: Some(Texture::Color(Rgb::new(1.0, 0.0, 0.0))),
                        normal: None,
                    }),
                    second: Material::Metal(Metal {
                        albedo: Some(Texture::Color(Rgb::new(0.0, 1.0, 0.0))),
                        normal: None,
                        fuzz: Some(Num::Lit(0.1)),
                    }),
                    blend_factor: Some(Num::Lit(0.5)),
                }))),
                transformations: vec![Transformation::RotationX(Num::Lit(30.))],
            })],
            camera: CameraConfig {
                vertical_fov_degrees: Some(Num::Lit(0.0)),
                aperture_size: Some(Num::Lit(0.0)),
                look_from: Pos::new(0.0, 0.0, 0.0),
                look_at: Some(Pos::new(0.0, 0.0, 0.0)),
                up: Some(Pos::new(0.0, 1.0, 0.0)),
            },
            background_color: Some(Rgb::new(0.0, 0.0, 0.0)),
            render_configuration: Some(RenderConfig {
                width_height: Some(WidthHeight::Custom(CustomWidthHeight {
                    width: 200,
                    height: 100,
                })),
                samples_per_pixel: Some(50),
                post_processors: vec![
                    PostProcessor::Denoise(DenoisePostProcessor {
                        strength: Some(Num::Lit(1.0)),
                        iterations: Some(5),
                        guide: Some(DenoiseGuide::ColorOnly),
                    }),
                    PostProcessor::Bloom(BloomPostProcessor {
                        kernel_size_fraction: Some(Num::Lit(0.1)),
                        threshold: Some(Num::Lit(1.5)),
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
        parse_scene(include_str!("../../resources/scene.yaml")).unwrap();
    }

    #[test]
    fn tera_templates_get_a_hint() {
        let err = parse_scene(
            "camera:\n  look_from: 0, 0, 0\nworld:\n{% for x in range(end=3) %}\n{% endfor %}\n",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("Tera templates are no longer supported"),
            "{}",
            err
        );
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
                "expected only one of `sphere`, `model`, `quad`, `box`, `repeat`, found `sphere`, `quad`"
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
