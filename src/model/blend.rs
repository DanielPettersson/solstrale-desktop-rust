use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::Materials;

use crate::model::material::Material;
use crate::model::texture::Texture;
use crate::model::FieldType::Normal;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Blend {
    pub a: Material,
    pub b: Material,
    pub blend_factor: f64,
}

impl Creator<Materials> for Blend {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Blend::new(
            self.a.create()?,
            self.b.create()?,
            self.blend_factor,
        ))
    }
}

impl HelpDocumentation for Blend {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "A blend of two underlying materials".to_string(),
            fields: HashMap::from([
                ("a".to_string(), FieldInfo::new(
                    "The first underlying material that will be blended",
                    Normal,
                    Material::get_documentation_structure()
                )),
                ("b".to_string(), FieldInfo::new(
                    "The second underlying material that will be blended",
                    Normal,
                    Texture::get_documentation_structure()
                )),
                ("blend_factor".to_string(), FieldInfo::new_simple(
                    "A factor of how much each of 'a' and 'b' will be blended",
                    Normal,
                    "For example: 0 uses only 'a', 1 uses only 'b' and 0.5 uses equal amount of both materials"
                )),
            ]),
        }
    }
}
