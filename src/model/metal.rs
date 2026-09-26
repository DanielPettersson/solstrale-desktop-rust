use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::Materials;

use crate::model::normal_texture::NormalTexture;
use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::texture::Texture;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Metal {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzz: Option<Num>,
}

visit_nums!(Metal, albedo, fuzz);

impl Creator<Materials> for Metal {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Metal::new(
            self.albedo
                .as_ref()
                .unwrap_or(&Texture::default())
                .create(ctx)?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create(ctx)?),
            },
            self.fuzz.eval_or(ctx, 0.05)?,
        )
        .into())
    }
}
