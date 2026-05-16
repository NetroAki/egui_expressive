//! Value-lane range mapping for automation and parameter editors.

use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq)]
pub struct ValueLane {
    pub range: RangeInclusive<f32>,
    pub inverted: bool,
}

impl ValueLane {
    pub fn new(range: RangeInclusive<f32>) -> Self {
        Self {
            range,
            inverted: true,
        }
    }
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }
    pub fn normalize(&self, value: f32) -> f32 {
        let min = *self.range.start();
        let max = *self.range.end();
        if (max - min).abs() <= f32::EPSILON {
            0.0
        } else {
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        }
    }
    pub fn denormalize(&self, t: f32) -> f32 {
        let min = *self.range.start();
        let max = *self.range.end();
        min + t.clamp(0.0, 1.0) * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_lane_normalizes_and_denormalizes_with_clamping() {
        let lane = ValueLane::new(0.0..=100.0).inverted(false);

        assert!(!lane.inverted);
        assert_eq!(lane.normalize(50.0), 0.5);
        assert_eq!(lane.normalize(-10.0), 0.0);
        assert_eq!(lane.normalize(150.0), 1.0);
        assert_eq!(lane.denormalize(0.25), 25.0);
        assert_eq!(lane.denormalize(2.0), 100.0);
    }

    #[test]
    fn value_lane_zero_range_normalizes_to_zero() {
        let lane = ValueLane::new(4.0..=4.0);

        assert_eq!(lane.normalize(4.0), 0.0);
        assert_eq!(lane.denormalize(0.5), 4.0);
    }
}
