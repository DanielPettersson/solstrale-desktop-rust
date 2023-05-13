use std::error::Error;

use derive_more::Display;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;
use solstrale::hittable::hittable_list::HittableList;
use solstrale::hittable::obj_model::load_obj_model;
use solstrale::hittable::Hittable as HittableTrait;
use solstrale::hittable::Hittables;
use solstrale::material::texture::{ImageTexture, SolidColor, Textures};
use solstrale::material::{Dielectric, DiffuseLight, Materials};
use solstrale::post::OidnPostProcessor;
use solstrale::renderer::shader::{
    AlbedoShader, NormalShader, PathTracingShader, Shaders, SimpleShader,
};

static MODEL_CACHE: Lazy<Cache<ModelKey, Result<Hittables, ModelError>>> =
    Lazy::new(|| Cache::new(10));

#[derive(PartialEq, Eq, Hash)]
struct ModelKey {
    path: String,
    filename: String,
    scale: i32,
    x: i32,
    y: i32,
    z: i32,
}

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

impl ModelKey {
    fn new(m: &Model) -> Self {
        Self {
            path: m.path.to_string(),
            filename: m.name.to_string(),
            scale: (m.scale * 100.) as i32,
            x: (m.pos.x * 100.) as i32,
            y: (m.pos.y * 100.) as i32,
            z: (m.pos.z * 100.) as i32,
        }
    }
}

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
    background_color: Pos,
    camera: CameraConfig,
    world: Vec<Hittable>,
}

impl Creator<solstrale::renderer::Scene> for Scene {
    fn create(&self) -> Result<solstrale::renderer::Scene, Box<dyn Error>> {
        let mut list = HittableList::new();
        for child in self.world.iter() {
            list.add(child.create()?)
        }

        Ok(solstrale::renderer::Scene {
            world: list,
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
    look_from: Pos,
    look_at: Pos,
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
struct Shader {
    #[serde(skip_serializing_if = "Option::is_none")]
    path_tracing: Option<PathTracing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    simple: Option<NoParamShader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    albedo: Option<NoParamShader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    normal: Option<NoParamShader>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PathTracing {
    max_depth: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct NoParamShader {}

impl Creator<Shaders> for Shader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        match self {
            Shader {
                path_tracing: Some(p),
                simple: None,
                albedo: None,
                normal: None,
            } => Ok(PathTracingShader::new(p.max_depth)),
            Shader {
                path_tracing: None,
                simple: Some(_),
                albedo: None,
                normal: None,
            } => Ok(SimpleShader::new()),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: Some(_),
                normal: None,
            } => Ok(AlbedoShader::new()),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: None,
                normal: Some(_),
            } => Ok(NormalShader::new()),
            _ => Err(
                Box::try_from(ModelError::new("Shader should have single field defined")).unwrap(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum PostProcessorType {
    Oidn,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct RenderConfig {
    samples_per_pixel: u32,
    shader: Shader,
    #[serde(skip_serializing_if = "Option::is_none")]
    post_processor: Option<PostProcessorType>,
}

impl Creator<solstrale::renderer::RenderConfig> for RenderConfig {
    fn create(&self) -> Result<solstrale::renderer::RenderConfig, Box<dyn Error>> {
        Ok(solstrale::renderer::RenderConfig {
            samples_per_pixel: self.samples_per_pixel,
            shader: self.shader.create()?,
            post_processor: self.post_processor.as_ref().map(|p| match p {
                PostProcessorType::Oidn => OidnPostProcessor::new(),
            }),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Pos {
    x: f64,
    y: f64,
    z: f64,
}

impl Creator<Vec3> for Pos {
    fn create(&self) -> Result<Vec3, Box<dyn Error>> {
        Ok(Vec3::new(self.x, self.y, self.z))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Hittable {
    #[serde(skip_serializing_if = "Option::is_none")]
    list: Option<List>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sphere: Option<Sphere>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<Model>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quad: Option<Quad>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rotation_y: Option<RotationY>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct List {
    children: Vec<Hittable>,
}

impl Creator<Hittables> for List {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        let mut list = HittableList::new();
        for child in self.children.iter() {
            list.add(child.create()?)
        }
        Ok(list)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Sphere {
    center: Pos,
    radius: f64,
    material: Material,
}

impl Creator<Hittables> for Sphere {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::sphere::Sphere::new(
            self.center.create()?,
            self.radius,
            self.material.create()?,
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Model {
    path: String,
    name: String,
    pos: Pos,
    scale: f64,
    angle_y: f64,
}

impl Creator<Hittables> for Model {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        let key = ModelKey::new(self);

        let pos = self.pos.create()?;
        let model = MODEL_CACHE.get_with(key, || {
            load_obj_model(&self.path, &self.name, self.scale, pos)
                .map_err(|err| ModelError::new_from_err(err))
        })?;

        Ok(if self.angle_y == 0. {
            model
        } else {
            solstrale::hittable::rotation_y::RotationY::new(model, self.angle_y)
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Quad {
    q: Pos,
    u: Pos,
    v: Pos,
    material: Material,
}

impl Creator<Hittables> for Quad {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::quad::Quad::new(
            self.q.create()?,
            self.u.create()?,
            self.v.create()?,
            self.material.create()?,
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct RotationY {
    child: Box<Hittable>,
    angle: f64,
}

impl Creator<Hittables> for RotationY {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::rotation_y::RotationY::new(
            self.child.create()?,
            self.angle,
        ))
    }
}

impl Creator<Hittables> for Hittable {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        match self {
            Hittable {
                list: Some(l),
                sphere: None,
                model: None,
                quad: None,
                rotation_y: None,
            } => l.create(),
            Hittable {
                list: None,
                sphere: Some(s),
                model: None,
                quad: None,
                rotation_y: None,
            } => s.create(),
            Hittable {
                list: None,
                sphere: None,
                model: Some(m),
                quad: None,
                rotation_y: None,
            } => m.create(),
            Hittable {
                list: None,
                sphere: None,
                model: None,
                quad: Some(q),
                rotation_y: None,
            } => q.create(),
            Hittable {
                list: None,
                sphere: None,
                model: None,
                quad: None,
                rotation_y: Some(ry),
            } => ry.create(),
            _ => Err(
                Box::try_from(ModelError::new("Hittable should have single field defined"))
                    .unwrap(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Material {
    #[serde(skip_serializing_if = "Option::is_none")]
    lambertian: Option<Lambertian>,
    #[serde(skip_serializing_if = "Option::is_none")]
    glass: Option<Glass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metal: Option<Metal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    light: Option<Light>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Lambertian {
    texture: Texture,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Glass {
    texture: Texture,
    index_of_refraction: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Metal {
    texture: Texture,
    fuzz: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Light {
    color: Rgb,
}

impl Creator<Materials> for Material {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        match self {
            Material {
                lambertian: Some(l),
                glass: None,
                metal: None,
                light: None,
            } => Ok(solstrale::material::Lambertian::new(l.texture.create()?)),
            Material {
                lambertian: None,
                glass: Some(g),
                metal: None,
                light: None,
            } => Ok(Dielectric::new(g.texture.create()?, g.index_of_refraction)),
            Material {
                lambertian: None,
                glass: None,
                metal: Some(m),
                light: None,
            } => Ok(solstrale::material::Metal::new(m.texture.create()?, m.fuzz)),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: Some(l),
            } => Ok(DiffuseLight::new(l.color.r, l.color.g, l.color.b)),
            _ => Err(
                Box::try_from(ModelError::new("Material should have single field defined"))
                    .unwrap(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Texture {
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Image {
    file: String,
}

impl Creator<Textures> for Texture {
    fn create(&self) -> Result<Textures, Box<dyn Error>> {
        match self {
            Texture {
                color: Some(c),
                image: None,
            } => Ok(SolidColor::new(c.r, c.g, c.b)),
            Texture {
                color: None,
                image: Some(im),
            } => ImageTexture::load(im.file.as_ref()),
            _ => Err(
                Box::try_from(ModelError::new("Texture should have single field defined")).unwrap(),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::scene_model::*;

    #[test]
    fn serialize() {
        let scene = Scene {
            world: vec![Hittable {
                list: None,
                sphere: Some(Sphere {
                    center: Pos {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    radius: 1.0,
                    material: Material {
                        lambertian: Some(Lambertian {
                            texture: Texture {
                                color: Some(Rgb {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                }),
                                image: None,
                            },
                        }),
                        glass: None,
                        metal: None,
                        light: None,
                    },
                }),
                model: None,
                quad: None,
                rotation_y: None,
            }],
            camera: CameraConfig {
                vertical_fov_degrees: 0.0,
                aperture_size: 0.0,
                focus_distance: 0.0,
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
            },
            background_color: Pos {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            render_configuration: RenderConfig {
                samples_per_pixel: 50,
                shader: Shader {
                    path_tracing: Some(PathTracing { max_depth: 50 }),
                    simple: None,
                    albedo: None,
                    normal: None,
                },
                post_processor: Some(PostProcessorType::Oidn),
            },
        };

        let yaml = serde_yaml::to_string(&scene).unwrap();
        assert_eq!(
            "render_configuration:
  samples_per_pixel: 50
  shader:
    path_tracing:
      max_depth: 50
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
- sphere:
    center:
      x: 0.0
      y: 0.0
      z: 0.0
    radius: 1.0
    material:
      lambertian:
        texture:
          color:
            r: 0.0
            g: 0.0
            b: 0.0
",
            yaml
        );

        let de_scene: Scene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene, de_scene);
    }
}
