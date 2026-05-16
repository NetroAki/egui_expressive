//! Timeline ruler configuration shared by future ruler widgets.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineRulerSpec {
    pub beats_per_bar: u32,
    pub height: f32,
    pub pixels_per_beat: f32,
}

impl TimelineRulerSpec {
    pub fn new(beats_per_bar: u32) -> Self {
        Self {
            beats_per_bar: beats_per_bar.max(1),
            height: 24.0,
            pixels_per_beat: 80.0,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn pixels_per_beat(mut self, value: f32) -> Self {
        self.pixels_per_beat = value.max(1.0);
        self
    }
}
