use crate::model::{Creator, DocumentationStructure, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AlbedoShader {}

impl Creator<Shaders> for AlbedoShader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::AlbedoShader::new())
    }
}

impl HelpDocumentation for AlbedoShader {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "A simple shader that just outputs the flat albedo color",
        )
    }
}
