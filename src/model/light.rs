use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::{DiffuseLight, Materials};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{Normal, Optional};
use crate::model::rgb::Rgb;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Light {
    pub color: Rgb,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation_half_length: Option<f64>,
}

impl Creator<Materials> for Light {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        Ok(DiffuseLight::new(
            self.color.r,
            self.color.g,
            self.color.b,
            self.attenuation_half_length,
        ))
    }
}

impl HelpDocumentation for Light {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Light>>".to_string(),
            fields: HashMap::from([
                ("color".to_string(), FieldInfo::new("<<color>>", Normal, Rgb::get_documentation_structure())),
                ("attenuation_half_length".to_string(), FieldInfo::new_simple("<<attenuation_half_length>>", Optional, "Option<f64>")),
            ]),
        }
    }
}