use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct DenoisePostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guide: Option<DenoiseGuide>,
}

visit_nums!(DenoisePostProcessor, strength);

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
            self.strength.eval_or(ctx, 1.)?,
            self.iterations,
            self.guide.map(Into::into),
            ctx.device,
        )?
        .into())
    }
}
