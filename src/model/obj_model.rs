use std::error::Error;
use moka::sync::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use solstrale::loader::Loader;
use solstrale::loader::obj::Obj;
use solstrale::material::texture::SolidColor;
use crate::model::{Creator, ModelError};
use crate::model::material::Material;
use crate::model::transformation::{create_transformation, Transformation};

static MODEL_CACHE: Lazy<Cache<String, Result<Hittables, ModelError>>> =
    Lazy::new(|| Cache::new(4));


#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ObjModel {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transformations: Vec<Transformation>,
}

impl Creator<Hittables> for ObjModel {
    fn create(&self) -> Result<Hittables, Box<dyn Error>> {
        let material = self.material.as_ref().map_or(
            Ok(solstrale::material::Lambertian::new(
                SolidColor::new(1., 1., 1.),
                None,
            )),
            |m| m.create(),
        )?;
        let transformation = create_transformation(&self.transformations)?;

        let key = format!("{:?}", self);
        let model_result = MODEL_CACHE.get_with(key.to_owned(), || {
            Obj::new(&self.path, &self.name)
                .load(&transformation, Some(material))
                .map_err(|err| ModelError::new_from_err(err))
        });

        match model_result {
            Ok(model) => Ok(model),
            Err(err) => {
                MODEL_CACHE.remove(&key);
                Err(Box::new(err))
            }
        }
    }
}
