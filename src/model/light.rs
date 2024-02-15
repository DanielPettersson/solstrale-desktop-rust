use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::{DiffuseLight, Materials};

use crate::model::rgb::Rgb;
use crate::model::FieldType::Optional;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Light {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation_half_length: Option<f64>,
}

impl Creator<Materials> for Light {
    fn create(&self, _: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        let c = self.color.unwrap_or(Rgb::new(15.0, 15.0, 15.0));
        Ok(DiffuseLight::new(
            c.r,
            c.g,
            c.b,
            self.attenuation_half_length,
        ))
    }
}

impl HelpDocumentation for Light {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A material that emits light".to_string(),
            fields: HashMap::from([
                ("color".to_string(), FieldInfo::new(
                    "The color of the light being emitted. The intensity of color is normally way over 1. Defaults to 15, 15, 15",
                    Optional,
                    Rgb::get_documentation_structure(depth + 1),
                )),
                ("attenuation_half_length".to_string(), FieldInfo::new_simple(
                    "Attenuation is the amount of intensity lost the further away from the light source",
                    Optional,
                    "The length at which the light has lost half it's intensity",
                )),
            ]),
        }
    }
}
