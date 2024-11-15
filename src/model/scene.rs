use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::Bvh;

use crate::model::camera_config::CameraConfig;
use crate::model::hittable::Hittable;
use crate::model::render_config::RenderConfig;
use crate::model::rgb::Rgb;
use crate::model::FieldType::{List, Normal, Optional};
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_configuration: Option<RenderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Rgb>,
    pub camera: CameraConfig,
    pub world: Vec<Hittable>,
}

impl Creator<solstrale::renderer::Scene> for Scene {
    fn create(&self, ctx: &CreatorContext) -> Result<solstrale::renderer::Scene, Box<dyn Error>> {
        let mut list = Vec::new();
        for child in self.world.iter() {
            list.append(&mut child.create(ctx)?)
        }

        Ok(solstrale::renderer::Scene {
            world: Bvh::new(list),
            camera: self.camera.create(ctx)?,
            background_color: self
                .background_color
                .unwrap_or(Rgb::new(0., 0., 0.))
                .create(ctx)?,
            render_config: self
                .render_configuration
                .as_ref()
                .unwrap_or(&RenderConfig::default())
                .create(ctx)?,
        })
    }
}

impl HelpDocumentation for Scene {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description:
                "The scene YAML is used to configure all aspects of the rendered image.\n\n\
            To help with repetitive configuration the yaml can be templated using Tera templates. For example: \n\n\
            {% for x in range(end=10) %}\n\
            \x20\x20- sphere:\n\
            \x20\x20\x20\x20\x20\x20center: {{ x }}, 0, 0\n\
            \x20\x20\x20\x20\x20\x20radius: 1\n\
            {% endfor %}\n\n\
            Use ctrl+space to autocomplete configuration keys and ctrl+r to restart the rendering\n\n\
            Progress bar shows percentage completed, remaining time, FPS (frames rendered per second) and MPPS (Million pixel samples rendered per second)"
                    .to_string(),
            fields: HashMap::from([
                (
                    "render_configuration".to_string(),
                    FieldInfo::new(
                        "General configuration for the renderer",
                        Optional,
                        RenderConfig::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "background_color".to_string(),
                    FieldInfo::new(
                        "The resulting pixel color for when a ray hits nothing. Defaults to black",
                        Optional,
                        Rgb::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "camera".to_string(),
                    FieldInfo::new(
                        "Describes the camera used in the scene",
                        Normal,
                        CameraConfig::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "world".to_string(),
                    FieldInfo::new(
                        "Contains all hittable objects that are visible in the scene",
                        List,
                        Hittable::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
