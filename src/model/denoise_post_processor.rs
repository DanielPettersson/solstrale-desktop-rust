use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::{Creator, CreatorContext, DocumentationStructure, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DenoisePostProcessor {}

impl Creator<PostProcessors> for DenoisePostProcessor {
    fn create(&self, _: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::OidnPostProcessor::new())
    }
}

impl HelpDocumentation for DenoisePostProcessor {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple("A post processor that applies a de-noising filter to the image. Which gives the appearance of a higher number of samples rendered.")
    }
}
