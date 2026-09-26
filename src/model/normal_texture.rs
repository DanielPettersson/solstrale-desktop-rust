use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::material::texture::{Textures, load_normal_texture};

use crate::model::num::visit_nums;
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct NormalTexture {
    pub file: String,
}

visit_nums!(NormalTexture);

impl Creator<Textures> for NormalTexture {
    fn create(&self, _: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        load_normal_texture(self.file.as_ref()).map(|t| t.into())
    }
}
