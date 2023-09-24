use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;

use crate::model::{Creator, DocumentationStructure, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimpleShader {}

impl Creator<Shaders> for SimpleShader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::SimpleShader::new())
    }
}

impl HelpDocumentation for SimpleShader {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "Combines albedo and normal color without any light scattering",
        )
    }
}
