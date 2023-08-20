use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use crate::model::albedo_shader::AlbedoShader;
use crate::model::FieldType::Optional;
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

impl HelpDocumentation for Shader {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Shader>>".to_string(),
            fields: HashMap::from([
                ("path_tracing".to_string(), FieldInfo::new("<<path_tracing>>", Optional, PathTracingShader::get_documentation_structure())),
                ("simple".to_string(), FieldInfo::new("<<simple>>", Optional, SimpleShader::get_documentation_structure())),
                ("albedo".to_string(), FieldInfo::new("<<albedo>>", Optional, AlbedoShader::get_documentation_structure())),
                ("normal".to_string(), FieldInfo::new("<<normal>>", Optional, NormalShader::get_documentation_structure())),
            ]),
        }
    }
}
