use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::{Dielectric, Materials};

use crate::model::normal_texture::NormalTexture;
use crate::model::texture::Texture;
use crate::model::FieldType::Optional;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_of_refraction: Option<f64>,
}

impl Creator<Materials> for Glass {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(Dielectric::new(
            self.albedo
                .as_ref()
                .unwrap_or(&Texture::default())
                .create(ctx)?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create(ctx)?),
            },
            self.index_of_refraction.unwrap_or(1.5),
        ))
    }
}

impl HelpDocumentation for Glass {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A dielectric material which has a glass-like appearance".to_string(),
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
                ("index_of_refraction".to_string(), FieldInfo::new_simple(
                    "The refractive index determines how much the path of light is bent, or refracted, when entering a material",
                    Optional,
                    "For example, glass normally has 1.5 and water 1.33. Defaults to 1.5"
                )),
            ]),
        }
    }
}
