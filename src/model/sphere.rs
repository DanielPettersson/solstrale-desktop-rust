use crate::model::material::Material;
use crate::model::pos::Pos;
use crate::model::FieldType::Normal;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use std::collections::HashMap;
use std::error::Error;

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

impl HelpDocumentation for Sphere {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A sphere hittable object".to_string(),
            fields: HashMap::from([
                (
                    "center".to_string(),
                    FieldInfo::new(
                        "Position of the sphere's center",
                        Normal,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "radius".to_string(),
                    FieldInfo::new_simple("Radius of the sphere", Normal, "Radius of the sphere"),
                ),
                (
                    "material".to_string(),
                    FieldInfo::new(
                        "Material of the sphere",
                        Normal,
                        Material::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
