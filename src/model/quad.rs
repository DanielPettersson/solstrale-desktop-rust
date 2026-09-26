use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;

use crate::model::material::Material;
use crate::model::num::visit_nums;
use crate::model::pos::Pos;
use crate::model::transformation::{Transformation, create_transformation};
use crate::model::{Creator, CreatorContext};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Quad {
    pub q: Pos,
    pub u: Pos,
    pub v: Pos,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub material: Option<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

visit_nums!(Quad, q, u, v, material, transformations);

impl Default for Quad {
    fn default() -> Self {
        Quad {
            q: Pos::default(),
            u: Pos::new(1., 0., 0.),
            v: Pos::new(0., 1., 0.),
            material: None,
            transformations: Vec::new(),
        }
    }
}

impl Creator<Hittables> for Quad {
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::Quad::new(
            self.q.create(ctx)?,
            self.u.create(ctx)?,
            self.v.create(ctx)?,
            self.material
                .as_ref()
                .unwrap_or(&Material::default())
                .create(ctx)?,
            &create_transformation(&self.transformations, ctx)?,
        )
        .into())
    }
}
