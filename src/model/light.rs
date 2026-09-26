use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::{DiffuseLight, Materials};

use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::rgb::Rgb;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Light {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attenuation_half_length: Option<Num>,
}

visit_nums!(Light, color, attenuation_half_length);

impl Creator<Materials> for Light {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        let c = self
            .color
            .as_ref()
            .unwrap_or(&Rgb::new(15.0, 15.0, 15.0))
            .create(ctx)?;
        Ok(DiffuseLight::new(c.x, c.y, c.z, self.attenuation_half_length.eval_opt(ctx)?).into())
    }
}
