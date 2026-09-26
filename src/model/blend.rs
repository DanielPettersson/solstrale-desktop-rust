use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::Materials;

use crate::model::material::Material;
use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Blend {
    pub first: Material,
    pub second: Material,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_factor: Option<Num>,
}

visit_nums!(Blend, first, second, blend_factor);

impl Creator<Materials> for Blend {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Blend::new(
            self.first.create(ctx)?,
            self.second.create(ctx)?,
            self.blend_factor.eval_or(ctx, 0.5)?,
        )
        .into())
    }
}
