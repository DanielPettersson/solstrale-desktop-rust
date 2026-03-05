use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::texture::{Textures, load_normal_texture};

use crate::model::FieldType::Normal;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NormalTexture {
    pub file: String,
}

impl Creator<Textures> for NormalTexture {
    fn create(&self, _: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        load_normal_texture(self.file.as_ref()).map(|t| t.into())
    }
}

impl HelpDocumentation for NormalTexture {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A texture for the normals of a hittable".to_string(),
            fields: HashMap::from([(
                "file".to_string(),
                FieldInfo::new_simple(
                    "A normal map image file",
                    Normal,
                    "The absolute file path to an image file for the normals, can be either a height map or a normal map.",
                ),
            )]),
        }
    }
}
