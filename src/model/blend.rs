use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::Materials;

use crate::model::material::Material;
use crate::model::FieldType::{Normal, Optional};
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Blend {
    pub first: Material,
    pub second: Material,
    pub blend_factor: Option<f64>,
}

impl Creator<Materials> for Blend {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Blend::new(
            self.first.create(ctx)?,
            self.second.create(ctx)?,
            self.blend_factor.unwrap_or(0.5),
        ))
    }
}

impl HelpDocumentation for Blend {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        if depth < 5 {
            DocumentationStructure {
                description: "A blend of two underlying materials".to_string(),
                fields: HashMap::from([
                    ("first".to_string(), FieldInfo::new(
                        "The first underlying material that will be blended",
                        Normal,
                        Material::get_documentation_structure(depth + 1)
                    )),
                    ("second".to_string(), FieldInfo::new(
                        "The second underlying material that will be blended",
                        Normal,
                        Material::get_documentation_structure(depth + 1)
                    )),
                    ("blend_factor".to_string(), FieldInfo::new_simple(
                        "A factor of how much each of 'first' and 'second' will be blended",
                        Optional,
                        "For example: 0 uses only 'first', 1 uses only 'second' and 0.5 uses equal amount of both materials. Defaults to 0.5"
                    )),
                ]),
            }
        } else {
            DocumentationStructure {
                description: "A blend of two underlying materials".to_string(),
                fields: HashMap::new(),
            }
        }
    }
}
