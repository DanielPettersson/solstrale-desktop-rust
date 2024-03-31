use std::collections::HashMap;
use std::error::Error;

use serde_json::{from_value, to_value, Value};
use solstrale::geo::vec3::Vec3;
use tera::Tera;

pub fn apply_template(templated_yaml: &str, frame_index: usize) -> Result<String, Box<dyn Error>> {
    let mut tera = Tera::default();

    tera.register_function("sin", sin);
    tera.register_function("cos", cos);
    tera.register_function("sqrt", sqrt);
    tera.register_function("abs", abs);
    tera.register_function("len", len);
    tera.register_function("range", range);

    tera.add_raw_template("template", templated_yaml)?;

    let mut context = tera::Context::new();
    context.try_insert("frameIndex", &frame_index)?;

    Ok(tera.render("template", &context)?)
}

pub fn sin(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = get_required_numeric_arg(args, "sin", "v")?;
    Ok(to_value(v.sin()).unwrap())
}

pub fn cos(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = get_required_numeric_arg(args, "cos", "v")?;
    Ok(to_value(v.cos()).unwrap())
}

pub fn abs(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = get_required_numeric_arg(args, "abs", "v")?;
    Ok(to_value(v.abs()).unwrap())
}

pub fn len(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let x = get_optional_numeric_arg(args, "len", "x", 0.)?;
    let y = get_optional_numeric_arg(args, "len", "y", 0.)?;
    let z = get_optional_numeric_arg(args, "len", "z", 0.)?;

    Ok(to_value(Vec3::new(x, y, z).length()).unwrap())
}

pub fn sqrt(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v = get_required_numeric_arg(args, "sqrt", "v")?;

    if v < 0. {
        return Err(tera::Error::msg(format!(
            "Function `sqrt` was called with negative `v` argument: {v}"
        )));
    }

    Ok(to_value(v.sqrt()).unwrap())
}

pub fn range(args: &HashMap<String, Value>) -> tera::Result<Value> {
    let start = get_optional_numeric_arg(args, "range", "start", 0.)?;
    let end = get_required_numeric_arg(args, "range", "end")?;
    let step_by = get_optional_numeric_arg(args, "range", "step_by", 1.)?;

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

fn get_required_numeric_arg(
    args: &HashMap<String, Value>,
    fn_name: &str,
    arg_name: &str,
) -> tera::Result<f64> {
    match args.get(arg_name) {
        Some(val) => value_to_numeric(fn_name, arg_name, val),
        None => Err(tera::Error::msg(format!(
            "Function `{fn_name}` was called without a `{arg_name}` argument"
        ))),
    }
}

fn get_optional_numeric_arg(
    args: &HashMap<String, Value>,
    fn_name: &str,
    arg_name: &str,
    default_val: f64,
) -> tera::Result<f64> {
    match args.get(arg_name) {
        Some(val) => value_to_numeric(fn_name, arg_name, val),
        None => Ok(default_val),
    }
}

fn value_to_numeric(fn_name: &str, arg_name: &str, val: &Value) -> tera::Result<f64> {
    match from_value::<f64>(val.clone()) {
        Ok(v) => Ok(v),
        Err(_) => Err(tera::Error::msg(format!(
            "Function `{fn_name}` received end={val} but `{arg_name}` can only be a number"
        ))),
    }
}
