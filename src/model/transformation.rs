use crate::model::pos::Pos;
use crate::model::FieldType::Optional;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use serde::{Deserialize, Serialize};
use solstrale::geo::transformation::{
    RotationX, RotationY, RotationZ, Scale, Transformations, Transformer, Translation,
};
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Transformation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<Pos>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_z: Option<f64>,
}

impl Creator<Box<dyn Transformer>> for Transformation {
    fn create(&self) -> Result<Box<dyn Transformer>, Box<dyn Error>> {
        match self {
            Transformation {
                translation: Some(p),
                scale: None,
                rotation_x: None,
                rotation_y: None,
                rotation_z: None,
            } => Ok(Box::new(Translation::new(p.into()))),
            Transformation {
                translation: None,
                scale: Some(s),
                rotation_x: None,
                rotation_y: None,
                rotation_z: None,
            } => Ok(Box::new(Scale::new(*s))),
            Transformation {
                translation: None,
                scale: None,
                rotation_x: Some(r),
                rotation_y: None,
                rotation_z: None,
            } => Ok(Box::new(RotationX::new(*r))),
            Transformation {
                translation: None,
                scale: None,
                rotation_x: None,
                rotation_y: Some(r),
                rotation_z: None,
            } => Ok(Box::new(RotationY::new(*r))),
            Transformation {
                translation: None,
                scale: None,
                rotation_x: None,
                rotation_y: None,
                rotation_z: Some(r),
            } => Ok(Box::new(RotationZ::new(*r))),
            _ => Err(From::from(ModelError::new(
                "Transformation should have single field defined",
            ))),
        }
    }
}

pub fn create_transformation(
    transformations: &Vec<Transformation>,
) -> Result<Transformations, Box<dyn Error>> {
    let mut trans: Vec<Box<dyn Transformer>> = Vec::with_capacity(transformations.len());
    for t in transformations {
        trans.push(t.create()?);
    }
    Ok(Transformations::new(trans))
}

impl HelpDocumentation for Transformation {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Changes a hittables position, rotation and / or size".to_string(),
            fields: HashMap::from([
                (
                    "translation".to_string(),
                    FieldInfo::new(
                        "Moves the hittable by the given offset",
                        Optional,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "scale".to_string(),
                    FieldInfo::new_simple(
                        "Scales the hittable uniformly by the given factor",
                        Optional,
                        "Scaling factor",
                    ),
                ),
                (
                    "rotation_x".to_string(),
                    FieldInfo::new_simple(
                        "Rotates the hittable around the X axis",
                        Optional,
                        "Rotation in degrees",
                    ),
                ),
                (
                    "rotation_y".to_string(),
                    FieldInfo::new_simple(
                        "Rotates the hittable around the Y axis",
                        Optional,
                        "Rotation in degrees",
                    ),
                ),
                (
                    "rotation_z".to_string(),
                    FieldInfo::new_simple(
                        "Rotates the hittable around the Z axis",
                        Optional,
                        "Rotation in degrees",
                    ),
                ),
            ]),
        }
    }
}
