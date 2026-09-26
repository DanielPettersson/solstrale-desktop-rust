use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::num::visit_nums;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ScreenWidthHeight {}

impl Creator<(usize, usize)> for ScreenWidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        Ok((ctx.screen_width, ctx.screen_height))
    }
}

visit_nums!(ScreenWidthHeight);
