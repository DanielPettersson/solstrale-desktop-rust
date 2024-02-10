use crate::model::material::Material;
use crate::model::transformation::{create_transformation, Transformation};
use crate::model::FieldType::{List, Normal, Optional};
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};
use moka::sync::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use solstrale::hittable::Hittables;
use solstrale::loader::obj::Obj;
use solstrale::loader::Loader;
use solstrale::material::texture::SolidColor;
use std::collections::HashMap;
use std::error::Error;

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
    fn create(&self, ctx: &CreatorContext) -> Result<Hittables, Box<dyn Error>> {
        let material = self.material.as_ref().map_or(
            Ok(solstrale::material::Lambertian::new(
                SolidColor::new(1., 1., 1.),
                None,
            )),
            |m| m.create(ctx),
        )?;
        let transformation = create_transformation(&self.transformations, ctx)?;

        let key = format!("{:?}", self);
        let model_result = MODEL_CACHE.get_with(key.to_owned(), || {
            Obj::new(&self.path, &self.name)
                .load(&transformation, Some(material))
                .map_err(ModelError::new_from_err)
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

impl HelpDocumentation for ObjModel {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "A model is loaded from an .obj file. And contains a 3d model composed by triangles with materials".to_string(),
            fields: HashMap::from([
                ("path".to_string(), FieldInfo::new_simple(
                    "Path to the folder containing the .obj file",
                    Normal,
                    "Absolute path to the folder containing the .obj file"
                )),
                ("name".to_string(), FieldInfo::new_simple(
                    "File name of the .obj file",
                    Normal,
                    "File name of the .obj file"
                )),
                ("material".to_string(), FieldInfo::new(
                    "The default material used on the model when no material exists in the file",
                    Optional,
                    Material::get_documentation_structure(depth + 1)
                )),
                ("transformations".to_string(), FieldInfo::new(
                    "Transformations to be applied to the position and size of the model",
                    List,
                    Transformation::get_documentation_structure(depth + 1)
                )),
            ]),
        }
    }
}
