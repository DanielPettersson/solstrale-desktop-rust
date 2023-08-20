use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use crate::model::{Creator, ModelError};
use crate::model::albedo_shader::AlbedoShader;
use crate::model::normal_shader::NormalShader;
use crate::model::path_tracing_shader::PathTracingShader;
use crate::model::simple_shader::SimpleShader;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Shader {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_tracing: Option<PathTracingShader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simple: Option<SimpleShader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<AlbedoShader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalShader>,
}

impl Creator<Shaders> for Shader {
    fn create(&self) -> Result<Shaders, Box<dyn Error>> {
        match self {
            Shader {
                path_tracing: Some(p),
                simple: None,
                albedo: None,
                normal: None,
            } => p.create(),
            Shader {
                path_tracing: None,
                simple: Some(s),
                albedo: None,
                normal: None,
            } => s.create(),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: Some(a),
                normal: None,
            } => a.create(),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: None,
                normal: Some(n),
            } => n.create(),
            _ => Err(
                Box::try_from(ModelError::new("Shader should have single field defined"))
                    .unwrap(),
            ),
        }
    }
}
