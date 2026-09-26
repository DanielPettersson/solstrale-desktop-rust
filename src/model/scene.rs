use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::{Bvh, Hittables};

use crate::model::FieldType::{List, Normal, Optional};
use crate::model::camera_config::CameraConfig;
use crate::model::hittable::Hittable;
use crate::model::num::visit_nums;
use crate::model::render_config::RenderConfig;
use crate::model::rgb::Rgb;
use crate::model::scope::Scope;
use crate::model::variables::Variables;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, ErrorPath, FieldInfo, HelpDocumentation,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    #[serde(skip_serializing_if = "Variables::is_empty", default)]
    pub variables: Variables,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_configuration: Option<RenderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Rgb>,
    pub camera: CameraConfig,
    pub world: Vec<Hittable>,
}

visit_nums!(
    Scene,
    variables,
    render_configuration,
    background_color,
    camera,
    world
);

impl Scene {
    /// The scope the scene's contents are evaluated in: `base` plus the
    /// scene variables
    pub fn scope(&self, base: &Scope) -> Result<Scope, Box<dyn Error>> {
        self.variables.scope(base)
    }

    /// The camera as it is at the given frame
    pub fn camera_at(
        &self,
        frame_index: usize,
    ) -> Result<solstrale::camera::CameraConfig, Box<dyn Error>> {
        let scope = self.scope(&Scope::builtin(frame_index))?;
        self.camera.eval(&scope).at(|| "camera".to_string())
    }

    /// The hittables of the world, in a context that already has [`Scene::scope`]
    pub fn create_world(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, Box<dyn Error>> {
        let mut list = Vec::new();
        for (i, child) in self.world.iter().enumerate() {
            list.append(&mut child.create(ctx).at(|| format!("world[{}]", i))?)
        }
        Ok(list)
    }
}

impl Creator<solstrale::renderer::Scene> for Scene {
    fn create(&self, ctx: &CreatorContext) -> Result<solstrale::renderer::Scene, Box<dyn Error>> {
        let scope = self.scope(ctx.scope)?;
        let ctx = &CreatorContext {
            scope: &scope,
            ..*ctx
        };

        Ok(solstrale::renderer::Scene {
            world: Bvh::new(self.create_world(ctx)?).into(),
            camera: self.camera.create(ctx).at(|| "camera".to_string())?,
            background_color: self
                .background_color
                .as_ref()
                .unwrap_or(&Rgb::new(0., 0., 0.))
                .create(ctx)
                .at(|| "background_color".to_string())?,
            render_config: self
                .render_configuration
                .as_ref()
                .unwrap_or(&RenderConfig::default())
                .create(ctx)
                .at(|| "render_configuration".to_string())?,
        })
    }
}

impl HelpDocumentation for Scene {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description:
                "The scene YAML is used to configure all aspects of the rendered image.\n\n\
            Numbers can be written as expressions, e.g. radius: sqrt(2) / 2 or center: i * 2, 0, sin(frameIndex * 0.1). \
            Expressions support + - * / % ^ and parentheses, and the functions \
            sin cos tan abs sqrt floor round pow(x, y) min(...) max(...) len(x, y, z).\n\n\
            They can read the built-in variables frameIndex (the frame number when batch rendering), pi and e, \
            the scene variables and the loop variables of enclosing repeats.\n\n\
            Use ctrl+space to autocomplete configuration keys and ctrl+r to restart the rendering\n\n\
            Progress bar shows percentage completed, remaining time, FPS (frames rendered per second) and MPPS (Million pixel samples rendered per second)"
                    .to_string(),
            fields: HashMap::from([
                (
                    "variables".to_string(),
                    FieldInfo::new_simple(
                        "Named values that expressions in the scene can use. Each can use the ones declared before it",
                        Optional,
                        "A map of name: number or expression, e.g. spacing: 2.5",
                    ),
                ),
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
