use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::{List, Normal};
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

impl HelpDocumentation for Quad {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "A flat rectangular hittable object".to_string(),
            fields: HashMap::from([
                ("q".to_string(), FieldInfo::new("<<q>>", Normal, Pos::get_documentation_structure())),
                ("u".to_string(), FieldInfo::new("<<u>>", Normal, Pos::get_documentation_structure())),
                ("v".to_string(), FieldInfo::new("<<v>>", Normal, Pos::get_documentation_structure())),
                ("material".to_string(), FieldInfo::new("<<material>>", Normal, Material::get_documentation_structure())),
                ("transformations".to_string(), FieldInfo::new("<<transformations>>", List, Transformation::get_documentation_structure())),
            ]),
        }
    }
}