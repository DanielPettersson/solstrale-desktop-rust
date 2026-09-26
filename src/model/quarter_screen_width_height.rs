use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::num::visit_nums;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct QuarterScreenWidthHeight {}

impl Creator<(usize, usize)> for QuarterScreenWidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        Ok((ctx.screen_width / 4, ctx.screen_height / 4))
    }
}

visit_nums!(QuarterScreenWidthHeight);
