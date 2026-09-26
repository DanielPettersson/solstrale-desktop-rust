use crate::model::blend::Blend;
use crate::model::glass::Glass;
use crate::model::lambertian::Lambertian;
use crate::model::light::Light;
use crate::model::metal::Metal;
use crate::model::one_of::one_of;
use crate::model::plastic::Plastic;
use crate::model::{Creator, CreatorContext};
use solstrale::material::Materials;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Material {
        lambertian => Lambertian(Lambertian),
        glass => Glass(Glass),
        metal => Metal(Metal),
        plastic => Plastic(Plastic),
        light => Light(Light),
        blend => Blend(Box<Blend>),
    }
    repr: MaterialRepr, MaterialReprRef;
    empty: Some(Material::default());
}

impl Default for Material {
    fn default() -> Self {
        Material::Lambertian(Lambertian::default())
    }
}

impl Creator<Materials> for Material {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        match self {
            Material::Lambertian(l) => l.create(ctx),
            Material::Glass(g) => g.create(ctx),
            Material::Metal(m) => m.create(ctx),
            Material::Plastic(p) => p.create(ctx),
            Material::Light(l) => l.create(ctx),
            Material::Blend(b) => b.create(ctx),
        }
    }
}
