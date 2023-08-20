use std::error::Error;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Bvh;
use crate::model::Creator;
use crate::model::camera_config::CameraConfig;
use crate::model::hittable::Hittable;
use crate::model::pos::Pos;
use crate::model::render_config::RenderConfig;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub render_configuration: RenderConfig,
    pub background_color: Pos,
    pub camera: CameraConfig,
    pub world: Vec<Hittable>,
}

impl Creator<solstrale::renderer::Scene> for Scene {
    fn create(&self) -> Result<solstrale::renderer::Scene, Box<dyn Error>> {
        let mut list = Vec::new();
        for child in self.world.iter() {
            list.append(&mut child.create()?)
        }

        Ok(solstrale::renderer::Scene {
            world: Bvh::new(list),
            camera: self.camera.create()?,
            background_color: self.background_color.create()?,
            render_config: self.render_configuration.create()?,
        })
    }
}