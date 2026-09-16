use std::error::Error;

use solstrale::geo::vec3::Vec3;
use tera::{Kwargs, State, Tera, TeraResult};

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
    context.insert("frameIndex", &frame_index);

    Ok(tera.render("template", &context)?)
}

pub fn sin(kwargs: Kwargs, _: &State) -> TeraResult<f64> {
    Ok(kwargs.must_get::<f64>("v")?.sin())
}

pub fn cos(kwargs: Kwargs, _: &State) -> TeraResult<f64> {
    Ok(kwargs.must_get::<f64>("v")?.cos())
}

pub fn abs(kwargs: Kwargs, _: &State) -> TeraResult<f64> {
    Ok(kwargs.must_get::<f64>("v")?.abs())
}

pub fn len(kwargs: Kwargs, _: &State) -> TeraResult<f64> {
    let x = kwargs.get::<f64>("x")?.unwrap_or_default();
    let y = kwargs.get::<f64>("y")?.unwrap_or_default();
    let z = kwargs.get::<f64>("z")?.unwrap_or_default();

    Ok(Vec3::new(x, y, z).length())
}

pub fn sqrt(kwargs: Kwargs, _: &State) -> TeraResult<f64> {
    let v = kwargs.must_get::<f64>("v")?;

    if v < 0. {
        return Err(tera::Error::message(format!(
            "Function `sqrt` was called with negative `v` argument: {v}"
        )));
    }

    Ok(v.sqrt())
}

pub fn range(kwargs: Kwargs, _: &State) -> TeraResult<Vec<f64>> {
    let start = kwargs.get::<f64>("start")?.unwrap_or_default();
    let end = kwargs.must_get::<f64>("end")?;
    let step_by = kwargs.get::<f64>("step_by")?.unwrap_or(1.);

    if start > end {
        return Err(tera::Error::message(
            "Function `range` was called with a `start` argument greater than the `end` one",
        ));
    }

    let mut i = start;
    let mut res = vec![];
    while i < end {
        res.push(i);
        i += step_by;
    }
    Ok(res)
}
