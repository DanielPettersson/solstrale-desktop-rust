use crate::model::{
    parse_option, Creator, CreatorContext, DocumentationStructure, HelpDocumentation,
};
use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;
use std::error::Error;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

static X: &str = "x";
static Y: &str = "y";
static Z: &str = "z";

impl Pos {
    /// Creates a new instance
    pub fn new(x: f64, y: f64, z: f64) -> Pos {
        Pos { x, y, z }
    }
}

impl Serialize for Pos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&format!("{}, {}, {}", self.x, self.y, self.z))
    }
}

impl<'de> Deserialize<'de> for Pos {
    fn deserialize<D>(deserializer: D) -> Result<Pos, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut split = s.split(',');
        let x = parse_option::<D>(split.next(), X)?;
        let y = parse_option::<D>(split.next(), Y)?;
        let z = parse_option::<D>(split.next(), Z)?;
        Ok(Pos { x, y, z })
    }
}

impl From<&Pos> for Vec3 {
    fn from(value: &Pos) -> Self {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl From<Vec3> for Pos {
    fn from(value: Vec3) -> Self {
        Pos {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl Creator<Vec3> for Pos {
    fn create(&self, _: &CreatorContext) -> Result<Vec3, Box<dyn Error>> {
        Ok(Vec3::new(self.x, self.y, self.z))
    }
}

impl HelpDocumentation for Pos {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "Value describing an X, Y, Z position in space. For example: 1.0, 2.0, -3.0",
        )
    }
}
