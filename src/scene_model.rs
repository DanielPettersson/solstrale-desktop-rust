use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Scene {
    /// World is the hittable objects in the scene
    pub world: Hittable,
    /// A camera for defining the view of the world
    pub camera: CameraConfig,
    /// Background color of the scene
    pub background_color: Vec3,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CameraConfig {
    /// Vertical field of view in degrees
    pub vertical_fov_degrees: f64,
    /// Radius of the lens of the camera, affects the depth of field
    pub aperture_size: f64,
    /// Distance where the lens is focused
    pub focus_distance: f64,
    /// Point where the camera is located
    pub look_from: Vec3,
    /// Point where the camera is looking
    pub look_at: Vec3,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Vec3 {
    /// x position
    pub x: f64,
    /// y position
    pub y: f64,
    /// z position
    pub z: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum HittableType {
    List,
    Sphere,
    Triangle,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Hittable {
    pub r#type: HittableType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Hittable>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum MaterialType {
    Lambertian,
    Light,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Material {
    pub r#type: MaterialType,
}

#[cfg(test)]
mod test {
    use crate::scene_model::{
        CameraConfig, Hittable, HittableType, Material, MaterialType, Scene, Vec3,
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
        };

        let yaml = serde_yaml::to_string(&scene).unwrap();
        assert_eq!(
            "world:
  type: List
  children:
  - type: Sphere
    material:
      type: Lambertian
    data:
      radius: 1.0
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
background_color:
  x: 0.0
  y: 0.0
  z: 0.0
",
            yaml
        );

        let de_scene: Scene = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(scene, de_scene);
    }
}
