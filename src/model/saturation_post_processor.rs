use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::post::PostProcessors;

use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct SaturationPostProcessor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturation_factor: Option<Num>,
}

visit_nums!(SaturationPostProcessor, saturation_factor);

impl Creator<PostProcessors> for SaturationPostProcessor {
    fn create(&self, ctx: &CreatorContext) -> Result<PostProcessors, Box<dyn Error>> {
        Ok(solstrale::post::SaturationPostProcessor::new(
            self.saturation_factor.eval_or(ctx, 0.5)?,
            ctx.device,
        )?
        .into())
    }
}
