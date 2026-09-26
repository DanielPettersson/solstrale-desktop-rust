use crate::model::FieldType::Optional;
use crate::model::r#box::Box;
use crate::model::obj_model::ObjModel;
use crate::model::one_of::one_of;
use crate::model::quad::Quad;
use crate::model::sphere::Sphere;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use solstrale::hittable::Hittables;
use std::collections::HashMap;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Hittable {
        sphere => Sphere(Sphere),
        model => Model(ObjModel),
        quad => Quad(Quad),
        r#box => Box(Box),
    }
    repr: HittableRepr, HittableReprRef;
    empty: None;
}

impl Creator<Vec<Hittables>> for Hittable {
    fn create(&self, ctx: &CreatorContext) -> Result<Vec<Hittables>, std::boxed::Box<dyn Error>> {
        match self {
            Hittable::Sphere(s) => s.create(ctx).map(|h| vec![h]),
            Hittable::Model(m) => m.create(ctx).map(|h| vec![h]),
            Hittable::Quad(q) => q.create(ctx).map(|h| vec![h]),
            Hittable::Box(b) => b.create(ctx),
        }
    }
}

impl HelpDocumentation for Hittable {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Objects that are hittable by rays shot by the ray tracer".to_string(),
            fields: HashMap::from([
                (
                    "sphere".to_string(),
                    FieldInfo::new(
                        "A sphere object",
                        Optional,
                        Sphere::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "model".to_string(),
                    FieldInfo::new(
                        "A model is loaded from an .obj file. And contains a 3d model composed by triangles with materials",
                        Optional,
                        ObjModel::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "quad".to_string(),
                    FieldInfo::new(
                        "A quad is a flat rectangular object",
                        Optional,
                        Quad::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "box".to_string(),
                    FieldInfo::new(
                        "A cuboid object consisting of 6 quads",
                        Optional,
                        Box::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
