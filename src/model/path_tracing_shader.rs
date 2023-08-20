use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use crate::model::Creator;

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