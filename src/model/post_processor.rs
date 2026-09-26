use crate::model::FieldType::Optional;
use crate::model::bloom_post_processor::BloomPostProcessor;
use crate::model::denoise_post_processor::DenoisePostProcessor;
use crate::model::one_of::one_of;
use crate::model::saturation_post_processor::SaturationPostProcessor;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use solstrale::post::PostProcessors;
use std::collections::HashMap;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum PostProcessor {
        denoise => Denoise(DenoisePostProcessor),
        bloom => Bloom(BloomPostProcessor),
        saturation => Saturation(SaturationPostProcessor),
    }
    repr: PostProcessorRepr, PostProcessorReprRef;
    empty: None;
}

impl Creator<PostProcessors> for PostProcessor {
    fn create(&self, ctx: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        match self {
            PostProcessor::Denoise(d) => d.create(ctx),
            PostProcessor::Bloom(b) => b.create(ctx),
            PostProcessor::Saturation(s) => s.create(ctx),
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
