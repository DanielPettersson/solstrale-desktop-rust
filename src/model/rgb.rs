use crate::model::{parse_option, Creator, DocumentationStructure, HelpDocumentation};
use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;
use std::error::Error;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

static R: &str = "r";
static G: &str = "g";
static B: &str = "b";

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
        let mut split = s.split(',');
        let r = parse_option::<D>(split.next(), R)?;
        let g = parse_option::<D>(split.next(), G)?;
        let b = parse_option::<D>(split.next(), B)?;
        Ok(Rgb { r, g, b })
    }
}

impl From<Rgb> for Vec3 {
    fn from(value: Rgb) -> Self {
        Vec3::new(value.r, value.g, value.b)
    }
}

impl Creator<Vec3> for Rgb {
    fn create(&self) -> Result<Vec3, Box<dyn Error>> {
        Ok(Vec3::new(self.r, self.g, self.b))
    }
}

impl HelpDocumentation for Rgb {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple("Value describing an R, G, B color. For example: 1, 1, 0 for yellow or 0.5, 0.5, 0.5 for gray")
    }
}
