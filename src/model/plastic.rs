use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::{Lambertian, Materials, Metal};

use crate::model::normal_texture::NormalTexture;
use crate::model::texture::Texture;
use crate::model::FieldType::Optional;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Plastic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossiness: Option<f64>,
}

impl Creator<Materials> for Plastic {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        let albedo = self
            .albedo
            .as_ref()
            .unwrap_or(&Texture::default())
            .create(ctx)?;
        let normal = match self.normal.as_ref() {
            None => None,
            Some(n) => Some(n.create(ctx)?),
        };

        Ok(solstrale::material::Blend::new(
            Lambertian::new(albedo.clone(), normal.clone()),
            Metal::new(albedo, normal, 0.05),
            self.glossiness.unwrap_or(0.1),
        ))
    }
}

impl HelpDocumentation for Plastic {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A material with plastic-like appearance".to_string(),
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
                ("glossiness".to_string(), FieldInfo::new_simple(
                    "The glossiness of the plastic. 0 is matte and 1 is metal. Defaults to 0.1",
                    Optional,
                    "The glossiness of the plastic. 0 is matte and 1 is metal. Defaults to 0.1"
                )),
            ]),
        }
    }
}
