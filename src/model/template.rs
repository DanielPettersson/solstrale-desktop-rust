use serde_json::{from_value, to_value, Value};
use std::collections::HashMap;
use std::error::Error;
use tera::Tera;

pub fn apply_template(templated_yaml: &str) -> Result<String, Box<dyn Error>> {
    let mut tera = Tera::default();
    tera.register_function("sin", sin);
    tera.register_function("cos", cos);
    tera.register_function("sqrt", sqrt);
    tera.register_function("range", range);
    tera.add_raw_template("template", templated_yaml)?;
    let context = tera::Context::new();
    let yaml = tera.render("template", &context)?;
    Ok(yaml)
}

pub fn sin(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = match args.get("v") {
        Some(val) => match from_value::<f64>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `sin` received v={val} but `v` can only be a number"
                )));
            }
        },
        None => {
            return Err(tera::Error::msg(
                "Function `sin` was called without a `v` argument",
            ));
        }
    };
    Ok(to_value(v.sin()).unwrap())
}

pub fn cos(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = match args.get("v") {
        Some(val) => match from_value::<f64>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `cos` received v={val} but `v` can only be a number"
                )));
            }
        },
        None => {
            return Err(tera::Error::msg(
                "Function `cos` was called without a `v` argument",
            ));
        }
    };
    Ok(to_value(v.cos()).unwrap())
}

pub fn sqrt(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = match args.get("v") {
        Some(val) => match from_value::<f64>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `sqrt` received v={val} but `v` can only be a number"
                )));
            }
        },
        None => {
            return Err(tera::Error::msg(
                "Function `sqrt` was called without a `v` argument",
            ));
        }
    };
    Ok(to_value(v.sqrt()).unwrap())
}

pub fn range(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let start = match args.get("start") {
        Some(val) => match from_value::<f64>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `range` received start={val} but `start` can only be a number"
                )));
            }
        },
        None => 0.,
    };
    let step_by = match args.get("step_by") {
        Some(val) => match from_value::<f64>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `range` received step_by={val} but `step` can only be a number"
                )));
            }
        },
        None => 1.,
    };
    let end = match args.get("end") {
        Some(val) => match from_value::<f64>(val.clone()) {
            Ok(v) => v,
            Err(_) => {
                return Err(tera::Error::msg(format!(
                    "Function `range` received end={val} but `end` can only be a number"
                )));
            }
        },
        None => {
            return Err(tera::Error::msg(
                "Function `range` was called without a `end` argument",
            ));
        }
    };

    if start > end {
        return Err(tera::Error::msg(
            "Function `range` was called with a `start` argument greater than the `end` one",
        ));
    }

    let mut i = start;
    let mut res = vec![];
    while i < end {
        res.push(i);
        i += step_by;
    }
    Ok(to_value(res).unwrap())
}
