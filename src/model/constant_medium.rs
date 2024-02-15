use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::geo::transformation::NopTransformer;
use solstrale::hittable::{Bvh, Hittables, Quad};
use solstrale::material::texture::SolidColor;
use solstrale::material::Lambertian;

use crate::model::pos::Pos;
use crate::model::rgb::Rgb;
use crate::model::FieldType::{Normal, Optional};
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConstantMedium {
    pub a: Pos,
    pub b: Pos,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Rgb>,
}

impl Creator<Hittables> for ConstantMedium {
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        Ok(solstrale::hittable::ConstantMedium::new(
            Bvh::new(Quad::new_box(
                self.a.create(ctx)?,
                self.b.create(ctx)?,
                Lambertian::new(SolidColor::new(0., 0., 0.), None),
                &NopTransformer {},
            )),
            self.density.unwrap_or(0.01),
            self.color.unwrap_or(Rgb::new(0.9, 0.9, 0.9)).into(),
        ))
    }
}

impl HelpDocumentation for ConstantMedium {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A fog type hittable object where rays not only scatter at the edge of the object, but at random points inside the object. Which gives a fog-like material.".to_string(),
            fields: HashMap::from([
                ("a".to_string(), FieldInfo::new(
                    "Position of a corner of the box containing the fog",
                    Normal,
                    Pos::get_documentation_structure(depth + 1),
                )),
                ("b".to_string(),
                FieldInfo::new(
                    "Position of the corner opposite to 'a' of the box containing the fog",
                    Normal,
                    Pos::get_documentation_structure(depth + 1),
                )),
                ("density".to_string(), FieldInfo::new_simple(
                    "Density of the fog",
                    Optional,
                    "A higher density increases the probability for a ray to scatter in a given range. Defaults to 0.01"
                )),
                ("color".to_string(), FieldInfo::new(
                    "Color of the fog material. Defaults to 0.9, 0.9, 0.9",
                    Optional,
                    Rgb::get_documentation_structure(depth + 1)
                )),
            ]),
        }
    }
}
