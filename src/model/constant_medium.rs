use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::{Bvh, Hittables};
use crate::model::hittable::Hittable;
use crate::model::Creator;
use crate::model::rgb::Rgb;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConstantMedium {
    pub child: Box<Hittable>,
    pub density: f64,
    pub color: Rgb,
}

impl Creator<Hittables> for ConstantMedium {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        let children = self.child.create()?;

        let child = if children.len() == 1 {
            children.get(0).unwrap().clone()
        } else {
            Bvh::new(children)
        };

        Ok(solstrale::hittable::ConstantMedium::new(
            child,
            self.density,
            self.color.into(),
        ))
    }
}