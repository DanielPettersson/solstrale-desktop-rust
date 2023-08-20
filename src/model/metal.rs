use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{Normal, Optional};
use crate::model::texture::Texture;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Metal {
    pub albedo: Texture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<Texture>,
    pub fuzz: f64,
}

impl Creator<Materials> for Metal {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Metal::new(
            self.albedo.create()?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create()?),
            },
            self.fuzz,
        ))
    }
}

impl HelpDocumentation for Metal {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Metal>>".to_string(),
            fields: HashMap::from([
                ("albedo".to_string(), FieldInfo::new("<<albedo>>", Normal, Texture::get_documentation_structure())),
                ("normal".to_string(), FieldInfo::new("<<normal>>", Optional, Texture::get_documentation_structure())),
                ("fuzz".to_string(), FieldInfo::new_simple("<<metal>>", Normal, "<<f64>>")),
            ]),
        }
    }
}