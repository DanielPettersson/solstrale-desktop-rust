use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::{Dielectric, Materials};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{Normal, Optional};
use crate::model::texture::Texture;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    pub albedo: Texture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<Texture>,
    pub index_of_refraction: f64,
}

impl Creator<Materials> for Glass {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        Ok(Dielectric::new(
            self.albedo.create()?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create()?),
            },
            self.index_of_refraction,
        ))
    }
}

impl HelpDocumentation for Glass {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Glass>>".to_string(),
            fields: HashMap::from([
                ("albedo".to_string(), FieldInfo::new("<<albedo>>", Normal, Texture::get_documentation_structure())),
                ("normal".to_string(), FieldInfo::new("<<normal>>", Optional, Texture::get_documentation_structure())),
                ("index_of_refraction".to_string(), FieldInfo::new_simple("<<index_of_refraction>>", Normal, "<<f64>>")),
            ]),
        }
    }
}