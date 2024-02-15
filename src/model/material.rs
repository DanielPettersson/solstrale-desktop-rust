use crate::model::blend::Blend;
use crate::model::glass::Glass;
use crate::model::lambertian::Lambertian;
use crate::model::light::Light;
use crate::model::metal::Metal;
use crate::model::FieldType::Optional;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
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
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        match self {
            Material {
                lambertian: Some(l),
                glass: None,
                metal: None,
                light: None,
                blend: None,
            } => l.create(ctx),
            Material {
                lambertian: None,
                glass: Some(g),
                metal: None,
                light: None,
                blend: None,
            } => g.create(ctx),
            Material {
                lambertian: None,
                glass: None,
                metal: Some(m),
                light: None,
                blend: None,
            } => m.create(ctx),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: Some(l),
                blend: None,
            } => l.create(ctx),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: None,
                blend: Some(b),
            } => b.create(ctx),
            Material {
                lambertian: None,
                glass: None,
                metal: None,
                light: None,
                blend: None,
            } => Lambertian::default().create(ctx),
            _ => Err(From::from(ModelError::new(
                "Material should have max a single field defined",
            ))),
        }
    }
}

impl HelpDocumentation for Material {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
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
                        Lambertian::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "glass".to_string(),
                    FieldInfo::new(
                        "A dielectric material which has a glass-like appearance",
                        Optional,
                        Glass::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "metal".to_string(),
                    FieldInfo::new(
                        "A reflective material that gives a metallic appearance",
                        Optional,
                        Metal::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "light".to_string(),
                    FieldInfo::new(
                        "A material that emits light",
                        Optional,
                        Light::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "blend".to_string(),
                    FieldInfo::new(
                        "A material that is a blend of two underlying materials",
                        Optional,
                        Blend::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
