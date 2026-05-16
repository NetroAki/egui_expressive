//! Accessibility, focus, and motion primitives.
//!
//! Keep semantic metadata, focus affordances, modal focus behavior, and reduced
//! motion policy visible in API names instead of burying them in widget code.

pub mod focus;
pub mod live_region;
pub mod metadata;
pub mod motion;

pub use focus::{
    FocusRing, ModalFocusTrap, ModalTrapAction, RovingFocusDirection, RovingFocusGroup,
    RovingFocusItem,
};
pub use live_region::{LiveRegion, LiveRegionPoliteness, LiveRegionRelevant};
pub use metadata::{AccessibilityMeta, AccessibilityRole};
pub use motion::{reduced_motion, set_reduced_motion, MotionPolicy, MotionPreference};
