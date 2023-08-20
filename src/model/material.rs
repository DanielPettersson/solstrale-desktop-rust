use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use crate::model::{Creator, ModelError};
use crate::model::glass::Glass;
use crate::model::lambertian::Lambertian;
use crate::model::light::Light;
use crate::model::metal::Metal;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Material {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lambertian: Option<Lambertian>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glass: Option<Glass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metal: Option<Metal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<Light>,
}

impl Creator<Materials> for Material {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        match self {
            Material {
                lambertian: Some(l),
                glass: None,
                metal: None,
                light: None,
            } => l.create(),
            Material {
                lambertian: None,
                glass: Some(g),
                metal: None,
                light: None,
            } => g.create(),
            Material {
                lambertian: None,
                glass: None,
                metal: Some(m),
                light: None,
            } => m.create(),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: Some(l),
            } => l.create(),
            _ => Err(Box::try_from(ModelError::new(
                "Material should have single field defined",
            ))
                .unwrap()),
        }
    }
}
