use std::collections::HashMap;
use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::FieldType::Normal;
use crate::model::{
    Creator, CreatorContext, DocumentationStructure, FieldInfo, HelpDocumentation, ModelError,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CustomWidthHeight {
    pub width: usize,
    pub height: usize,
}

impl Creator<(usize, usize)> for CustomWidthHeight {
    fn create(&self, _: &CreatorContext) -> Result<(usize, usize), Box<dyn Error>> {
        if self.width < 1 || self.width > 8000 {
            return Err(From::from(ModelError::new(
                "Width must be between than 1 and 8000",
            )));
        }

        if self.height < 1 || self.height > 8000 {
            return Err(From::from(ModelError::new(
                "Height must be between than 1 and 8000",
            )));
        }

        Ok((self.width, self.height))
    }
}

impl HelpDocumentation for CustomWidthHeight {
    fn get_documentation_structure(_: u8) -> DocumentationStructure {
        DocumentationStructure {
            description: "Custom width and height".to_string(),
            fields: HashMap::from([
                (
                    "width".to_string(),
                    FieldInfo::new_simple("Width in pixels", Normal, "Width in pixels"),
                ),
                (
                    "height".to_string(),
                    FieldInfo::new_simple("Height in pixels", Normal, "Height in pixels"),
                ),
            ]),
        }
    }
}
