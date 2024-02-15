use crate::model::albedo_shader::AlbedoShader;
use crate::model::normal_shader::NormalShader;
use crate::model::path_tracing_shader::PathTracingShader;
use crate::model::simple_shader::SimpleShader;
use crate::model::FieldType::Optional;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};
use serde::{Deserialize, Serialize};
use solstrale::renderer::shader::Shaders;
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
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
    fn create(&self, ctx: &CreatorContext) -> Result<Shaders, Box<dyn Error>> {
        match self {
            Shader {
                path_tracing: Some(p),
                simple: None,
                albedo: None,
                normal: None,
            } => p.create(ctx),
            Shader {
                path_tracing: None,
                simple: Some(s),
                albedo: None,
                normal: None,
            } => s.create(ctx),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: Some(a),
                normal: None,
            } => a.create(ctx),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: None,
                normal: Some(n),
            } => n.create(ctx),
            Shader {
                path_tracing: None,
                simple: None,
                albedo: None,
                normal: None,
            } => PathTracingShader::default().create(ctx),
            _ => Err(From::from(ModelError::new(
                "Shader should have single field defined",
            ))),
        }
    }
}

impl HelpDocumentation for Shader {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description:
                "A shader is responsible for coloring the pixels where a ray has hit an object"
                    .to_string(),
            fields: HashMap::from([
                (
                    "path_tracing".to_string(),
                    FieldInfo::new(
                        "A path tracing shader",
                        Optional,
                        PathTracingShader::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "simple".to_string(),
                    FieldInfo::new(
                        "Combines albedo and normal color without any light scattering",
                        Optional,
                        SimpleShader::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "albedo".to_string(),
                    FieldInfo::new(
                        "A simple shader that just shows the hittable's albedo color",
                        Optional,
                        AlbedoShader::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "normal".to_string(),
                    FieldInfo::new(
                        "Shader for displaying the normals of where rays intersect with hittables",
                        Optional,
                        NormalShader::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
