use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;

use crate::model::num::{Num, VisitNums, parse_triple};
use crate::model::{Creator, CreatorContext, ModelError};

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
        let component = |n: &Num, name: &str| -> Result<f64, Box<dyn Error>> {
            let v = n.eval(ctx)?;
            if v < 0. {
                return Err(Box::new(ModelError::new(&format!(
                    "a color can not be negative, but {} is {}",
                    name, v
                ))));
            }
            Ok(v)
        };
        Ok(Vec3::new(
            component(&self.r, "r")?,
            component(&self.g, "g")?,
            component(&self.b, "b")?,
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
