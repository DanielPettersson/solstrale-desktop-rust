use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::{Dielectric, Materials};

use crate::model::normal_texture::NormalTexture;
use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::rgb::Rgb;
use crate::model::texture::Texture;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Glass {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_of_refraction: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roughness: Option<Num>,
}

visit_nums!(Glass, albedo, index_of_refraction, roughness);

/// Full transmission per world unit, so glass without an albedo is clear.
static CLEAR_GLASS_ALBEDO: Texture = Texture::Color(Rgb::new(1., 1., 1.));

impl Creator<Materials> for Glass {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(Dielectric::new(
            self.albedo
                .as_ref()
                .unwrap_or(&CLEAR_GLASS_ALBEDO)
                .create(ctx)?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create(ctx)?),
            },
            self.index_of_refraction.eval_or(ctx, 1.5)?,
            self.roughness.eval_or(ctx, 0.)?,
        )
        .into())
    }
}
