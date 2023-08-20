use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{Normal, Optional};

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

impl HelpDocumentation for BloomPostProcessor {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<BloomPostProcessor>>".to_string(),
            fields: HashMap::from([
                ("kernel_size_fraction".to_string(), FieldInfo::new_simple("<<kernel_size_fraction>>", Normal, "<<f64>>")),
                ("threshold".to_string(), FieldInfo::new_simple("<<threshold>>", Optional, "<<Option<f64>>>")),
                ("max_intensity".to_string(), FieldInfo::new_simple("<<max_intensity>>", Optional,"<<Option<f64>>>"))
            ]),
        }
    }
}