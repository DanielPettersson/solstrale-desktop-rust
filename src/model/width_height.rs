use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::custom_width_height::CustomWidthHeight;
use crate::model::half_screen_width_height::HalfScreenWidthHeight;
use crate::model::quarter_screen_width_height::QuarterScreenWidthHeight;
use crate::model::screen_width_height::ScreenWidthHeight;
use crate::model::FieldType::{Normal, OptionalList};
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct WidthHeight {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen: Option<ScreenWidthHeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub half_screen: Option<HalfScreenWidthHeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarter_screen: Option<QuarterScreenWidthHeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomWidthHeight>,
}

impl Default for WidthHeight {
    fn default() -> Self {
        WidthHeight {
            screen: Some(ScreenWidthHeight {}),
            half_screen: None,
            quarter_screen: None,
            custom: None,
        }
    }
}

impl Creator<(usize, usize)> for WidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        match self {
            WidthHeight {
                screen: Some(s),
                half_screen: None,
                quarter_screen: None,
                custom: None,
            } => s.create(ctx),
            WidthHeight {
                screen: None,
                half_screen: Some(s),
                quarter_screen: None,
                custom: None,
            } => s.create(ctx),
            WidthHeight {
                screen: None,
                half_screen: None,
                quarter_screen: Some(s),
                custom: None,
            } => s.create(ctx),
            WidthHeight {
                screen: None,
                half_screen: None,
                quarter_screen: None,
                custom: Some(s),
            } => s.create(ctx),
            _ => Err(From::from(ModelError::new(
                "WidthHeight should have single field defined",
            ))),
        }
    }
}

impl HelpDocumentation for WidthHeight {
    fn get_documentation_structure(depth: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Defines the with and height in pixels of the rendered image".to_string(),
            fields: HashMap::from([
                (
                    "screen".to_string(),
                    FieldInfo::new(
                        "Same width and height as the visible window",
                        Normal,
                        ScreenWidthHeight::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "half_screen".to_string(),
                    FieldInfo::new(
                        "Half of the width and height as the visible window",
                        Normal,
                        HalfScreenWidthHeight::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "quarter_screen".to_string(),
                    FieldInfo::new(
                        "Quarter of the width and height as the visible window",
                        OptionalList,
                        QuarterScreenWidthHeight::get_documentation_structure(depth + 1),
                    ),
                ),
                (
                    "custom".to_string(),
                    FieldInfo::new(
                        "Custom defined width and height",
                        Normal,
                        CustomWidthHeight::get_documentation_structure(depth + 1),
                    ),
                ),
            ]),
        }
    }
}
