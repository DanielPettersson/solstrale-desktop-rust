use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::{Bvh, Hittables};

use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::FieldType::Normal;
use crate::model::r#box::Box as BoxHittable;
use crate::model::rgb::Rgb;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConstantMedium {
    pub r#box: Box<BoxHittable>,
    pub density: f64,
    pub color: Rgb,
}

impl Creator<Hittables> for ConstantMedium {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        let children = self.r#box.create()?;

        let child = if children.len() == 1 {
            children.get(0).unwrap().clone()
        } else {
            Bvh::new(children)
        };

        Ok(solstrale::hittable::ConstantMedium::new(
            child,
            self.density,
            self.color.into(),
        ))
    }
}

impl HelpDocumentation for ConstantMedium {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<ConstantMedium>>".to_string(),
            fields: HashMap::from([
                ("box".to_string(), FieldInfo::new("<<box>>", Normal, BoxHittable::get_documentation_structure())),
                ("density".to_string(), FieldInfo::new_simple("<<density>>", Normal, "<<f64>>")),
                ("color".to_string(), FieldInfo::new("<<color>>", Normal, Rgb::get_documentation_structure())),
            ]),
        }
    }
}