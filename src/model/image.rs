use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::texture::{ImageMap, Textures};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::Normal;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub file: String,
}

impl Creator<Textures> for Image {
    fn create(&self) -> Result<Textures, Box<dyn Error>> {
        ImageMap::load(self.file.as_ref())
    }
}

impl HelpDocumentation for Image {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Image>>".to_string(),
            fields: HashMap::from([
                ("file".to_string(), FieldInfo::new_simple("<<file>>", Normal, "<<String>>")),
            ]),
        }
    }
}
