use serde::{Deserialize, Serialize};
use std::error::Error;
use solstrale::hittable::{Hittables};
use crate::model::{Creator, ModelError};
use crate::model::constant_medium::ConstantMedium;
use crate::model::r#box::Box;
use crate::model::obj_model::ObjModel;
use crate::model::quad::Quad;
use crate::model::sphere::Sphere;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Hittable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sphere: Option<Sphere>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ObjModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quad: Option<Quad>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#box: Option<Box>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant_medium: Option<ConstantMedium>,
}

impl Creator<Vec<Hittables>> for Hittable {
    fn create(&self) -> Result<Vec<Hittables>, std::boxed::Box<dyn Error>> {
        match self {
            Hittable {
                sphere: Some(s),
                model: None,
                quad: None,
                r#box: None,
                constant_medium: None,
            } => s.create().map(|h| vec![h]),
            Hittable {
                sphere: None,
                model: Some(m),
                quad: None,
                r#box: None,
                constant_medium: None,
            } => m.create().map(|h| vec![h]),
            Hittable {
                sphere: None,
                model: None,
                quad: Some(q),
                r#box: None,
                constant_medium: None,
            } => q.create().map(|h| vec![h]),
            Hittable {
                sphere: None,
                model: None,
                quad: None,
                r#box: Some(b),
                constant_medium: None,
            } => b.create(),
            Hittable {
                sphere: None,
                model: None,
                quad: None,
                r#box: None,
                constant_medium: Some(cm),
            } => cm.create().map(|h| vec![h]),
            _ => Err(std::boxed::Box::try_from(ModelError::new(
                "Hittable should have single field defined",
            ))
                .unwrap()),
        }
    }
}
