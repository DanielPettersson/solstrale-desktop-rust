use std::collections::BTreeMap;

/// Names every scope has, which scene variables and loop variables may not reuse.
pub const BUILTINS: &[&str] = &["frameIndex", "pi", "e"];

/// The variables an expression can read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scope {
    vars: BTreeMap<String, f64>,
}

impl Scope {
    pub fn builtin(frame_index: usize) -> Scope {
        Scope::default()
            .with("frameIndex", frame_index as f64)
            .with("pi", std::f64::consts::PI)
            .with("e", std::f64::consts::E)
    }

    pub fn with(&self, name: &str, value: f64) -> Scope {
        let mut vars = self.vars.clone();
        vars.insert(name.to_string(), value);
        Scope { vars }
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        self.vars.get(name).copied()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// The values of the given variables, for keying caches on what an object reads
    pub fn fingerprint<'a>(&self, names: impl IntoIterator<Item = &'a String>) -> String {
        names
            .into_iter()
            .map(|n| format!("{}={:?}", n, self.get(n)))
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Whether `name` can be used as a variable name
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Checks that `name` can be declared as a scene or loop variable
pub fn check_name(name: &str) -> Result<(), String> {
    if !is_identifier(name) {
        Err(format!(
            "`{}` is not a valid name, use letters, digits and _",
            name
        ))
    } else if BUILTINS.contains(&name) {
        Err(format!("`{}` is a built-in variable", name))
    } else if crate::model::expr::FUNCTIONS
        .iter()
        .any(|(f, _, _)| *f == name)
    {
        Err(format!("`{}` is a function name", name))
    } else {
        Ok(())
    }
}
