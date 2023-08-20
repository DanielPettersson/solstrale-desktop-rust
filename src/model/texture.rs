use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::texture::{SolidColor, Textures};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use crate::model::FieldType::Optional;
use crate::model::image::Image;
use crate::model::rgb::Rgb;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
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
            } => im.create(),
            _ => Err(
                Box::try_from(ModelError::new("Texture should have single field defined"))
                    .unwrap(),
            ),
        }
    }
}

impl HelpDocumentation for Texture {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Texture>>".to_string(),
            fields: HashMap::from([
                ("color".to_string(), FieldInfo::new("<<color>>", Optional, Rgb::get_documentation_structure())),
                ("image".to_string(), FieldInfo::new("<<image>>", Optional, Image::get_documentation_structure())),
            ]),
        }
    }
}
