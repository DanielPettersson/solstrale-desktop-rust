use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::{Creator, CreatorContext, DocumentationStructure, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScreenWidthHeight {}

impl Creator<(usize, usize)> for ScreenWidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        Ok((ctx.screen_width, ctx.screen_height))
    }
}

impl HelpDocumentation for ScreenWidthHeight {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "The width and height is the same as the visible render window",
        )
    }
}
