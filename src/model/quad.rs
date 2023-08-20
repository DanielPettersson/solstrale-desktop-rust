use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use crate::model::Creator;
use crate::model::material::Material;
use crate::model::pos::Pos;
use crate::model::transformation::{create_transformation, Transformation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Quad {
    pub q: Pos,
    pub u: Pos,
    pub v: Pos,
    pub material: Material,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

impl Creator<Hittables> for Quad {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::Quad::new(
            self.q.create()?,
            self.u.create()?,
            self.v.create()?,
            self.material.create()?,
            &create_transformation(&self.transformations)?,
        ))
    }
}
