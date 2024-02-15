use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::Materials;

use crate::model::normal_texture::NormalTexture;
use crate::model::texture::Texture;
use crate::model::FieldType::Optional;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Metal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
    pub fuzz: Option<f64>,
}

impl Creator<Materials> for Metal {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Metal::new(
            self.albedo
                .as_ref()
                .unwrap_or(&Texture::default())
                .create(ctx)?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create(ctx)?),
            },
            self.fuzz.unwrap_or(0.05),
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
                    Optional,
                    Texture::get_documentation_structure(depth + 1)
                )),
                ("normal".to_string(), FieldInfo::new(
                    "Texture for the material's normals. Used to give the illusion of fine structure of the hittable",
                    Optional,
                    NormalTexture::get_documentation_structure(depth + 1)
                )),
                ("fuzz".to_string(), FieldInfo::new_simple(
                    "The smoothness of the material",
                    Optional,
                    "The fraction of randomness for the ray scattering direction. Defaults to 0.05"
                )),
            ]),
        }
    }
}
