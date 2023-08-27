use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use crate::model::bloom_post_processor::BloomPostProcessor;
use crate::model::denoise_post_processor::DenoisePostProcessor;
use crate::model::FieldType::Optional;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bloom: Option<BloomPostProcessor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denoise: Option<DenoisePostProcessor>,
}

impl Creator<PostProcessors> for PostProcessor {
    fn create(&self) -> Result<PostProcessors, Box<dyn Error>> {
        match self {
            PostProcessor {
                bloom: Some(b),
                denoise: None,
            } => b.create(),
            PostProcessor {
                bloom: None,
                denoise: Some(d),
            } => d.create(),
            _ => Err(Box::try_from(ModelError::new(
                "PostProcessor should have single field defined",
            ))
                .unwrap()),
        }
    }
}

impl HelpDocumentation for PostProcessor {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "A post processor is applied to the image after rendering for various effects".to_string(),
            fields: HashMap::from([
                ("bloom".to_string(), FieldInfo::new(
                    "A post processor that applies a bloom effect to bright areas of the image",
                    Optional,
                    BloomPostProcessor::get_documentation_structure()
                )),
                ("denoise".to_string(), FieldInfo::new(
                    "A post processor that applies a de-noising filter to the image. Which gives the appearance of a higher number of samples rendered.",
                    Optional,
                    DenoisePostProcessor::get_documentation_structure()
                )),
            ]),
        }
    }
}