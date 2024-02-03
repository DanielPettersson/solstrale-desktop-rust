use crate::model::constant_medium::ConstantMedium;
use crate::model::obj_model::ObjModel;
use crate::model::quad::Quad;
use crate::model::r#box::Box;
use crate::model::sphere::Sphere;
use crate::model::FieldType::Optional;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError};
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use std::collections::HashMap;
use std::error::Error;

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
            _ => Err(From::from(ModelError::new(
                "Hittable should have single field defined",
            ))),
        }
    }
}

impl HelpDocumentation for Hittable {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "Objects that are hittable by rays shot by the ray tracer".to_string(),
            fields: HashMap::from([
                ("sphere".to_string(), FieldInfo::new(
                    "A sphere object",
                    Optional,
                    Sphere::get_documentation_structure()
                )),
                ("model".to_string(), FieldInfo::new(
                    "A model is loaded from an .obj file. And contains a 3d model composed by triangles with materials",
                    Optional,
                    ObjModel::get_documentation_structure()
                )),
                ("quad".to_string(), FieldInfo::new(
                    "A quad is a flat rectangular object",
                    Optional,
                    Quad::get_documentation_structure()
                )),
                ("box".to_string(), FieldInfo::new(
                    "A cuboid object consisting of 6 quads",
                    Optional,
                    Box::get_documentation_structure()
                )),
                ("constant_medium".to_string(), FieldInfo::new(
                    "A box shaped hittable object with a fog-type material",
                    Optional,
                    ConstantMedium::get_documentation_structure()
                )),
            ]),
        }
    }
}
