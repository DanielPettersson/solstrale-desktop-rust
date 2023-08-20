use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::model::{Creator, Pos};

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