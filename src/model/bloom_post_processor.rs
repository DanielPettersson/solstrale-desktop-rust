use crate::model::FieldType::{Normal, Optional};
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;
use std::collections::HashMap;
use std::error::Error;

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
    fn create(&self, _: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::BloomPostProcessor::new(
            self.kernel_size_fraction,
            self.threshold,
            self.max_intensity,
        )?)
    }
}

impl HelpDocumentation for BloomPostProcessor {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A post processor that applies a bloom effect to bright areas of the image".to_string(),
            fields: HashMap::from([
                ("kernel_size_fraction".to_string(), FieldInfo::new_simple(
                    "Size of the convolution filter applied to create the bloom effect",
                    Normal,
                    "A float number expressed as a fraction of the image width"
                )),
                ("threshold".to_string(), FieldInfo::new_simple(
                    "Amount of brightness needed for bloom effect to be applied to a pixel",
                    Optional,
                    "The threshold as the length of the color as a vector"
                )),
                ("max_intensity".to_string(), FieldInfo::new_simple(
                    "Used to limit the intensity of the bloom effect",
                    Optional,
                    "When applying the bloom effect pixels will be normalized to maximum this value"
                ))
            ]),
        }
    }
}
