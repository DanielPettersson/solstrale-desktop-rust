use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::model::num::visit_nums;
use crate::model::{Creator, CreatorContext, ModelError};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomWidthHeight {
    pub width: usize,
    pub height: usize,
}

visit_nums!(CustomWidthHeight);

impl Default for CustomWidthHeight {
    fn default() -> Self {
        CustomWidthHeight {
            width: 800,
            height: 600,
        }
    }
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
