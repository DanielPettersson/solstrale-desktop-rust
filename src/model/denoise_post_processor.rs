use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::FieldType::Optional;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DenoisePostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guide: Option<DenoiseGuide>,
}

/// Which primary-hit channels the filter is allowed to use as an edge guide.
/// Mirrors [`solstrale::post::DenoiseGuide`], as a plain scalar so a scene
/// writes `guide: color_only` rather than nesting another mapping.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DenoiseGuide {
    Full,
    ColorOnly,
}

impl From<DenoiseGuide> for solstrale::post::DenoiseGuide {
    fn from(guide: DenoiseGuide) -> Self {
        match guide {
            DenoiseGuide::Full => solstrale::post::DenoiseGuide::Full,
            DenoiseGuide::ColorOnly => solstrale::post::DenoiseGuide::ColorOnly,
        }
    }
}

impl Creator<PostProcessors> for DenoisePostProcessor {
    fn create(&self, ctx: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::DenoisePostProcessor::new(
            self.strength.unwrap_or(1.),
            self.iterations,
            self.guide.map(Into::into),
            ctx.device,
        )?
        .into())
    }
}

impl HelpDocumentation for DenoisePostProcessor {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A post processor that removes noise from the rendered image".to_string(),
            fields: HashMap::from([
                (
                    "strength".to_string(),
                    FieldInfo::new_simple(
                        "How hard the filter is allowed to blur",
                        Optional,
                        "From 0 to 10, where 1 is the tuned default. Below 1 keeps more detail and more noise, above 1 blurs harder. Defaults to 1",
                    ),
                ),
                (
                    "iterations".to_string(),
                    FieldInfo::new_simple(
                        "Number of filter passes, each one reaching twice as far as the last",
                        Optional,
                        "From 1 to 8. Five reaches 32 pixels, which is the usual choice. Defaults to 5",
                    ),
                ),
                (
                    "guide".to_string(),
                    FieldInfo::new_simple(
                        "Which information about the surface seen in each pixel is used to avoid blurring across edges",
                        Optional,
                        "Either \"full\", using the surface color, angle and distance, or \"color_only\", which is noticeably softer as silhouettes between similarly lit surfaces merge. Defaults to \"full\"",
                    ),
                ),
            ]),
        }
    }
}
