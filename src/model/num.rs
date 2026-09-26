use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::expr::{Expr, ParseError};
use crate::model::scope::Scope;
use crate::model::{CreatorContext, ModelError};

/// A number in the scene, either written out or computed from an expression.
#[derive(Clone, Debug, PartialEq)]
pub enum Num {
    Lit(f64),
    Expr(Expr),
}

impl Num {
    /// Reads `s` as a number if it is one, else as an expression
    pub fn parse(s: &str) -> Result<Num, ParseError> {
        let s = s.trim();
        match s.parse::<f64>() {
            Ok(n) => Ok(Num::Lit(n)),
            Err(_) => Expr::parse(s).map(Num::Expr),
        }
    }

    pub fn eval_scope(&self, scope: &Scope) -> Result<f64, String> {
        match self {
            Num::Lit(n) => Ok(*n),
            Num::Expr(e) => e
                .eval(scope)
                .map_err(|err| format!("{} in `{}`", err, e.src())),
        }
    }

    pub fn eval(&self, ctx: &CreatorContext) -> Result<f64, Box<dyn Error>> {
        self.eval_scope(ctx.scope)
            .map_err(|e| Box::new(ModelError::new(&e)) as Box<dyn Error>)
    }

    pub fn free_vars(&self, vars: &mut BTreeSet<String>) {
        if let Num::Expr(e) = self {
            vars.extend(e.free_vars());
        }
    }
}

impl From<f64> for Num {
    fn from(n: f64) -> Self {
        Num::Lit(n)
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Num::Lit(n) => write!(f, "{}", n),
            Num::Expr(e) => write!(f, "{}", e.src()),
        }
    }
}

/// Evaluation of optional numbers that fall back to a default
pub trait EvalOr {
    fn eval_or(&self, ctx: &CreatorContext, default: f64) -> Result<f64, Box<dyn Error>>;
    fn eval_opt(&self, ctx: &CreatorContext) -> Result<Option<f64>, Box<dyn Error>>;
}

impl EvalOr for Option<Num> {
    fn eval_opt(&self, ctx: &CreatorContext) -> Result<Option<f64>, Box<dyn Error>> {
        self.as_ref().map(|n| n.eval(ctx)).transpose()
    }

    fn eval_or(&self, ctx: &CreatorContext, default: f64) -> Result<f64, Box<dyn Error>> {
        match self {
            Some(n) => n.eval(ctx),
            None => Ok(default),
        }
    }
}

impl Serialize for Num {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Num::Lit(n) => s.serialize_f64(*n),
            Num::Expr(e) => s.serialize_str(e.src()),
        }
    }
}

impl<'de> Deserialize<'de> for Num {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Num, D::Error> {
        struct NumVisitor;

        impl Visitor<'_> for NumVisitor {
            type Value = Num;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number or an expression")
            }

            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Num, E> {
                Ok(Num::Lit(v))
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Num, E> {
                Ok(Num::Lit(v as f64))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Num, E> {
                Ok(Num::Lit(v as f64))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Num, E> {
                Num::parse(v).map_err(|e| E::custom(format!("{} in `{}`", e, v)))
            }
        }

        d.deserialize_any(NumVisitor)
    }
}

/// Splits `"a, b, c"` into its parts, ignoring commas inside parentheses so
/// that components can be function calls with several arguments.
pub fn split_components(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(s[start..].trim());
    parts
}

/// Parses the three comma separated components of a vector or colour
pub fn parse_triple<E: de::Error>(s: &str) -> Result<[Num; 3], E> {
    let parts = split_components(s);
    if parts.len() != 3 {
        return Err(E::custom(format!(
            "expected 3 comma separated values, got {} in `{}`",
            parts.len(),
            s
        )));
    }
    let parse = |p: &str| Num::parse(p).map_err(|e| E::custom(format!("{} in `{}`", e, p)));
    Ok([parse(parts[0])?, parse(parts[1])?, parse(parts[2])?])
}

/// Walks every number in a model value, e.g. to find the variables it reads.
pub trait VisitNums {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num));

    fn free_vars(&self) -> BTreeSet<String> {
        let mut vars = BTreeSet::new();
        self.visit_nums(&mut |n| n.free_vars(&mut vars));
        vars
    }
}

impl VisitNums for Num {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        f(self)
    }
}

impl<T: VisitNums> VisitNums for Option<T> {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        if let Some(v) = self {
            v.visit_nums(f)
        }
    }
}

impl<T: VisitNums> VisitNums for Vec<T> {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        self.iter().for_each(|v| v.visit_nums(f))
    }
}

impl<T: VisitNums> VisitNums for Box<T> {
    fn visit_nums(&self, f: &mut dyn FnMut(&Num)) {
        self.as_ref().visit_nums(f)
    }
}

/// Implements [`VisitNums`] for a struct by visiting the listed fields.
macro_rules! visit_nums {
    ($ty:ty $(, $field:ident)* $(,)?) => {
        impl $crate::model::num::VisitNums for $ty {
            #[allow(unused_variables)]
            fn visit_nums(&self, f: &mut dyn FnMut(&$crate::model::num::Num)) {
                $($crate::model::num::VisitNums::visit_nums(&self.$field, f);)*
            }
        }
    };
}

pub(crate) use visit_nums;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ignores_commas_in_calls() {
        assert_eq!(vec!["1", "2", "3"], split_components("1, 2,3"));
        assert_eq!(
            vec!["len(1, 2, 3)", "i * 2", "-1"],
            split_components("len(1, 2, 3), i * 2, -1")
        );
    }

    #[test]
    fn numbers_read_as_literals() {
        assert_eq!(Num::Lit(30.), serde_yaml::from_str::<Num>("30").unwrap());
        assert_eq!(
            Num::Lit(30.),
            serde_yaml::from_str::<Num>("\"30\"").unwrap()
        );
        assert_eq!(Num::Lit(-0.5), serde_yaml::from_str::<Num>("-0.5").unwrap());
    }

    #[test]
    fn expressions_round_trip_unquoted() {
        let n: Num = serde_yaml::from_str("sin(frameIndex * 0.1) * 2").unwrap();
        assert!(matches!(n, Num::Expr(_)));
        assert_eq!(
            "sin(frameIndex * 0.1) * 2\n",
            serde_yaml::to_string(&n).unwrap()
        );
    }

    #[test]
    fn invalid_expression_is_a_load_error() {
        let err = serde_yaml::from_str::<Num>("1 +").unwrap_err().to_string();
        assert!(err.contains("expected a value"), "{}", err);
    }

    #[test]
    fn triple_needs_three_parts() {
        let err = parse_triple::<serde_yaml::Error>("1, 2").unwrap_err();
        assert!(err.to_string().contains("expected 3"), "{}", err);
    }
}
