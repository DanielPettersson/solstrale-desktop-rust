use crate::model::FieldType::Optional;
use crate::model::image::Image;
use crate::model::one_of::one_of;
use crate::model::rgb::Rgb;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use solstrale::material::texture::{SolidColor, Textures};
use std::collections::HashMap;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Texture {
        color => Color(Rgb),
        image => Image(Image),
    }
    repr: TextureRepr, TextureReprRef;
    empty: Some(Texture::default());
}

impl Default for Texture {
    fn default() -> Self {
        Texture::Color(Rgb::new(0.8, 0.8, 0.8))
    }
}

impl Creator<Textures> for Texture {
    fn create(&self, ctx: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        match self {
            Texture::Color(c) => Ok(SolidColor::new(c.r, c.g, c.b).into()),
            Texture::Image(im) => im.create(ctx),
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
