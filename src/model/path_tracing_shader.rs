use crate::model::FieldType::Normal;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct PathTracingShader {
    pub max_depth: u32,
}

impl Creator<Shaders> for PathTracingShader {
    fn create(&self, _: &CreatorContext) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::PathTracingShader::new(
            self.max_depth,
        ))
    }
}

impl HelpDocumentation for PathTracingShader {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "The main shader for this renderer. Gives the most realistic output".to_string(),
            fields: HashMap::from([
                ("max_depth".to_string(), FieldInfo::new_simple(
                    "Max number of bounces for each ray shot from the camera",
                    Normal,
                    "Max number of bounces. No more scattering of the ray is done when this is exceeded"
                )),
            ]),
        }
    }
}
