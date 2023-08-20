use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, Pos};
use crate::model::FieldType::Normal;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CameraConfig {
    pub vertical_fov_degrees: f64,
    pub aperture_size: f64,
    pub look_from: Pos,
    pub look_at: Pos,
    pub up: Pos,
}

impl Creator<solstrale::camera::CameraConfig> for CameraConfig {
    fn create(&self) -> Result<solstrale::camera::CameraConfig, Box<dyn Error>> {
        Ok(solstrale::camera::CameraConfig {
            vertical_fov_degrees: self.vertical_fov_degrees,
            aperture_size: self.aperture_size,
            look_from: self.look_from.create()?,
            look_at: self.look_at.create()?,
            up: self.up.create()?,
        })
    }
}

impl HelpDocumentation for CameraConfig {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "<<CameraConfig>>".to_string(),
            fields: HashMap::from([
                ("vertical_fov_degrees".to_string(), FieldInfo::new_simple("<<vertical_fov_degrees>>", Normal, "<<f64>>")),
                ("aperture_size".to_string(), FieldInfo::new_simple("<<aperture_size>>", Normal, "<<f64>>")),
                ("look_from".to_string(), FieldInfo::new("<<look_from>>", Normal, Pos::get_documentation_structure())),
                ("look_at".to_string(), FieldInfo::new("<<look_at>>", Normal, Pos::get_documentation_structure())),
                ("up".to_string(), FieldInfo::new("<<up>>", Normal, Pos::get_documentation_structure()))
            ]),
        }
    }
}