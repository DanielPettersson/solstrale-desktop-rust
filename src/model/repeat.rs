use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Deserializer, Serialize};
use solstrale::hittable::Hittables;

use crate::model::FieldType::{List, Normal, Optional};
use crate::model::hittable::Hittable;
use crate::model::num::{EvalOr, Num, visit_nums};
use crate::model::scope::check_name;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, ErrorPath, FieldInfo, HelpDocumentation,
    ModelError,
};

/// Most iterations a single repeat may run
pub const MAX_ITERATIONS: usize = 10_000;
/// Most hittables a repeat may expand to, counting nested repeats
pub const MAX_HITTABLES: usize = 1_000_000;

/// Creates its world once per value of a loop variable.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    #[serde(deserialize_with = "deserialize_variable")]
    pub variable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Num>,
    pub to: Num,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<Num>,
    #[serde(default)]
    pub world: Vec<Hittable>,
}

visit_nums!(Repeat, from, to, step, world);

impl Default for Repeat {
    fn default() -> Self {
        Repeat {
            variable: "i".to_string(),
            from: None,
            to: Num::Lit(10.),
            step: None,
            world: Vec::new(),
        }
    }
}

fn deserialize_variable<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let name = String::deserialize(d)?;
    check_name(&name).map_err(serde::de::Error::custom)?;
    Ok(name)
}

/// The values the loop variable takes: from `from`, by `step`, up to but not
/// including `to`. Computed by multiplication so steps like 0.1 do not drift.
pub fn loop_values(from: f64, to: f64, step: f64) -> Result<Vec<f64>, String> {
    if step == 0. {
        return Err("step can not be 0".to_string());
    }
    // The epsilon keeps `to` excluded when (to - from) / step rounds up
    // past a whole number.
    let n = ((to - from) / step - 1e-9).ceil().max(0.);
    if n > MAX_ITERATIONS as f64 {
        return Err(format!(
            "{} iterations is more than the maximum {}",
            n, MAX_ITERATIONS
        ));
    }
    Ok((0..n as usize).map(|k| from + k as f64 * step).collect())
}

impl Creator<Vec<Hittables>> for Repeat {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, Box<dyn Error>> {
        let values = (|| {
            if ctx.scope.contains(&self.variable) {
                return Err(Box::new(ModelError::new(&format!(
                    "`{}` is already defined, use another name for the loop variable",
                    self.variable
                ))) as Box<dyn Error>);
            }
            let from = self.from.eval_or(ctx, 0.)?;
            let to = self.to.eval(ctx)?;
            let step = self.step.eval_or(ctx, 1.)?;
            loop_values(from, to, step).map_err(|e| Box::new(ModelError::new(&e)) as Box<dyn Error>)
        })()
        .at(|| "repeat".to_string())?;

        let mut list = Vec::new();
        for v in values {
            let scope = ctx.scope.with(&self.variable, v);
            let child = CreatorContext {
                scope: &scope,
                ..*ctx
            };
            for (i, h) in self.world.iter().enumerate() {
                let mut created = h
                    .create(&child)
                    .at(|| format!("world[{}]", i))
                    .at(|| format!("repeat ({} = {})", self.variable, v))?;
                list.append(&mut created);
            }
            if list.len() > MAX_HITTABLES {
                let err: Box<dyn Error> = Box::new(ModelError::new(&format!(
                    "expands to more than the maximum {} objects",
                    MAX_HITTABLES
                )));
                return Err(err).at(|| "repeat".to_string());
            }
        }
        Ok(list)
    }
}

impl HelpDocumentation for Repeat {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        let mut doc = DocumentationStructure {
            description: "Creates the hittables in its world once for each value of a loop variable, which expressions in the world can use. E.g. variable: i, to: 10 and center: i * 2, 0, 0 places ten spheres in a row".to_string(),
            fields: HashMap::from([
                (
                    "variable".to_string(),
                    FieldInfo::new_simple(
                        "Name of the loop variable",
                        Normal,
                        "A name made of letters, digits and _",
                    ),
                ),
                (
                    "from".to_string(),
                    FieldInfo::new_simple(
                        "First value of the loop variable. Defaults to 0",
                        Optional,
                        "Number or expression",
                    ),
                ),
                (
                    "to".to_string(),
                    FieldInfo::new_simple(
                        "The loop stops before reaching this value",
                        Normal,
                        "Number or expression",
                    ),
                ),
                (
                    "step".to_string(),
                    FieldInfo::new_simple(
                        "How much the loop variable changes each iteration. Negative counts down. Defaults to 1",
                        Optional,
                        "Number or expression",
                    ),
                ),
            ]),
        };
        // Hittable and Repeat document each other
        if depth < 8 {
            doc.fields.insert(
                "world".to_string(),
                FieldInfo::new(
                    "The hittables to repeat",
                    List,
                    Hittable::get_documentation_structure(depth + 1),
                ),
            );
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_values_exclude_to() {
        assert_eq!(Ok(vec![0., 1., 2.]), loop_values(0., 3., 1.));
        assert_eq!(Ok(vec![3., 2., 1.]), loop_values(3., 0., -1.));
        assert_eq!(Ok(vec![]), loop_values(3., 0., 1.));
        assert_eq!(3, loop_values(0., 0.3, 0.1).unwrap().len());
        assert_eq!(10, loop_values(0., 1., 0.1).unwrap().len());
        assert!((loop_values(0., 1., 0.1).unwrap()[9] - 0.9).abs() < 1e-12);
        assert!(loop_values(0., 1., 0.).is_err());
        assert!(loop_values(0., 1e9, 1.).is_err());
    }

    #[test]
    fn variable_name_is_checked() {
        let err = serde_yaml::from_str::<Repeat>("variable: pi\nto: 3\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("built-in"), "{}", err);
    }
}
