use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::Bvh;

use crate::model::{Creator, DocumentationStructure, FieldInfo, HelpDocumentation};
use crate::model::camera_config::CameraConfig;
use crate::model::FieldType::{List, Normal};
use crate::model::hittable::Hittable;
use crate::model::render_config::RenderConfig;
use crate::model::rgb::Rgb;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub render_configuration: RenderConfig,
    pub background_color: Rgb,
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

impl HelpDocumentation for Scene {
    fn get_documentation_structure() -> DocumentationStructure {
        DocumentationStructure {
            description: "The scene YAML is used to configure all aspects of the rendered image.\n\n\
            Use ctrl+space to autocomplete configuration keys and ctrl+r to restart the rendering".to_string(),
            fields: HashMap::from([
                ("render_configuration".to_string(), FieldInfo::new(
                    "General configuration for the renderer",
                    Normal,
                    RenderConfig::get_documentation_structure()
                )),
                ("background_color".to_string(), FieldInfo::new(
                    "The resulting pixel color for when a ray hits nothing",
                    Normal,
                    Rgb::get_documentation_structure()
                )),
                ("camera".to_string(), FieldInfo::new(
                    "Describes the camera used in the scene",
                    Normal,
                    CameraConfig::get_documentation_structure()
                )),
                ("world".to_string(), FieldInfo::new(
                    "Contains all hittable objects that are visible in the scene",
                    List,
                    Hittable::get_documentation_structure()
                )),
            ]),
        }
    }
}
