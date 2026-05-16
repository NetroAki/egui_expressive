//! Timeline widgets.

mod automation;
mod clip_widget;
mod fade;
mod loop_region;
mod ruler_widget;

pub use automation::{AutomationCurve, AutomationPoint, AutomationSegment};
pub use clip_widget::{ClipKind, TimelineClip};
pub use fade::{FadeHandle, FadeSide};
pub use loop_region::LoopRegion;
pub use ruler_widget::Ruler;
