//! State-variant entry methods for `Tw`.

use crate::tailwind::{state::TwVariants, Tw};

impl Tw {
    pub fn hover(self, style: Self) -> TwVariants {
        TwVariants::new(self).hover(style)
    }
    pub fn pressed(self, style: Self) -> TwVariants {
        TwVariants::new(self).pressed(style)
    }
    pub fn focus(self, style: Self) -> TwVariants {
        TwVariants::new(self).focus(style)
    }
    pub fn selected(self, style: Self) -> TwVariants {
        TwVariants::new(self).selected(style)
    }
    pub fn disabled(self, style: Self) -> TwVariants {
        TwVariants::new(self).disabled(style)
    }
}
