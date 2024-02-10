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
pub struct Box {
    pub a: Pos,
    pub b: Pos,
    pub material: Material,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

impl Creator<Vec<Hittables>> for Box {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, std::boxed::Box<dyn Error>> {
        Ok(solstrale::hittable::Quad::new_box(
            self.a.create(ctx)?,
            self.b.create(ctx)?,
            self.material.create(ctx)?,
            &create_transformation(&self.transformations, ctx)?,
        ))
    }
}

impl HelpDocumentation for Box {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A hittable in the shape of a box".to_string(),
            fields: HashMap::from([
                (
                    "a".to_string(),
                    FieldInfo::new(
                        "Position of a corner of the box",
                        Normal,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "b".to_string(),
                    FieldInfo::new(
                        "Position of the corner opposite to 'a' of the box",
                        Normal,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "material".to_string(),
                    FieldInfo::new(
                        "Material of the box",
                        Normal,
                        Material::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "transformations".to_string(),
                    FieldInfo::new(
                        "Transformations to be applied to the position and size of the box",
                        OptionalList,
                        Transformation::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
