use crate::model::FieldType::Normal;
use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation, Pos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

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
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Describes the location, orientation and other properties of the camera in the scene".to_string(),
            fields: HashMap::from([
                ("vertical_fov_degrees".to_string(), FieldInfo::new_simple(
                    "Field of view for the camera in degrees",
                    Normal,
                    "Amount of vertical field of view for the camera"
                )),
                ("aperture_size".to_string(), FieldInfo::new_simple(
                    "Aperture is defined by the size of the opening through which light can enter the camera. A higher value gives a more shallow depth of field",
                    Normal,
                    "The radius of the aperture"
                )),
                ("look_from".to_string(), FieldInfo::new(
                    "Position where the camera is located",
                    Normal,
                    Pos::get_documentation_structure(depth + 1)
                )),
                ("look_at".to_string(), FieldInfo::new(
                    "Position the camera is pointed at",
                    Normal,
                    Pos::get_documentation_structure(depth + 1)
                )),
                ("up".to_string(), FieldInfo::new(
                    "A vector pointing in the 'up' direction of the camera",
                    Normal,
                    Pos::get_documentation_structure(depth + 1)
                ))
            ]),
        }
    }
}
