use std::error::Error;
use std::fmt;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::num::{Num, VisitNums};
use crate::model::scope::{Scope, check_name};
use crate::model::{ErrorPath, ModelError};

/// Named values that the rest of the scene can use in expressions. Each one
/// can use the ones declared before it.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Variables(pub Vec<(String, Num)>);

impl Variables {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// `base` with the variables added
    pub fn scope(&self, base: &Scope) -> Result<Scope, Box<dyn Error>> {
        let mut scope = base.clone();
        for (name, value) in &self.0 {
            let v = value
                .eval_scope(&scope)
                .map_err(|e| Box::new(ModelError::new(&e)) as Box<dyn Error>)
                .at(|| name.clone())
                .at(|| "variables".to_string());
            scope = scope.with(name, v?);
        }
        Ok(scope)
    }
}

impl VisitNums for Variables {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        self.0.iter().for_each(|(_, v)| f(v))
    }
}

impl Serialize for Variables {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Variables {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Variables, D::Error> {
        struct VarsVisitor;

        impl<'de> Visitor<'de> for VarsVisitor {
            type Value = Variables;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of variable names to values")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Variables, A::Error> {
                let mut vars: Vec<(String, Num)> = Vec::new();
                while let Some((name, value)) = map.next_entry::<String, Num>()? {
                    check_name(&name).map_err(serde::de::Error::custom)?;
                    if vars.iter().any(|(n, _)| *n == name) {
                        return Err(serde::de::Error::custom(format!(
                            "variable `{}` is declared twice",
                            name
                        )));
                    }
                    vars.push((name, value));
                }
                Ok(Variables(vars))
            }
        }

        d.deserialize_map(VarsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_order_and_reads_earlier_variables() {
        let vars: Variables = serde_yaml::from_str("b: 2\na: b * 3 + frameIndex\n").unwrap();
        assert_eq!(
            "b: 2.0\na: b * 3 + frameIndex\n",
            serde_yaml::to_string(&vars).unwrap()
        );
        let scope = vars.scope(&Scope::builtin(1)).unwrap();
        assert_eq!(Some(7.), scope.get("a"));
    }

    #[test]
    fn later_variables_are_not_visible() {
        let vars: Variables = serde_yaml::from_str("a: b\nb: 1\n").unwrap();
        let err = vars.scope(&Scope::builtin(0)).unwrap_err().to_string();
        assert_eq!("variables › a: unknown variable `b` in `b`", err);
    }

    #[test]
    fn rejects_bad_names() {
        for (yaml, msg) in [
            ("a: 1\na: 2\n", "declared twice"),
            ("frameIndex: 1\n", "built-in"),
            ("sin: 1\n", "function name"),
            ("2x: 1\n", "not a valid name"),
        ] {
            let err = serde_yaml::from_str::<Variables>(yaml)
                .unwrap_err()
                .to_string();
            assert!(err.contains(msg), "{}: {}", yaml, err);
        }
    }
}
