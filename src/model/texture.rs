use crate::model::image::Image;
use crate::model::one_of::one_of;
use crate::model::rgb::Rgb;
use crate::model::{Creator, CreatorContext};
use solstrale::material::texture::{SolidColor, Textures};
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Texture {
        color => Color(Rgb) = Rgb::new(0.8, 0.8, 0.8),
        image => Image(Image),
    }
    repr: TextureRepr, TextureReprRef;
    empty: Some(Texture::default());
}

impl Default for Texture {
    fn default() -> Self {
        Texture::Color(Rgb::new(0.8, 0.8, 0.8))
    }
}

impl Creator<Textures> for Texture {
    fn create(&self, ctx: &CreatorContext) -> Result<Textures, Box<dyn Error>> {
        match self {
            Texture::Color(c) => {
                let c = c.create(ctx)?;
                Ok(SolidColor::new(c.x, c.y, c.z).into())
            }
            Texture::Image(im) => im.create(ctx),
        }
    }
}
