use crate::model::material::Material;
use crate::model::pos::Pos;
use crate::model::FieldType::{Normal, Optional};
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Sphere {
    pub center: Pos,
    pub radius: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
}

impl Creator<Hittables> for Sphere {
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::Sphere::new(
            self.center.create(ctx)?,
            self.radius,
            self.material
                .as_ref()
                .unwrap_or(&Material::default())
                .create(ctx)?,
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
                        Optional,
                        Material::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
