use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;

use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{Normal, OptionalList};
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

impl HelpDocumentation for Box {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "A hittable in the shape of a box".to_string(),
            fields: HashMap::from([
                ("a".to_string(), FieldInfo::new(
                    "Position of a corner of the box",
                    Normal,
                    Pos::get_documentation_structure()
                )),
                ("b".to_string(), FieldInfo::new(
                    "Position of the corner opposite to 'a' of the box",
                    Normal,
                    Pos::get_documentation_structure()
                )),
                ("material".to_string(), FieldInfo::new(
                    "Material of the box",
                    Normal,
                    Material::get_documentation_structure()
                )),
                ("transformations".to_string(), FieldInfo::new(
                    "Transformations to be applied to the position and size of the box",
                    OptionalList,
                    Transformation::get_documentation_structure()
                ))
            ]),
        }
    }
}