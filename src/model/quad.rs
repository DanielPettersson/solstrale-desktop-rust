use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;

use crate::model::material::Material;
use crate::model::pos::Pos;
use crate::model::transformation::{create_transformation, Transformation};
use crate::model::FieldType::{Normal, OptionalList};
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

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
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::Quad::new(
            self.q.create(ctx)?,
            self.u.create(ctx)?,
            self.v.create(ctx)?,
            self.material.create(ctx)?,
            &create_transformation(&self.transformations, ctx)?,
        ))
    }
}

impl HelpDocumentation for Quad {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A flat rectangular hittable object".to_string(),
            fields: HashMap::from([
                (
                    "q".to_string(),
                    FieldInfo::new(
                        "Position of a corner of the quad",
                        Normal,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "u".to_string(),
                    FieldInfo::new(
                        "Direction of the first edge from 'q'",
                        Normal,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "v".to_string(),
                    FieldInfo::new(
                        "Direction of the other edge from 'q'",
                        Normal,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "material".to_string(),
                    FieldInfo::new(
                        "Material of the quad",
                        Normal,
                        Material::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "transformations".to_string(),
                    FieldInfo::new(
                        "Transformations to be applied to the position and size of the quad",
                        OptionalList,
                        Transformation::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
