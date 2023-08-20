use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::geo::transformation::{RotationX, RotationY, RotationZ, Scale, Transformations, Transformer, Translation};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use crate::model::FieldType::Optional;
use crate::model::pos::Pos;

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
            _ => Err(Box::try_from(ModelError::new(
                "Transformation should have single field defined",
            ))
                .unwrap()),
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
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<Transformation>>".to_string(),
            fields: HashMap::from([
                ("translation".to_string(), FieldInfo::new("<<translation>>", Optional, Pos::get_documentation_structure())),
                ("scale".to_string(), FieldInfo::new_simple("<<scale>>", Optional, "<<f64>>")),
                ("rotation_x".to_string(), FieldInfo::new_simple("<<rotation_x>>", Optional, "<<f64>>")),
                ("rotation_y".to_string(), FieldInfo::new_simple("<<rotation_y>>", Optional, "<<f64>>")),
                ("rotation_z".to_string(), FieldInfo::new_simple("<<rotation_z>>", Optional, "<<f64>>")),
            ]),
        }
    }
}
