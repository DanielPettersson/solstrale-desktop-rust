use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;
use crate::model::Creator;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BloomPostProcessor {
    pub kernel_size_fraction: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_intensity: Option<f64>,
}

impl Creator<PostProcessors> for BloomPostProcessor {
    fn create(&self) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::BloomPostProcessor::new(
            self.kernel_size_fraction,
            self.threshold,
            self.max_intensity,
        )?)
    }
}