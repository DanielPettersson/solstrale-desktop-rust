use crate::model::bloom_post_processor::BloomPostProcessor;
use crate::model::denoise_post_processor::DenoisePostProcessor;
use crate::model::one_of::one_of;
use crate::model::saturation_post_processor::SaturationPostProcessor;
use crate::model::{Creator, CreatorContext};
use solstrale::post::PostProcessors;
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
