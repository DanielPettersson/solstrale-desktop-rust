use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::hittable::{Bvh, Hittables};

use crate::model::camera_config::CameraConfig;
use crate::model::hittable::Hittable;
use crate::model::num::{VisitNums, visit_nums};
use crate::model::render_config::RenderConfig;
use crate::model::rgb::Rgb;
use crate::model::scope::Scope;
use crate::model::variables::Variables;
use crate::model::{Creator, CreatorContext, ErrorPath};

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

    /// Whether anything in the scene changes between frames
    pub fn uses_frame_index(&self) -> bool {
        self.free_vars().contains("frameIndex")
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
