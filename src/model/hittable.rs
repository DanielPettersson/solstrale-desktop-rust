use crate::model::r#box::Box;
use crate::model::obj_model::ObjModel;
use crate::model::one_of::one_of;
use crate::model::quad::Quad;
use crate::model::repeat::Repeat;
use crate::model::sphere::Sphere;
use crate::model::{Creator, CreatorContext, ErrorPath};
use solstrale::hittable::Hittables;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Hittable {
        sphere => Sphere(Sphere),
        model => Model(ObjModel),
        quad => Quad(Quad),
        r#box => Box(Box),
        repeat => Repeat(Repeat),
    }
    repr: HittableRepr, HittableReprRef;
    empty: None;
}

impl Creator<Vec<Hittables>> for Hittable {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, std::boxed::Box<dyn Error>> {
        match self {
            Hittable::Sphere(s) => s.create(ctx).map(|h| vec![h]).at(|| "sphere".into()),
            Hittable::Model(m) => m.create(ctx).map(|h| vec![h]).at(|| "model".into()),
            Hittable::Quad(q) => q.create(ctx).map(|h| vec![h]).at(|| "quad".into()),
            Hittable::Box(b) => b.create(ctx).at(|| "box".into()),
            // Adds its own path segments, with the loop variable's value
            Hittable::Repeat(r) => r.create(ctx),
        }
    }
}
