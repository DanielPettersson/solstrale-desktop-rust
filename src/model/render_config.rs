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
            description: "General configuration for the renderer".to_string(),
            fields: HashMap::from([
                ("samples_per_pixel".to_string(), FieldInfo::new_simple(
                    "Number of rays shot for each pixel. More rays gives less noisy image but takes longer time",
                    Normal,
                    "Count of rays shot per pixel"
                )),
                ("shader".to_string(), FieldInfo::new(
                    "A shader is responsible for coloring the pixels where a ray has hit an object",
                    Normal,
                    Shader::get_documentation_structure()
                )),
                ("post_processors".to_string(), FieldInfo::new(
                    "A post processor is applied to the image after rendering for various effects",
                    List,
                    PostProcessor::get_documentation_structure()
                )),
                ("preview_interval_ms".to_string(), FieldInfo::new_simple(
                    "The minimum amount of milliseconds between preview images being generated by the renderer",
                    Normal,
                    "Milliseconds between preview images"
                )),
            ]),
        }
    }
}
