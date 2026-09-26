use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::num::{Num, visit_nums};
use crate::model::pos::Pos;
use crate::model::scope::Scope;
use crate::model::{Creator, CreatorContext, ModelError};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CameraConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_fov_degrees: Option<Num>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aperture_size: Option<Num>,
    pub look_from: Pos,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub look_at: Option<Pos>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up: Option<Pos>,
}

visit_nums!(
    CameraConfig,
    vertical_fov_degrees,
    aperture_size,
    look_from,
    look_at,
    up
);

impl Creator<solstrale::camera::CameraConfig> for CameraConfig {
    fn create(
        &self,
        ctx: &CreatorContext,
    ) -> Result<solstrale::camera::CameraConfig, Box<dyn Error>> {
        self.eval(ctx.scope)
    }
}

impl CameraConfig {
    /// Whether where the camera is or looks is computed by an expression
    pub fn position_uses_expressions(&self) -> bool {
        let has_expr = |p: &Pos| [&p.x, &p.y, &p.z].iter().any(|n| matches!(n, Num::Expr(_)));
        has_expr(&self.look_from) || self.look_at.as_ref().is_some_and(has_expr)
    }

    /// The camera with its expressions evaluated. Needs no GPU, unlike `create`.
    pub fn eval(&self, scope: &Scope) -> Result<solstrale::camera::CameraConfig, Box<dyn Error>> {
        let num =
            |n: &Option<Num>, default: f64| n.as_ref().map_or(Ok(default), |n| n.eval_scope(scope));
        let pos = |p: &Option<Pos>, default: Pos| p.as_ref().unwrap_or(&default).eval(scope);
        let camera = (|| {
            Ok(solstrale::camera::CameraConfig {
                vertical_fov_degrees: num(&self.vertical_fov_degrees, 60.)?,
                aperture_size: num(&self.aperture_size, 0.)?,
                look_from: self.look_from.eval(scope)?,
                look_at: pos(&self.look_at, Pos::default())?,
                up: pos(&self.up, Pos::new(0., 1., 0.))?,
            })
        })();
        camera.map_err(|e: String| Box::new(ModelError::new(&e)) as Box<dyn Error>)
    }
}
