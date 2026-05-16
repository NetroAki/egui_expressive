//! Axis definitions and tick generation for editor rulers.

use std::ops::{Range, RangeInclusive};

#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    kind: AxisKind,
    range: RangeInclusive<f32>,
    major_step: f32,
    minor_step: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AxisKind {
    Continuous,
    Indexed,
    Time { unit: &'static str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisTick {
    pub value: f32,
    pub major: bool,
    pub label: Option<String>,
}

impl Axis {
    pub fn continuous(range: RangeInclusive<f32>, major_step: f32) -> Self {
        Self {
            kind: AxisKind::Continuous,
            range,
            major_step: major_step.max(f32::EPSILON),
            minor_step: None,
        }
    }

    pub fn indexed(range: Range<i32>) -> Self {
        Self {
            kind: AxisKind::Indexed,
            range: range.start as f32..=range.end as f32,
            major_step: 1.0,
            minor_step: None,
        }
    }

    pub fn time(range: RangeInclusive<f32>, major_step: f32) -> Self {
        Self {
            kind: AxisKind::Time { unit: "t" },
            range,
            major_step: major_step.max(f32::EPSILON),
            minor_step: None,
        }
    }

    pub fn unit(mut self, unit: &'static str) -> Self {
        if matches!(self.kind, AxisKind::Time { .. }) {
            self.kind = AxisKind::Time { unit };
        }
        self
    }

    pub fn minor_step(mut self, step: f32) -> Self {
        self.minor_step = (step > f32::EPSILON).then_some(step);
        self
    }

    pub fn range(&self) -> &RangeInclusive<f32> {
        &self.range
    }

    pub fn value_to_pixel(&self, value: f32, pixel_span: RangeInclusive<f32>) -> f32 {
        let start = *self.range.start();
        let end = *self.range.end();
        let t = if (end - start).abs() <= f32::EPSILON {
            0.0
        } else {
            ((value - start) / (end - start)).clamp(0.0, 1.0)
        };
        *pixel_span.start() + t * (*pixel_span.end() - *pixel_span.start())
    }

    pub fn pixel_to_value(&self, pixel: f32, pixel_span: RangeInclusive<f32>) -> f32 {
        let pixel_start = *pixel_span.start();
        let pixel_end = *pixel_span.end();
        let t = if (pixel_end - pixel_start).abs() <= f32::EPSILON {
            0.0
        } else {
            ((pixel - pixel_start) / (pixel_end - pixel_start)).clamp(0.0, 1.0)
        };
        *self.range.start() + t * (*self.range.end() - *self.range.start())
    }

    pub fn visible_ticks(&self, visible: RangeInclusive<f32>) -> Vec<AxisTick> {
        let start = (*visible.start()).max(*self.range.start());
        let end = (*visible.end()).min(*self.range.end());
        if end < start {
            return Vec::new();
        }

        let mut ticks = Vec::new();
        if let Some(minor_step) = self.minor_step {
            self.push_ticks(start, end, minor_step, false, &mut ticks);
        }
        self.push_ticks(start, end, self.major_step, true, &mut ticks);
        ticks.sort_by(|a, b| {
            a.value
                .total_cmp(&b.value)
                .then_with(|| b.major.cmp(&a.major))
        });
        ticks.dedup_by(|a, b| (a.value - b.value).abs() < 0.0001 && a.major == b.major);
        ticks
    }

    fn push_ticks(&self, start: f32, end: f32, step: f32, major: bool, out: &mut Vec<AxisTick>) {
        let mut value = (start / step).floor() * step;
        let mut guard = 0usize;
        while value <= end + 0.0001 && guard < 10_000 {
            if value >= start - 0.0001 {
                let is_major_position =
                    ((value / self.major_step).round() - value / self.major_step).abs() < 0.0001;
                if major || !is_major_position {
                    out.push(AxisTick {
                        value,
                        major,
                        label: major.then(|| self.label_for(value)),
                    });
                }
            }
            value += step;
            guard += 1;
        }
    }

    fn label_for(&self, value: f32) -> String {
        match self.kind {
            AxisKind::Indexed => format!("{}", value.round() as i32),
            AxisKind::Time { unit } => format!("{value:.0}{unit}"),
            AxisKind::Continuous => format!("{value:.2}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_maps_values_and_generates_ticks() {
        let axis = Axis::time(0.0..=4.0, 1.0).minor_step(0.5);
        assert_eq!(axis.value_to_pixel(2.0, 0.0..=100.0), 50.0);
        assert_eq!(axis.pixel_to_value(25.0, 0.0..=100.0), 1.0);
        let ticks = axis.visible_ticks(0.0..=2.0);
        assert!(ticks
            .iter()
            .any(|tick| tick.major && (tick.value - 1.0).abs() < 0.0001));
        assert!(ticks
            .iter()
            .any(|tick| !tick.major && (tick.value - 0.5).abs() < 0.0001));
    }
}
