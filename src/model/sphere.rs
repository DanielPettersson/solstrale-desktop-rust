use crate::model::material::Material;
use crate::model::num::{Num, visit_nums};
use crate::model::pos::Pos;
use crate::model::transformation::{Transformation, create_transformation};
use crate::model::{Creator, CreatorContext};
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Sphere {
    pub center: Pos,
    pub radius: Num,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

visit_nums!(Sphere, center, radius, material, transformations);

impl Default for Sphere {
    fn default() -> Self {
        Sphere {
            center: Pos::default(),
            radius: Num::Lit(1.),
            material: None,
            transformations: Vec::new(),
        }
    }
}

impl Creator<Hittables> for Sphere {
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::Sphere::new(
            self.center.create(ctx)?,
            self.radius.eval(ctx)?,
            self.material
                .as_ref()
                .unwrap_or(&Material::default())
                .create(ctx)?,
            &create_transformation(&self.transformations, ctx)?,
        )
        .into())
    }
}
