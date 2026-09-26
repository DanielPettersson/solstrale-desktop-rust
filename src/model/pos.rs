use crate::model::num::{Num, VisitNums, parse_triple};
use crate::model::scope::Scope;
use crate::model::{Creator, CreatorContext};
use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;
use std::error::Error;

#[derive(Clone, PartialEq, Debug)]
pub struct Pos {
    pub x: Num,
    pub y: Num,
    pub z: Num,
}

impl Pos {
    /// Creates a new instance
    pub fn new(x: f64, y: f64, z: f64) -> Pos {
        Pos {
            x: Num::Lit(x),
            y: Num::Lit(y),
            z: Num::Lit(z),
        }
    }
}

impl Default for Pos {
    fn default() -> Self {
        Pos::new(0., 0., 0.)
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
        let [x, y, z] = parse_triple(&s)?;
        Ok(Pos { x, y, z })
    }
}

impl From<Vec3> for Pos {
    fn from(value: Vec3) -> Self {
        Pos::new(value.x, value.y, value.z)
    }
}

impl Creator<Vec3> for Pos {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec3, Box<dyn Error>> {
        Ok(Vec3::new(
            self.x.eval(ctx)?,
            self.y.eval(ctx)?,
            self.z.eval(ctx)?,
        ))
    }
}

impl Pos {
    pub fn eval(&self, scope: &Scope) -> Result<Vec3, String> {
        Ok(Vec3::new(
            self.x.eval_scope(scope)?,
            self.y.eval_scope(scope)?,
            self.z.eval_scope(scope)?,
        ))
    }
}

impl VisitNums for Pos {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        f(&self.x);
        f(&self.y);
        f(&self.z);
    }
}
