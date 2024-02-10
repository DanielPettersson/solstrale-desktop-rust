use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::{Creator, CreatorContext, DocumentationStructure, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct HalfScreenWidthHeight {}

impl Creator<(usize, usize)> for HalfScreenWidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        Ok((ctx.screen_width / 2, ctx.screen_height / 2))
    }
}

impl HelpDocumentation for HalfScreenWidthHeight {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "The width and height is half of the visible render window in each dimension",
        )
    }
}
