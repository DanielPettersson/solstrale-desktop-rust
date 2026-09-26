use crate::model::FieldType::Optional;
use crate::model::num::Num;
use crate::model::one_of::one_of;
use crate::model::pos::Pos;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};
use solstrale::geo::transformation::{
    RotationX, RotationY, RotationZ, Scale, Transformations, Transformer, Translation,
};
use std::collections::HashMap;
use std::error::Error;

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum Transformation {
        translation => Translation(Pos),
        scale => Scale(Num),
        rotation_x => RotationX(Num),
        rotation_y => RotationY(Num),
        rotation_z => RotationZ(Num),
    }
    repr: TransformationRepr, TransformationReprRef;
    empty: None;
}

impl Creator<Box<dyn Transformer>> for Transformation {
    fn create(&self, ctx: &CreatorContext) -> Result<Box<dyn Transformer>, Box<dyn Error>> {
        Ok(match self {
            Transformation::Translation(p) => Box::new(Translation::new(p.create(ctx)?)),
            Transformation::Scale(s) => Box::new(Scale::new(s.eval(ctx)?)),
            Transformation::RotationX(r) => Box::new(RotationX::new(r.eval(ctx)?)),
            Transformation::RotationY(r) => Box::new(RotationY::new(r.eval(ctx)?)),
            Transformation::RotationZ(r) => Box::new(RotationZ::new(r.eval(ctx)?)),
        })
    }
}

pub fn create_transformation(
    transformations: &Vec<Transformation>,
    ctx: &CreatorContext,
) -> Result<Transformations, Box<dyn Error>> {
    let mut trans: Vec<Box<dyn Transformer>> = Vec::with_capacity(transformations.len());
    for t in transformations {
        trans.push(t.create(ctx)?);
    }
    Ok(Transformations::new(trans))
}

impl HelpDocumentation for Transformation {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Changes a hittables position, rotation and / or size".to_string(),
            fields: HashMap::from([
                (
                    "translation".to_string(),
                    FieldInfo::new(
                        "Moves the hittable by the given offset",
                        Optional,
                        Pos::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "scale".to_string(),
                    FieldInfo::new_simple(
                        "Scales the hittable uniformly by the given factor",
                        Optional,
                        "Scaling factor",
                    ),
                ),
                (
                    "rotation_x".to_string(),
                    FieldInfo::new_simple(
                        "Rotates the hittable around the X axis",
                        Optional,
                        "Rotation in degrees",
                    ),
                ),
                (
                    "rotation_y".to_string(),
                    FieldInfo::new_simple(
                        "Rotates the hittable around the Y axis",
                        Optional,
                        "Rotation in degrees",
                    ),
                ),
                (
                    "rotation_z".to_string(),
                    FieldInfo::new_simple(
                        "Rotates the hittable around the Z axis",
                        Optional,
                        "Rotation in degrees",
                    ),
                ),
            ]),
        }
    }
}
