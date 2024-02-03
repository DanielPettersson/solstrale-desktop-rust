use crate::model::texture::Texture;
use crate::model::FieldType::{Normal, Optional};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::material::Materials;
use std::collections::HashMap;
use std::error::Error;

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
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A reflective material that gives a metallic appearance".to_string(),
            fields: HashMap::from([
                ("albedo".to_string(), FieldInfo::new(
                    "Texture for the material's albedo color",
                    Normal,
                    Texture::get_documentation_structure(depth + 1)
                )),
                ("normal".to_string(), FieldInfo::new(
                    "Texture for the material's normals. Used to give the illusion of fine structure of the hittable",
                    Optional,
                    Texture::get_documentation_structure(depth + 1)
                )),
                ("fuzz".to_string(), FieldInfo::new_simple(
                    "The smoothness of the material",
                    Normal,
                    "The fraction of randomness for the ray scattering direction"
                )),
            ]),
        }
    }
}
