use crate::model::FieldType::Optional;
use crate::model::bloom_post_processor::BloomPostProcessor;
use crate::model::denoise_post_processor::DenoisePostProcessor;
use crate::model::saturation_post_processor::SaturationPostProcessor;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};
use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denoise: Option<DenoisePostProcessor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bloom: Option<BloomPostProcessor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturation: Option<SaturationPostProcessor>,
}

impl Creator<PostProcessors> for PostProcessor {
    fn create(&self, ctx: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        match self {
            PostProcessor {
                denoise: Some(d),
                bloom: None,
                saturation: None,
            } => d.create(ctx),
            PostProcessor {
                denoise: None,
                bloom: Some(b),
                saturation: None,
            } => b.create(ctx),
            PostProcessor {
                denoise: None,
                bloom: None,
                saturation: Some(s),
            } => s.create(ctx),
            _ => Err(From::from(ModelError::new(
                "PostProcessor should have single field defined",
            ))),
        }
    }
}

impl HelpDocumentation for PostProcessor {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description:
                "A post processor is applied to the image after rendering for various effects"
                    .to_string(),
            fields: HashMap::from([
                (
                    "denoise".to_string(),
                    FieldInfo::new(
                        "A post processor that removes noise from the image. Put it before any other post processor: denoising a bloomed image blurs the bloom",
                        Optional,
                        DenoisePostProcessor::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "bloom".to_string(),
                    FieldInfo::new(
                        "A post processor that applies a bloom effect to bright areas of the image",
                        Optional,
                        BloomPostProcessor::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "saturation".to_string(),
                    FieldInfo::new(
                        "A post processor that applies saturation to the image.",
                        Optional,
                        SaturationPostProcessor::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
