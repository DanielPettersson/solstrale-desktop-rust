use std::error::Error;

use crate::model::custom_width_height::CustomWidthHeight;
use crate::model::half_screen_width_height::HalfScreenWidthHeight;
use crate::model::one_of::one_of;
use crate::model::quarter_screen_width_height::QuarterScreenWidthHeight;
use crate::model::screen_width_height::ScreenWidthHeight;
use crate::model::{Creator, CreatorContext};

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
