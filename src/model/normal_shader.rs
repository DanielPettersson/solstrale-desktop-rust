use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use crate::model::{Creator, DocumentationStructure, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct NormalShader {}

impl Creator<Shaders> for NormalShader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::NormalShader::new())
    }
}

impl HelpDocumentation for NormalShader {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure::new_simple("<<NormalShader>>")
    }
}