use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use crate::model::FieldType::Optional;
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

impl HelpDocumentation for Material {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Material>>".to_string(),
            fields: HashMap::from([
                ("lambertian".to_string(), FieldInfo::new("<<lambertian>>", Optional, Lambertian::get_documentation_structure())),
                ("glass".to_string(), FieldInfo::new("<<glass>>", Optional, Glass::get_documentation_structure())),
                ("metal".to_string(), FieldInfo::new("<<metal>>", Optional, Metal::get_documentation_structure())),
                ("light".to_string(), FieldInfo::new("<<light>>", Optional, Light::get_documentation_structure())),
            ]),
        }
    }
}
