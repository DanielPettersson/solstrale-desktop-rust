use crate::model::num::Num;
use crate::model::one_of::one_of;
use crate::model::pos::Pos;
use crate::model::{Creator, CreatorContext};
use solstrale::geo::transformation::{
    RotationX, RotationY, RotationZ, Scale, Transformations, Transformer, Translation,
};
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Transformation {
        translation => Translation(Pos),
        scale => Scale(Num) = Num::Lit(1.),
        rotation_x => RotationX(Num) = Num::Lit(0.),
        rotation_y => RotationY(Num) = Num::Lit(0.),
        rotation_z => RotationZ(Num) = Num::Lit(0.),
    }
    repr: TransformationRepr, TransformationReprRef;
    empty: None;
}

impl Creator<Box<dyn Transformer>> for Transformation {
    fn create(&self, ctx: &CreatorContext) -> Result<Box<dyn Transformer>, Box<dyn Error>> {
        Ok(match self {
            Transformation::Translation(p) => Box::new(Translation::new(p.create(ctx)?)),
            Transformation::Scale(s) => Box::new(Scale::new(s.eval(ctx)?)),
            Transformation::RotationX(r) => Box::new(RotationX::new(r.eval(ctx)?)),
            Transformation::RotationY(r) => Box::new(RotationY::new(r.eval(ctx)?)),
            Transformation::RotationZ(r) => Box::new(RotationZ::new(r.eval(ctx)?)),
        })
    }
}

pub fn create_transformation(
    transformations: &Vec<Transformation>,
    ctx: &CreatorContext,
) -> Result<Transformations, Box<dyn Error>> {
    let mut trans: Vec<Box<dyn Transformer>> = Vec::with_capacity(transformations.len());
    for t in transformations {
        trans.push(t.create(ctx)?);
    }
    Ok(Transformations::new(trans))
}
