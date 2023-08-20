use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::Normal;
use crate::model::material::Material;
use crate::model::pos::Pos;

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
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Sphere>>".to_string(),
            fields: HashMap::from([
                ("center".to_string(), FieldInfo::new("<<center>>", Normal, Pos::get_documentation_structure())),
                ("radius".to_string(), FieldInfo::new_simple("<<radius>>", Normal, "<<f64>>")),
                ("material".to_string(), FieldInfo::new("<<material>>", Normal, Material::get_documentation_structure())),
            ]),
        }
    }
}