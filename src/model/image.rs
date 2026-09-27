use crate::model::num::visit_nums;
use crate::model::texture_cache::{Kind, texture};
use crate::model::{Creator, CreatorContext};
use serde::{Deserialize, Serialize};
use solstrale::material::texture::{ImageMap, Textures};
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub file: String,
}

visit_nums!(Image);

impl Creator<Textures> for Image {
    fn create(&self, _: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        texture(Kind::Color, &self.file, ImageMap::load).map(|t| t.into())
    }
}
