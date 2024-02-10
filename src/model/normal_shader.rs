use crate::model::{Creator, CreatorContext, DocumentationStructure, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct NormalShader {}

impl Creator<Shaders> for NormalShader {
    fn create(&self, _: &CreatorContext) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::NormalShader::new())
    }
}

impl HelpDocumentation for NormalShader {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple("A simple shader that outputs a color representing the normal of the hittable of the ray's intersection point")
    }
}
