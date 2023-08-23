use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;
use solstrale::renderer::RenderImageStrategy;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{List, Normal};
use crate::model::post_processor::PostProcessor;
use crate::model::shader::Shader;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct RenderConfig {
    pub samples_per_pixel: u32,
    pub shader: Shader,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub post_processors: Vec<PostProcessor>,
    pub preview_interval_ms: u64,
}

impl Creator<solstrale::renderer::RenderConfig> for RenderConfig {
    fn create(&self) -> Result<solstrale::renderer::RenderConfig, Box<dyn Error>> {
        let mut post_processors: Vec<PostProcessors> = Vec::new();

        for p in &self.post_processors {
            post_processors.push(p.create()?);
        }

        Ok(solstrale::renderer::RenderConfig {
            samples_per_pixel: self.samples_per_pixel,
            shader: self.shader.create()?,
            post_processors,
            render_image_strategy: if self.preview_interval_ms == 0 {
                RenderImageStrategy::EverySample
            } else {
                RenderImageStrategy::Interval(Duration::from_millis(self.preview_interval_ms))
            },
        })
    }
}

impl HelpDocumentation for RenderConfig {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<RenderConfig>>".to_string(),
            fields: HashMap::from([
                ("samples_per_pixel".to_string(), FieldInfo::new_simple("<<samples_per_pixel>>", Normal, "<<u32>>")),
                ("shader".to_string(), FieldInfo::new("<<shader>>", Normal, Shader::get_documentation_structure())),
                ("post_processors".to_string(), FieldInfo::new("<<post_processors>>", List, PostProcessor::get_documentation_structure())),
                ("preview_interval_ms".to_string(), FieldInfo::new_simple("<<preview_interval_ms>>", Normal, "<<u64>>")),
            ]),
        }
    }
}
