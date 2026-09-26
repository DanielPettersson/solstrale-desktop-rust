use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;

use crate::model::num::{Num, VisitNums, parse_triple};
use crate::model::{Creator, CreatorContext, DocumentationStructure, HelpDocumentation};

#[derive(PartialEq, Debug, Clone)]
pub struct Rgb {
    pub r: Num,
    pub g: Num,
    pub b: Num,
}

impl Rgb {
    /// Creates a new instance
    pub const fn new(r: f64, g: f64, b: f64) -> Rgb {
        Rgb {
            r: Num::Lit(r),
            g: Num::Lit(g),
            b: Num::Lit(b),
        }
    }
}

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}, {}, {}", self.r, self.g, self.b))
    }
}

impl<'de> Deserialize<'de> for Rgb {
    fn deserialize<D>(deserializer: D) -> Result<Rgb, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let [r, g, b] = parse_triple(&s)?;
        Ok(Rgb { r, g, b })
    }
}

impl Creator<Vec3> for Rgb {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec3, Box<dyn Error>> {
        Ok(Vec3::new(
            self.r.eval(ctx)?,
            self.g.eval(ctx)?,
            self.b.eval(ctx)?,
        ))
    }
}

impl VisitNums for Rgb {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        f(&self.r);
        f(&self.g);
        f(&self.b);
    }
}

impl HelpDocumentation for Rgb {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "Value describing an R, G, B color. For example: 1, 1, 0 for yellow or 0.5, 0.5, 0.5 for gray. Each value can be an expression",
        )
    }
}
