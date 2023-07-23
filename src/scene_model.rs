use std::boxed::Box as StdBox;
use std::error::Error;

use derive_more::Display;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solstrale::geo::transformation::{
    RotationX, RotationY, RotationZ, Scale, Transformations, Transformer, Translation,
};
use solstrale::geo::vec3::Vec3;
use solstrale::hittable::Hittable as HittableTrait;
use solstrale::hittable::HittableList;
use solstrale::hittable::Hittables;
use solstrale::loader::Loader;
use solstrale::loader::obj::Obj;
use solstrale::material::{Dielectric, DiffuseLight, Materials};
use solstrale::material::texture::{ImageMap, SolidColor, Textures};
use solstrale::post::{OidnPostProcessor, PostProcessors};
use solstrale::renderer::Scene;
use solstrale::renderer::shader::{
    AlbedoShader, NormalShader, PathTracingShader, Shaders, SimpleShader,
};

static MODEL_CACHE: Lazy<Cache<String, Result<Hittables, ModelError>>> =
    Lazy::new(|| Cache::new(4));

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

    fn new_from_err(err: StdBox<dyn Error>) -> Self {
        Self {
            message: format!("{}", err),
        }
    }
}

impl Error for ModelError {}

pub fn create_scene(yaml: &str) -> Result<SceneModel, StdBox<dyn Error>> {
    let scene: SceneModel = serde_yaml::from_str(yaml)?;
    Ok(scene)
}

pub trait Creator<T> {
    fn create(&self) -> Result<T, StdBox<dyn Error>>;
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SceneModel {
    render_configuration: RenderConfig,
    background_color: Pos,
    camera: CameraConfig,
    world: Vec<Hittable>,
}

impl Creator<Scene> for SceneModel {
    fn create(&self) -> Result<Scene, StdBox<dyn Error>> {
        let mut list = HittableList::new();
        for child in self.world.iter() {
            list.add(child.create()?)
        }

        Ok(Scene {
            world: list,
            camera: self.camera.create()?,
            background_color: self.background_color.create()?,
            render_config: self.render_configuration.create()?,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct CameraConfig {
    vertical_fov_degrees: f64,
    aperture_size: f64,
    look_from: Pos,
    look_at: Pos,
}

impl Creator<solstrale::camera::CameraConfig> for CameraConfig {
    fn create(&self) -> Result<solstrale::camera::CameraConfig, StdBox<dyn Error>> {
        Ok(solstrale::camera::CameraConfig {
            vertical_fov_degrees: self.vertical_fov_degrees,
            aperture_size: self.aperture_size,
            look_from: self.look_from.create()?,
            look_at: self.look_at.create()?,
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct PathTracing {
    max_depth: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct NoParamShader {}

impl Creator<Shaders> for Shader {
    fn create(&self) -> Result<Shaders, StdBox<dyn Error>> {
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
                StdBox::try_from(ModelError::new("Shader should have single field defined"))
                    .unwrap(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct PostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    bloom: Option<BloomPostProcessor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    denoise: Option<NoParamPostProcessor>,
}

impl Creator<PostProcessors> for PostProcessor {
    fn create(&self) -> Result<PostProcessors, StdBox<dyn Error>> {
        match self {
            PostProcessor { bloom: Some(b), denoise: None } => b.create(),
            PostProcessor { bloom: None, denoise: Some(_) } => Ok(OidnPostProcessor::new()),
            _ => Err(StdBox::try_from(ModelError::new(
                "PostProcessor should have single field defined",
            ))
            .unwrap()),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct BloomPostProcessor {
    kernel_size_fraction: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_intensity: Option<f64>,
}

impl Creator<PostProcessors> for BloomPostProcessor {
    fn create(&self) -> Result<PostProcessors, StdBox<dyn Error>> {
        Ok(solstrale::post::BloomPostProcessor::new(
            self.kernel_size_fraction,
            self.threshold,
            self.max_intensity)?
        )
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct NoParamPostProcessor {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct RenderConfig {
    samples_per_pixel: u32,
    shader: Shader,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    post_processors: Vec<PostProcessor>,
}

impl Creator<solstrale::renderer::RenderConfig> for RenderConfig {
    fn create(&self) -> Result<solstrale::renderer::RenderConfig, StdBox<dyn Error>> {

        let mut post_processors: Vec<PostProcessors> = Vec::new();

        for p in &self.post_processors {
            post_processors.push(p.create()?);
        }

        Ok(solstrale::renderer::RenderConfig {
            samples_per_pixel: self.samples_per_pixel,
            shader: self.shader.create()?,
            post_processors,
        })
    }
}

#[derive(PartialEq, Debug)]
struct Pos {
    x: f64,
    y: f64,
    z: f64,
}

static X: &str = "x";
static Y: &str = "y";
static Z: &str = "z";

impl Serialize for Pos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}, {}, {}", self.x, self.y, self.z))
    }
}

impl<'de> Deserialize<'de> for Pos {
    fn deserialize<D>(deserializer: D) -> Result<Pos, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut split = s.split(',');
        let x = parse_option::<D>(split.next(), X)?;
        let y = parse_option::<D>(split.next(), Y)?;
        let z = parse_option::<D>(split.next(), Z)?;
        Ok(Pos { x, y, z })
    }
}

impl From<&Pos> for Vec3 {
    fn from(value: &Pos) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

fn parse_option<'de, D>(a: Option<&str>, expected_field: &'static str) -> Result<f64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    a.ok_or(serde::de::Error::missing_field(expected_field))?
        .trim()
        .parse::<f64>()
        .map_err(serde::de::Error::custom)
}

impl Creator<Vec3> for Pos {
    fn create(&self) -> Result<Vec3, StdBox<dyn Error>> {
        Ok(Vec3::new(self.x, self.y, self.z))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
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
    r#box: Option<Box>,
    #[serde(skip_serializing_if = "Option::is_none")]
    constant_medium: Option<ConstantMedium>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct List {
    children: Vec<Hittable>,
}

impl Creator<Hittables> for List {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        let mut list = HittableList::new();
        for child in self.children.iter() {
            list.add(child.create()?)
        }
        Ok(list)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Sphere {
    center: Pos,
    radius: f64,
    material: Material,
}

impl Creator<Hittables> for Sphere {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        Ok(solstrale::hittable::Sphere::new(
            self.center.create()?,
            self.radius,
            self.material.create()?,
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Model {
    path: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    material: Option<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    transformations: Vec<Transformation>,
}

impl Creator<Hittables> for Model {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        let material = self.material.as_ref().map_or(
            Ok(solstrale::material::Lambertian::new(
                SolidColor::new(1., 1., 1.),
                None,
            )),
            |m| m.create(),
        )?;
        let transformation = create_transformation(&self.transformations)?;

        let key = format!("{:?}", self);
        let model_result = MODEL_CACHE.get_with(key.to_owned(), || {
            Obj::new(&self.path, &self.name)
                .load(&transformation, Some(material))
                .map_err(|err| ModelError::new_from_err(err))
        });

        match model_result {
            Ok(model) => Ok(model),
            Err(err) => {
                MODEL_CACHE.remove(&key);
                Err(StdBox::new(err))
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Quad {
    q: Pos,
    u: Pos,
    v: Pos,
    material: Material,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    transformations: Vec<Transformation>,
}

impl Creator<Hittables> for Quad {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        Ok(solstrale::hittable::Quad::new(
            self.q.create()?,
            self.u.create()?,
            self.v.create()?,
            self.material.create()?,
            &create_transformation(&self.transformations)?,
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Box {
    a: Pos,
    b: Pos,
    material: Material,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    transformations: Vec<Transformation>,
}

impl Creator<Hittables> for Box {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        Ok(solstrale::hittable::Quad::new_box(
            self.a.create()?,
            self.b.create()?,
            self.material.create()?,
            &create_transformation(&self.transformations)?,
        ))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct ConstantMedium {
    child: StdBox<Hittable>,
    density: f64,
    color: Rgb,
}

impl Creator<Hittables> for ConstantMedium {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        Ok(solstrale::hittable::ConstantMedium::new(
            self.child.create()?,
            self.density,
            self.color.into(),
        ))
    }
}

impl Creator<Hittables> for Hittable {
    fn create(&self) -> Result<Hittables, StdBox<dyn Error>> {
        match self {
            Hittable {
                list: Some(l),
                sphere: None,
                model: None,
                quad: None,
                r#box: None,
                constant_medium: None,
            } => l.create(),
            Hittable {
                list: None,
                sphere: Some(s),
                model: None,
                quad: None,
                r#box: None,
                constant_medium: None,
            } => s.create(),
            Hittable {
                list: None,
                sphere: None,
                model: Some(m),
                quad: None,
                r#box: None,
                constant_medium: None,
            } => m.create(),
            Hittable {
                list: None,
                sphere: None,
                model: None,
                quad: Some(q),
                r#box: None,
                constant_medium: None,
            } => q.create(),
            Hittable {
                list: None,
                sphere: None,
                model: None,
                quad: None,
                r#box: Some(b),
                constant_medium: None,
            } => b.create(),
            Hittable {
                list: None,
                sphere: None,
                model: None,
                quad: None,
                r#box: None,
                constant_medium: Some(cm),
            } => cm.create(),
            _ => Err(StdBox::try_from(ModelError::new(
                "Hittable should have single field defined",
            ))
            .unwrap()),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct Lambertian {
    albedo: Texture,
    #[serde(skip_serializing_if = "Option::is_none")]
    normal: Option<Texture>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Glass {
    albedo: Texture,
    #[serde(skip_serializing_if = "Option::is_none")]
    normal: Option<Texture>,
    index_of_refraction: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Metal {
    albedo: Texture,
    #[serde(skip_serializing_if = "Option::is_none")]
    normal: Option<Texture>,
    fuzz: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Light {
    color: Rgb,
    #[serde(skip_serializing_if = "Option::is_none")]
    attenuation_half_length: Option<f64>,
}

impl Creator<Materials> for Material {
    fn create(&self) -> Result<Materials, StdBox<dyn Error>> {
        match self {
            Material {
                lambertian: Some(l),
                glass: None,
                metal: None,
                light: None,
            } => Ok(solstrale::material::Lambertian::new(
                l.albedo.create()?,
                match l.normal.as_ref() {
                    None => None,
                    Some(n) => Some(n.create()?),
                },
            )),
            Material {
                lambertian: None,
                glass: Some(g),
                metal: None,
                light: None,
            } => Ok(Dielectric::new(
                g.albedo.create()?,
                match g.normal.as_ref() {
                    None => None,
                    Some(n) => Some(n.create()?),
                },
                g.index_of_refraction,
            )),
            Material {
                lambertian: None,
                glass: None,
                metal: Some(m),
                light: None,
            } => Ok(solstrale::material::Metal::new(
                m.albedo.create()?,
                match m.normal.as_ref() {
                    None => None,
                    Some(n) => Some(n.create()?),
                },
                m.fuzz,
            )),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: Some(l),
            } => Ok(DiffuseLight::new(
                l.color.r,
                l.color.g,
                l.color.b,
                l.attenuation_half_length,
            )),
            _ => Err(StdBox::try_from(ModelError::new(
                "Material should have single field defined",
            ))
            .unwrap()),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Texture {
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Rgb {
    r: f64,
    g: f64,
    b: f64,
}

static R: &str = "r";
static G: &str = "g";
static B: &str = "b";

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}, {}, {}", self.r, self.g, self.b))
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D>(deserializer: D) -> Result<Rgb, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut split = s.split(',');
        let r = parse_option::<D>(split.next(), R)?;
        let g = parse_option::<D>(split.next(), G)?;
        let b = parse_option::<D>(split.next(), B)?;
        Ok(Rgb { r, g, b })
    }
}

impl From<Rgb> for Vec3 {
    fn from(value: Rgb) -> Self {
        Vec3::new(value.r, value.g, value.b)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Image {
    file: String,
}

impl Creator<Textures> for Texture {
    fn create(&self) -> Result<Textures, StdBox<dyn Error>> {
        match self {
            Texture {
                color: Some(c),
                image: None,
            } => Ok(SolidColor::new(c.r, c.g, c.b)),
            Texture {
                color: None,
                image: Some(im),
            } => ImageMap::load(im.file.as_ref()),
            _ => Err(
                StdBox::try_from(ModelError::new("Texture should have single field defined"))
                    .unwrap(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
struct Transformation {
    #[serde(skip_serializing_if = "Option::is_none")]
    translation: Option<Pos>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rotation_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rotation_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rotation_z: Option<f64>,
}

impl Creator<StdBox<dyn Transformer>> for Transformation {
    fn create(&self) -> Result<StdBox<dyn Transformer>, StdBox<dyn Error>> {
        match self {
            Transformation {
                translation: Some(p),
                scale: None,
                rotation_x: None,
                rotation_y: None,
                rotation_z: None,
            } => Ok(StdBox::new(Translation::new(p.into()))),
            Transformation {
                translation: None,
                scale: Some(s),
                rotation_x: None,
                rotation_y: None,
                rotation_z: None,
            } => Ok(StdBox::new(Scale::new(*s))),
            Transformation {
                translation: None,
                scale: None,
                rotation_x: Some(r),
                rotation_y: None,
                rotation_z: None,
            } => Ok(StdBox::new(RotationX::new(*r))),
            Transformation {
                translation: None,
                scale: None,
                rotation_x: None,
                rotation_y: Some(r),
                rotation_z: None,
            } => Ok(StdBox::new(RotationY::new(*r))),
            Transformation {
                translation: None,
                scale: None,
                rotation_x: None,
                rotation_y: None,
                rotation_z: Some(r),
            } => Ok(StdBox::new(RotationZ::new(*r))),
            _ => Err(StdBox::try_from(ModelError::new(
                "Transformation should have single field defined",
            ))
            .unwrap()),
        }
    }
}

fn create_transformation(
    transformations: &Vec<Transformation>,
) -> Result<Transformations, StdBox<dyn Error>> {
    let mut trans: Vec<StdBox<dyn Transformer>> = Vec::with_capacity(transformations.len());
    for t in transformations {
        trans.push(t.create()?);
    }
    Ok(Transformations::new(trans))
}

#[cfg(test)]
mod test {
    use crate::scene_model::*;

    #[test]
    fn serde() {
        let scene = SceneModel {
            world: vec![Hittable {
                list: None,
                sphere: None,
                model: None,
                quad: None,
                r#box: Some(Box {
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
                post_processors: vec!(
                    PostProcessor {
                        bloom: Some(BloomPostProcessor { kernel_size_fraction: 0.1, threshold: Some(1.5), max_intensity: None }),
                        denoise: None,
                    },
                    PostProcessor {
                        bloom: None,
                        denoise: Some(NoParamPostProcessor{}),
                    }
                ),
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
background_color: 0, 0, 0
camera:
  vertical_fov_degrees: 0.0
  aperture_size: 0.0
  look_from: 0, 0, 0
  look_at: 0, 0, 0
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

        let de_scene: SceneModel = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene, de_scene);
    }
}
