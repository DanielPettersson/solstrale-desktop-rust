use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{Normal, Optional};
use crate::model::texture::Texture;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Lambertian {
    pub albedo: Texture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<Texture>,
}

impl Creator<Materials> for Lambertian {
    fn create(&self) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Lambertian::new(
            self.albedo.create()?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create()?),
            },
        ))
    }
}

impl HelpDocumentation for Lambertian {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Lambertian>>".to_string(),
            fields: HashMap::from([
                ("albedo".to_string(), FieldInfo::new("<<albedo>>", Normal, Texture::get_documentation_structure())),
                ("normal".to_string(), FieldInfo::new("<<normal>>", Optional, Texture::get_documentation_structure())),
            ]),
        }
    }
}