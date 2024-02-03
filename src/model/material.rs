use crate::model::glass::Glass;
use crate::model::lambertian::Lambertian;
use crate::model::light::Light;
use crate::model::metal::Metal;
use crate::model::FieldType::Optional;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use std::collections::HashMap;
use std::error::Error;
use crate::model::blend::Blend;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend: Option<Box<Blend>>,

}

impl Creator<Materials> for Material {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        match self {
            Material {
                lambertian: Some(l),
                glass: None,
                metal: None,
                light: None,
                blend: None,
            } => l.create(),
            Material {
                lambertian: None,
                glass: Some(g),
                metal: None,
                light: None,
                blend: None,
            } => g.create(),
            Material {
                lambertian: None,
                glass: None,
                metal: Some(m),
                light: None,
                blend: None,
            } => m.create(),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: Some(l),
                blend: None,
            } => l.create(),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: None,
                blend: Some(b),
            } => b.create(),
            _ => Err(
                From::from(ModelError::new("Material should have single field defined")),
            ),
        }
    }
}

impl HelpDocumentation for Material {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description:
                "A material gives hittable objects it's looks as they scatter the light differently"
                    .to_string(),
            fields: HashMap::from([
                (
                    "lambertian".to_string(),
                    FieldInfo::new(
                        "A material with the appearance of a matte surface",
                        Optional,
                        Lambertian::get_documentation_structure(),
                    ),
                ),
                (
                    "glass".to_string(),
                    FieldInfo::new(
                        "A dielectric material which has a glass-like appearance",
                        Optional,
                        Glass::get_documentation_structure(),
                    ),
                ),
                (
                    "metal".to_string(),
                    FieldInfo::new(
                        "A reflective material that gives a metallic appearance",
                        Optional,
                        Metal::get_documentation_structure(),
                    ),
                ),
                (
                    "light".to_string(),
                    FieldInfo::new(
                        "A material that emits light",
                        Optional,
                        Light::get_documentation_structure(),
                    ),
                ),
            ]),
        }
    }
}
