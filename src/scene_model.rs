use std::collections::HashMap;
use std::error::Error;

use crate::scene_model::HittableType::RotationY;
use crate::scene_model::MaterialType::Lambertian;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use solstrale::hittable::hittable_list::HittableList;
use solstrale::hittable::obj_model::load_obj_model;
use solstrale::hittable::sphere::Sphere;
use solstrale::hittable::translation::Translation;
use solstrale::hittable::Hittable as HittableTrait;
use solstrale::hittable::Hittables;
use solstrale::material::texture::{ImageTexture, SolidColor, Textures};
use solstrale::material::{Dielectric, DiffuseLight, Materials};
use solstrale::post::OidnPostProcessor;
use solstrale::renderer::shader::{AlbedoShader, NormalShader, PathTracingShader, SimpleShader};

pub fn create_scene(yaml: &str) -> Result<solstrale::renderer::Scene, Box<dyn Error>> {
    let scene: Scene = serde_yaml::from_str(yaml)?;
    scene.create()
}

trait Creator<T> {
    fn create(&self) -> Result<T, Box<dyn Error>>;
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Scene {
    render_configuration: RenderConfig,
    background_color: Vec3,
    camera: CameraConfig,
    world: Hittable,
}

impl Creator<solstrale::renderer::Scene> for Scene {
    fn create(&self) -> Result<solstrale::renderer::Scene, Box<dyn Error>> {
        Ok(solstrale::renderer::Scene {
            world: self.world.create()?,
            camera: self.camera.create()?,
            background_color: self.background_color.create()?,
            render_config: self.render_configuration.create()?,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct CameraConfig {
    vertical_fov_degrees: f64,
    aperture_size: f64,
    focus_distance: f64,
    look_from: Vec3,
    look_at: Vec3,
}

impl Creator<solstrale::camera::CameraConfig> for CameraConfig {
    fn create(&self) -> Result<solstrale::camera::CameraConfig, Box<dyn Error>> {
        Ok(solstrale::camera::CameraConfig {
            vertical_fov_degrees: self.vertical_fov_degrees,
            aperture_size: self.aperture_size,
            focus_distance: self.focus_distance,
            look_from: self.look_from.create()?,
            look_at: self.look_at.create()?,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum ShaderType {
    PathTracing,
    Albedo,
    Normal,
    Simple,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum PostProcessorType {
    Oidn,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct RenderConfig {
    samples_per_pixel: u32,
    shader: ShaderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    post_processor: Option<PostProcessorType>,
}

impl Creator<solstrale::renderer::RenderConfig> for RenderConfig {
    fn create(&self) -> Result<solstrale::renderer::RenderConfig, Box<dyn Error>> {
        Ok(solstrale::renderer::RenderConfig {
            samples_per_pixel: self.samples_per_pixel,
            shader: match self.shader {
                ShaderType::PathTracing => PathTracingShader::new(50),
                ShaderType::Albedo => AlbedoShader::new(),
                ShaderType::Normal => NormalShader::new(),
                ShaderType::Simple => SimpleShader::new(),
            },
            post_processor: self.post_processor.as_ref().map(|p| match p {
                PostProcessorType::Oidn => OidnPostProcessor::new(),
            }),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Creator<solstrale::geo::vec3::Vec3> for Vec3 {
    fn create(&self) -> Result<solstrale::geo::vec3::Vec3, Box<dyn Error>> {
        Ok(solstrale::geo::vec3::Vec3::new(self.x, self.y, self.z))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum HittableType {
    List,
    Sphere,
    Model,
    RotationY,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Hittable {
    r#type: HittableType,
    #[serde(skip_serializing_if = "Option::is_none")]
    material: Option<Material>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<Hittable>>,
}

impl Creator<Hittables> for Hittable {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        Ok(match self.r#type {
            HittableType::List => {
                let mut list = HittableList::new();
                for child in self
                    .children
                    .as_ref()
                    .ok_or("List expects children")?
                    .iter()
                {
                    list.add(child.create()?)
                }
                list
            }
            HittableType::Sphere => Sphere::new(
                get_pos_opt(&self.data)?,
                get_f64_opt(&self.data, "radius")?,
                self.material
                    .as_ref()
                    .ok_or("Sphere expects material")?
                    .create()?,
            ),
            HittableType::Model => {
                let model = load_obj_model(
                    get_str_opt(&self.data, "path")?,
                    get_str_opt(&self.data, "name")?,
                    get_f64_opt(&self.data, "scale")?,
                )?;

                let pos = get_pos_opt(&self.data)?;

                let translated = if pos.near_zero() {
                    model
                } else {
                    Translation::new(model, pos)
                };

                let angle_y = get_f64_opt(&self.data, "angle_y")?;

                if angle_y == 0. {
                    translated
                } else {
                    solstrale::hittable::rotation_y::RotationY::new(translated, angle_y)
                }
            }
            RotationY => {
                let child = self
                    .children
                    .as_ref()
                    .expect("RotationY expects children")
                    .iter()
                    .next()
                    .expect("RotationY expects a child");

                solstrale::hittable::rotation_y::RotationY::new(
                    child.create()?,
                    get_f64_opt(&self.data, "angle")?,
                )
            }
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum MaterialType {
    Lambertian,
    Glass,
    Light,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Material {
    r#type: MaterialType,
    #[serde(skip_serializing_if = "Option::is_none")]
    texture: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<HashMap<String, Value>>,
}

impl Creator<Materials> for Material {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        Ok(match self.r#type {
            Lambertian => solstrale::material::Lambertian::new(
                self.texture
                    .as_ref()
                    .ok_or("Lambertian expects texture")?
                    .create()?,
            ),
            MaterialType::Glass => Dielectric::new(
                self.texture
                    .as_ref()
                    .ok_or("Glass expects texture")?
                    .create()?,
                get_f64_opt(&self.data, "index_of_refraction")?,
            ),
            MaterialType::Light => DiffuseLight::new_from_vec3(get_col_opt(&self.data)?),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum TextureType {
    Color,
    Image,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Texture {
    r#type: TextureType,
    data: HashMap<String, Value>,
}

impl Creator<Textures> for Texture {
    fn create(&self) -> Result<Textures, Box<dyn Error>> {
        Ok(match self.r#type {
            TextureType::Color => SolidColor::new_from_vec3(get_col(&self.data)?),
            TextureType::Image => ImageTexture::load(get_str(&self.data, "file")?)?,
        })
    }
}

fn get_f64_opt(map: &Option<HashMap<String, Value>>, key: &str) -> Result<f64, Box<dyn Error>> {
    get_f64(map.as_ref().expect("expected data"), key)
}

fn get_f64(map: &HashMap<String, Value>, key: &str) -> Result<f64, Box<dyn Error>> {
    map.get(key)
        .ok_or(format!("expected key {}", key))?
        .as_f64()
        .ok_or(Box::try_from(format!("key {} is not a number", key)).unwrap())
}

fn get_str_opt<'a>(
    map: &'a Option<HashMap<String, Value>>,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    get_str(map.as_ref().expect("expected data"), key)
}

fn get_str<'a>(map: &'a HashMap<String, Value>, key: &str) -> Result<&'a str, Box<dyn Error>> {
    map.get(key)
        .ok_or(format!("expected key {}", key))?
        .as_str()
        .ok_or(Box::try_from(format!("key {} is not a string", key)).unwrap())
}

fn get_pos_opt(
    map: &Option<HashMap<String, Value>>,
) -> Result<solstrale::geo::vec3::Vec3, Box<dyn Error>> {
    Ok(solstrale::geo::vec3::Vec3::new(
        get_f64_opt(map, "x")?,
        get_f64_opt(map, "y")?,
        get_f64_opt(map, "z")?,
    ))
}

fn get_col(map: &HashMap<String, Value>) -> Result<solstrale::geo::vec3::Vec3, Box<dyn Error>> {
    Ok(solstrale::geo::vec3::Vec3::new(
        get_f64(map, "r")?,
        get_f64(map, "g")?,
        get_f64(map, "b")?,
    ))
}

fn get_col_opt(
    map: &Option<HashMap<String, Value>>,
) -> Result<solstrale::geo::vec3::Vec3, Box<dyn Error>> {
    Ok(solstrale::geo::vec3::Vec3::new(
        get_f64_opt(map, "r")?,
        get_f64_opt(map, "g")?,
        get_f64_opt(map, "b")?,
    ))
}

#[cfg(test)]
mod test {
    use crate::scene_model::{
        CameraConfig, Hittable, HittableType, Material, MaterialType, PostProcessorType,
        RenderConfig, Scene, ShaderType, Texture, TextureType, Vec3,
    };
    use serde_yaml::{Number, Value};
    use std::collections::HashMap;

    #[test]
    fn serialize() {
        let scene = Scene {
            world: Hittable {
                r#type: HittableType::List,
                material: None,
                data: None,
                children: Some(vec![Hittable {
                    r#type: HittableType::Sphere,
                    material: Some(Material {
                        r#type: MaterialType::Lambertian,
                        texture: Some(Texture {
                            r#type: TextureType::Color,
                            data: Default::default(),
                        }),
                        data: None,
                    }),
                    data: Some(HashMap::from([(
                        "radius".to_owned(),
                        Value::Number(Number::from(1.)),
                    )])),
                    children: None,
                }]),
            },
            camera: CameraConfig {
                vertical_fov_degrees: 0.0,
                aperture_size: 0.0,
                focus_distance: 0.0,
                look_from: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                look_at: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            background_color: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            render_configuration: RenderConfig {
                samples_per_pixel: 50,
                shader: ShaderType::PathTracing,
                post_processor: Some(PostProcessorType::Oidn),
            },
        };

        let yaml = serde_yaml::to_string(&scene).unwrap();
        assert_eq!(
            "render_configuration:
  samples_per_pixel: 50
  shader: PathTracing
  post_processor: Oidn
background_color:
  x: 0.0
  y: 0.0
  z: 0.0
camera:
  vertical_fov_degrees: 0.0
  aperture_size: 0.0
  focus_distance: 0.0
  look_from:
    x: 0.0
    y: 0.0
    z: 0.0
  look_at:
    x: 0.0
    y: 0.0
    z: 0.0
world:
  type: List
  children:
  - type: Sphere
    material:
      type: Lambertian
      texture:
        type: Color
        data: {}
    data:
      radius: 1.0
",
            yaml
        );

        let de_scene: Scene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene, de_scene);
    }
}
