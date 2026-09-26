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
pub struct Box {
    pub a: Pos,
    pub b: Pos,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

visit_nums!(Box, a, b, material, transformations);

impl Default for Box {
    fn default() -> Self {
        Box {
            a: Pos::default(),
            b: Pos::new(1., 1., 1.),
            material: None,
            transformations: Vec::new(),
        }
    }
}

impl Creator<Vec<Hittables>> for Box {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, std::boxed::Box<dyn Error>> {
        Ok(solstrale::hittable::Quad::new_box(
            self.a.create(ctx)?,
            self.b.create(ctx)?,
            self.material
                .as_ref()
                .unwrap_or(&Material::default())
                .create(ctx)?,
            &create_transformation(&self.transformations, ctx)?,
        ))
    }
}
