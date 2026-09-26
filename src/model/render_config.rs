use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::FieldType::{Optional, OptionalList};
use crate::model::post_processor::PostProcessor;
use crate::model::width_height::WidthHeight;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct RenderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_height: Option<WidthHeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples_per_pixel: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub post_processors: Vec<PostProcessor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
}

impl Creator<solstrale::renderer::RenderConfig> for RenderConfig {
    fn create(
        &self,
        ctx: &CreatorContext,
    ) -> Result<solstrale::renderer::RenderConfig, Box<dyn Error>> {
        let mut post_processors: Vec<PostProcessors> = Vec::new();

        for p in &self.post_processors {
            post_processors.push(p.create(ctx)?);
        }

        let (width, height) = self
            .width_height
            .as_ref()
            .unwrap_or(&WidthHeight::default())
            .create(ctx)?;

        Ok(solstrale::renderer::RenderConfig {
            width,
            height,
            samples_per_pixel: self.samples_per_pixel.unwrap_or(200),
            post_processors,
            preview: self.preview.unwrap_or(false),
            // Take library defaults for the rest (max_depth, samples_per_batch).
            // Spreading rather than listing them keeps this immune to new fields.
            ..Default::default()
        })
    }
}

impl HelpDocumentation for RenderConfig {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "General configuration for the renderer".to_string(),
            fields: HashMap::from([
                (
                    "width_height".to_string(),
                    FieldInfo::new(
                        "Width and height in pixels of the rendered output",
                        Optional,
                        WidthHeight::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "samples_per_pixel".to_string(),
                    FieldInfo::new_simple(
                        "Number of rays shot for each pixel. More rays gives less noisy image but takes longer time. Defaults to 200",
                        Optional,
                        "Count of rays shot per pixel",
                    ),
                ),
                (
                    "post_processors".to_string(),
                    FieldInfo::new(
                        "A post processor is applied to the image after rendering for various effects",
                        OptionalList,
                        PostProcessor::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "preview".to_string(),
                    FieldInfo::new_simple(
                        "Run the denoise and saturation post processors on every batch rather than only the last, so the image is filtered while it renders and while the camera moves. Costs render time. Defaults to false",
                        Optional,
                        "true or false",
                    ),
                ),
            ]),
        }
    }
}
