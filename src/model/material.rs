use crate::model::FieldType::Optional;
use crate::model::blend::Blend;
use crate::model::glass::Glass;
use crate::model::lambertian::Lambertian;
use crate::model::light::Light;
use crate::model::metal::Metal;
use crate::model::one_of::one_of;
use crate::model::plastic::Plastic;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use solstrale::material::Materials;
use std::collections::HashMap;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Material {
        lambertian => Lambertian(Lambertian),
        glass => Glass(Glass),
        metal => Metal(Metal),
        plastic => Plastic(Plastic),
        light => Light(Light),
        blend => Blend(Box<Blend>),
    }
    repr: MaterialRepr, MaterialReprRef;
    empty: Some(Material::default());
}

impl Default for Material {
    fn default() -> Self {
        Material::Lambertian(Lambertian::default())
    }
}

impl Creator<Materials> for Material {
    fn create(&self, ctx: &CreatorContext) -> Result<Materials, Box<dyn Error>> {
        match self {
            Material::Lambertian(l) => l.create(ctx),
            Material::Glass(g) => g.create(ctx),
            Material::Metal(m) => m.create(ctx),
            Material::Plastic(p) => p.create(ctx),
            Material::Light(l) => l.create(ctx),
            Material::Blend(b) => b.create(ctx),
        }
    }
}

impl HelpDocumentation for Material {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description:
                "A material gives hittable objects it's looks as they scatter the light differently"
                    .to_string(),
            fields: HashMap::from([
                (
                    "lambertian".to_string(),
                    FieldInfo::new(
                        "A material with the appearance of a matte surface",
                        Optional,
                        Lambertian::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "glass".to_string(),
                    FieldInfo::new(
                        "A dielectric material which has a glass-like appearance",
                        Optional,
                        Glass::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "metal".to_string(),
                    FieldInfo::new(
                        "A reflective material that gives a metallic appearance",
                        Optional,
                        Metal::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "plastic".to_string(),
                    FieldInfo::new(
                        "A material with plastic-like appearance",
                        Optional,
                        Plastic::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "light".to_string(),
                    FieldInfo::new(
                        "A material that emits light",
                        Optional,
                        Light::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "blend".to_string(),
                    FieldInfo::new(
                        "A material that is a blend of two underlying materials",
                        Optional,
                        Blend::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
