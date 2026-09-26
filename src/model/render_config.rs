use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::num::visit_nums;
use crate::model::post_processor::PostProcessor;
use crate::model::width_height::WidthHeight;
use crate::model::{Creator, CreatorContext};

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

visit_nums!(RenderConfig, post_processors);

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
