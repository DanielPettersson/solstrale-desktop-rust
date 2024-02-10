use crate::model::image::Image;
use crate::model::rgb::Rgb;
use crate::model::FieldType::Optional;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};
use serde::{Deserialize, Serialize};
use solstrale::material::texture::{SolidColor, Textures};
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
}

impl Creator<Textures> for Texture {
    fn create(&self, ctx: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        match self {
            Texture {
                color: Some(c),
                image: None,
            } => Ok(SolidColor::new(c.r, c.g, c.b)),
            Texture {
                color: None,
                image: Some(im),
            } => im.create(ctx),
            _ => Err(From::from(ModelError::new(
                "Texture should have single field defined",
            ))),
        }
    }
}

impl HelpDocumentation for Texture {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A texture defines the color of hittable objects".to_string(),
            fields: HashMap::from([
                (
                    "color".to_string(),
                    FieldInfo::new(
                        "Simple one-color texture",
                        Optional,
                        Rgb::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "image".to_string(),
                    FieldInfo::new(
                        "Texture where the color of each coordinate is read from an image file",
                        Optional,
                        Image::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
