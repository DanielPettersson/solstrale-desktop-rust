use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::{Bvh, Hittables};

use crate::model::r#box::Box as BoxHittable;
use crate::model::rgb::Rgb;
use crate::model::FieldType::Normal;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};

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
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A fog type hittable object where rays not only scatter at the edge of the object, but at random points inside the object. Which gives a fog-like material.".to_string(),
            fields: HashMap::from([
                ("box".to_string(), FieldInfo::new(
                    "A box describing the fog volume",
                    Normal,
                    BoxHittable::get_documentation_structure(depth + 1)
                )),
                ("density".to_string(), FieldInfo::new_simple(
                    "Density of the fog",
                    Normal,
                    "A higher density increases the probability for a ray to scatter in a given range"
                )),
                ("color".to_string(), FieldInfo::new(
                    "Color of the fog material",
                    Normal,
                    Rgb::get_documentation_structure(depth + 1)
                )),
            ]),
        }
    }
}
