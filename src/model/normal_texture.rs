use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::texture::{Textures, load_normal_texture};

use crate::model::num::visit_nums;
use crate::model::texture_cache::{Kind, texture};
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct NormalTexture {
    pub file: String,
}

visit_nums!(NormalTexture);

impl Creator<Textures> for NormalTexture {
    fn create(&self, _: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        texture(Kind::Normal, &self.file, load_normal_texture).map(|t| t.into())
    }
}
