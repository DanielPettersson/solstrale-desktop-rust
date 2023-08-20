use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::Normal;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct PathTracingShader {
    pub max_depth: u32,
}

impl Creator<Shaders> for PathTracingShader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::PathTracingShader::new(self.max_depth))
    }
}

impl HelpDocumentation for PathTracingShader {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<PathTracingShader>>".to_string(),
            fields: HashMap::from([
                ("max_depth".to_string(), FieldInfo::new_simple("<<max_depth>>", Normal, "<<u32>>")),
            ]),
        }
    }
}