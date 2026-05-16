//! Timeline clip metadata shared by future clip widgets.

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineClipSpec {
    pub start: f32,
    pub length: f32,
    pub label: String,
}

impl TimelineClipSpec {
    pub fn new(start: f32, length: f32, label: impl Into<String>) -> Self {
        Self {
            start,
            length: length.max(0.0),
            label: label.into(),
        }
    }

    pub fn end(&self) -> f32 {
        self.start + self.length
    }
}
