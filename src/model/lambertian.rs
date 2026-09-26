use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::Materials;

use crate::model::normal_texture::NormalTexture;
use crate::model::num::visit_nums;
use crate::model::texture::Texture;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct Lambertian {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo: Option<Texture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal: Option<NormalTexture>,
}

visit_nums!(Lambertian, albedo);

impl Creator<Materials> for Lambertian {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        Ok(solstrale::material::Lambertian::new(
            self.albedo
                .as_ref()
                .unwrap_or(&Texture::default())
                .create(ctx)?,
            match self.normal.as_ref() {
                None => None,
                Some(n) => Some(n.create(ctx)?),
            },
        )
        .into())
    }
}
