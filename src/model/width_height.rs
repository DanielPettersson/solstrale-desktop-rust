use std::collections::HashMap;
use std::error::Error;

use crate::model::FieldType::{Normal, OptionalList};
use crate::model::custom_width_height::CustomWidthHeight;
use crate::model::half_screen_width_height::HalfScreenWidthHeight;
use crate::model::one_of::one_of;
use crate::model::quarter_screen_width_height::QuarterScreenWidthHeight;
use crate::model::screen_width_height::ScreenWidthHeight;
use crate::model::{Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation};

one_of! {
    #[derive(PartialEq, Debug, Clone)]
    pub enum WidthHeight {
        screen => Screen(ScreenWidthHeight),
        half_screen => HalfScreen(HalfScreenWidthHeight),
        quarter_screen => QuarterScreen(QuarterScreenWidthHeight),
        custom => Custom(CustomWidthHeight),
    }
    repr: WidthHeightRepr, WidthHeightReprRef;
    empty: None;
}

impl Default for WidthHeight {
    fn default() -> Self {
        WidthHeight::Screen(ScreenWidthHeight {})
    }
}

impl Creator<(usize, usize)> for WidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        match self {
            WidthHeight::Screen(s) => s.create(ctx),
            WidthHeight::HalfScreen(s) => s.create(ctx),
            WidthHeight::QuarterScreen(s) => s.create(ctx),
            WidthHeight::Custom(s) => s.create(ctx),
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
