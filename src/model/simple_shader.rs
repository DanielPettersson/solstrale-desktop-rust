use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use crate::model::Creator;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimpleShader {}

impl Creator<Shaders> for SimpleShader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        Ok(solstrale::renderer::shader::SimpleShader::new())
    }
}