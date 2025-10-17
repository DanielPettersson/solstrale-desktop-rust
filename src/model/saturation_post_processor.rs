use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::FieldType::Optional;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SaturationPostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturation_factor: Option<f64>,
}

impl Creator<PostProcessors> for SaturationPostProcessor {
    fn create(&self, _: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::SaturationPostProcessor::new(
            self.saturation_factor.unwrap_or(0.5),
        )?)
    }
}

impl HelpDocumentation for SaturationPostProcessor {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A post processor that applies saturation to the image".to_string(),
            fields: HashMap::from([
                ("saturation_factor".to_string(), FieldInfo::new_simple(
                    "The amount of saturation applied to the image",
                    Optional,
                    "Controls how much the image is saturated. From -1 (grayscale image) to 1 (fully saturated colors). Defaults to 0.5"
                )),
            ]),
        }
    }
}
