use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::material::texture::{ImageMap, Textures};
use crate::model::Creator;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub file: String,
}

impl Creator<Textures> for Image {
    fn create(&self) -> Result<Textures, Box<dyn Error>> {
        ImageMap::load(self.file.as_ref())
    }
}