use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::Creator;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DenoisePostProcessor {}

impl Creator<PostProcessors> for DenoisePostProcessor {
    fn create(&self) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::OidnPostProcessor::new())
    }
}