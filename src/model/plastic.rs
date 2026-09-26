use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::{Lambertian, Materials, Metal};

use crate::model::normal_texture::NormalTexture;
use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::texture::Texture;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Plastic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossiness: Option<Num>,
}

visit_nums!(Plastic, albedo, glossiness);

impl Creator<Materials> for Plastic {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        let albedo = self
            .albedo
            .as_ref()
            .unwrap_or(&Texture::default())
            .create(ctx)?;
        let normal = match self.normal.as_ref() {
            None => None,
            Some(n) => Some(n.create(ctx)?),
        };

        Ok(solstrale::material::Blend::new(
            Lambertian::new(albedo.clone(), normal.clone()).into(),
            Metal::new(albedo, normal, 0.05).into(),
            self.glossiness.eval_or(ctx, 0.1)?,
        )
        .into())
    }
}
