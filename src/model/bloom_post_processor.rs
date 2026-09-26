use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct BloomPostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_size_fraction: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_intensity: Option<Num>,
}

visit_nums!(
    BloomPostProcessor,
    kernel_size_fraction,
    threshold,
    max_intensity
);

impl Creator<PostProcessors> for BloomPostProcessor {
    fn create(&self, ctx: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::BloomPostProcessor::new(
            self.kernel_size_fraction.eval_or(ctx, 0.1)?,
            self.threshold.eval_opt(ctx)?,
            self.max_intensity.eval_opt(ctx)?,
            ctx.device,
        )?
        .into())
    }
}
