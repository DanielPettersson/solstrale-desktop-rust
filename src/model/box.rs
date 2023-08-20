use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use crate::model::Creator;
use crate::model::material::Material;
use crate::model::pos::Pos;
use crate::model::transformation::{create_transformation, Transformation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Box {
    pub a: Pos,
    pub b: Pos,
    pub material: Material,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

impl Creator<Vec<Hittables>> for Box {
    fn create(&self) -> Result<Vec<Hittables>, std::boxed::Box<dyn Error>> {
        Ok(solstrale::hittable::Quad::new_box(
            self.a.create()?,
            self.b.create()?,
            self.material.create()?,
            &create_transformation(&self.transformations)?,
        ))
    }
}