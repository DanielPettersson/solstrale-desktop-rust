use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::{Creator, CreatorContext, DocumentationStructure, HelpDocumentation};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct QuarterScreenWidthHeight {}

impl Creator<(usize, usize)> for QuarterScreenWidthHeight {
    fn create(&self, ctx: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        Ok((ctx.screen_width / 4, ctx.screen_height / 4))
    }
}

impl HelpDocumentation for QuarterScreenWidthHeight {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure::new_simple(
            "The width and height is quarter of the visible render window in each dimension",
        )
    }
}
