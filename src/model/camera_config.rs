use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::FieldType::{Normal, Optional};
use crate::model::num::{Num, visit_nums};
use crate::model::pos::Pos;
use crate::model::scope::Scope;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};

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

impl HelpDocumentation for CameraConfig {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Describes the location, orientation and other properties of the camera in the scene".to_string(),
            fields: HashMap::from([
                ("vertical_fov_degrees".to_string(), FieldInfo::new_simple(
                    "Field of view for the camera in degrees. Defaults to 60",
                    Optional,
                    "Amount of vertical field of view for the camera"
                )),
                ("aperture_size".to_string(), FieldInfo::new_simple(
                    "Aperture is defined by the size of the opening through which light can enter the camera. A higher value gives a more shallow depth of field",
                    Optional,
                    "The radius of the aperture. Defaults to 0"
                )),
                ("look_from".to_string(), FieldInfo::new(
                    "Position where the camera is located",
                    Normal,
                    Pos::get_documentation_structure(depth + 1)
                )),
                ("look_at".to_string(), FieldInfo::new(
                    "Position the camera is pointed at. Defaults to 0, 0, 0",
                    Optional,
                    Pos::get_documentation_structure(depth + 1)
                )),
                ("up".to_string(), FieldInfo::new(
                    "A vector pointing in the 'up' direction of the camera. Default to 0, 1, 0 to have y pointing upwards",
                    Optional,
                    Pos::get_documentation_structure(depth + 1)
                ))
            ]),
        }
    }
}
