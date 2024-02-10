use crate::model::FieldType::Normal;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::material::texture::{ImageMap, Textures};
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub file: String,
}

impl Creator<Textures> for Image {
    fn create(&self, _: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        ImageMap::load(self.file.as_ref())
    }
}

impl HelpDocumentation for Image {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description:
                "A texture where the colors for a coordinate is looked up from an image file"
                    .to_string(),
            fields: HashMap::from([(
                "file".to_string(),
                FieldInfo::new_simple(
                    "Path to the image file",
                    Normal,
                    "An absolute path to the texture image file",
                ),
            )]),
        }
    }
}
