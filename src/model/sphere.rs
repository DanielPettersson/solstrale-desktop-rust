use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use crate::model::Creator;
use crate::model::material::Material;
use crate::model::pos::Pos;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Sphere {
    pub center: Pos,
    pub radius: f64,
    pub material: Material,
}

impl Creator<Hittables> for Sphere {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::Sphere::new(
            self.center.create()?,
            self.radius,
            self.material.create()?,
        ))
    }
}