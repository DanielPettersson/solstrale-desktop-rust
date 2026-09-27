use std::error::Error;

use serde::{Deserialize, Serialize};
use solstrale::geo::vec3::Vec3;
use solstrale::hittable::{Bvh, Hittables};
use solstrale::post::PostProcessors;

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
    pub fn create_hittables(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, Box<dyn Error>> {
        let mut list = Vec::new();
        for (i, child) in self.world.iter().enumerate() {
            list.append(&mut child.create(ctx).at(|| format!("world[{}]", i))?)
        }
        Ok(list)
    }

    // The parts of the scene the renderer takes separately, each in a context
    // that already has [`Scene::scope`]

    pub fn create_world(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        Ok(Bvh::new(self.create_hittables(ctx)?).into())
    }

    pub fn create_camera(
        &self,
        ctx: &CreatorContext,
    ) -> Result<solstrale::camera::CameraConfig, Box<dyn Error>> {
        self.camera.create(ctx).at(|| "camera".to_string())
    }

    pub fn create_background_color(&self, ctx: &CreatorContext) -> Result<Vec3, Box<dyn Error>> {
        self.background_color
            .as_ref()
            .unwrap_or(&Rgb::new(0., 0., 0.))
            .create(ctx)
            .at(|| "background_color".to_string())
    }

    pub fn create_render_config(
        &self,
        ctx: &CreatorContext,
    ) -> Result<solstrale::renderer::RenderConfig, Box<dyn Error>> {
        self.render_configuration
            .as_ref()
            .unwrap_or(&RenderConfig::default())
            .create(ctx)
            .at(|| "render_configuration".to_string())
    }

    pub fn create_post_processors(
        &self,
        ctx: &CreatorContext,
    ) -> Result<Vec<PostProcessors>, Box<dyn Error>> {
        self.render_configuration
            .as_ref()
            .map_or(Ok(Vec::new()), |c| c.create_post_processors(ctx))
            .at(|| "render_configuration".to_string())
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
            world: self.create_world(ctx)?,
            camera: self.create_camera(ctx)?,
            background_color: self.create_background_color(ctx)?,
            render_config: self.create_render_config(ctx)?,
            post_processors: self.create_post_processors(ctx)?,
        })
    }
}
