#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomationSegment {
    Linear,
    Smooth,
    Hold,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutomationPoint {
    pub beat: f32,
    pub value: f32,
    pub segment: AutomationSegment,
}

pub struct AutomationCurve {
    pub points: Vec<AutomationPoint>,
}

impl AutomationCurve {
    pub fn new(points: Vec<AutomationPoint>) -> Self {
        Self { points }
    }
    pub fn value_at(&self, beat: f32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        let mut prev = self.points[0];
        for p in &self.points[1..] {
            if beat <= p.beat {
                let t = ((beat - prev.beat) / (p.beat - prev.beat).max(1e-6)).clamp(0.0, 1.0);
                return match p.segment {
                    AutomationSegment::Linear => prev.value + (p.value - prev.value) * t,
                    AutomationSegment::Smooth => {
                        prev.value + (p.value - prev.value) * (t * t * (3.0 - 2.0 * t))
                    }
                    AutomationSegment::Hold => prev.value,
                };
            }
            prev = *p;
        }
        prev.value
    }

    pub fn paint(
        &self,
        painter: &egui::Painter,
        grid: &crate::widgets::GridCanvas,
        rect: egui::Rect,
        color: egui::Color32,
    ) {
        let _ = (painter, grid, rect, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_interpolates_linear_segments() {
        let curve = AutomationCurve {
            points: vec![
                AutomationPoint {
                    beat: 0.0,
                    value: 0.0,
                    segment: AutomationSegment::Linear,
                },
                AutomationPoint {
                    beat: 1.0,
                    value: 1.0,
                    segment: AutomationSegment::Linear,
                },
            ],
        };

        assert!((curve.value_at(0.5) - 0.5).abs() < 1e-6);
    }
}
