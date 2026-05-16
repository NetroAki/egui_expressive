/// Ballistics config for smoothing meter values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterBallistics {
    pub attack_seconds: f32,
    pub release_seconds: f32,
    pub peak_hold_seconds: f32,
}

impl Default for MeterBallistics {
    fn default() -> Self {
        Self {
            attack_seconds: 0.01,
            release_seconds: 0.25,
            peak_hold_seconds: 0.8,
        }
    }
}

impl MeterBallistics {
    pub fn smooth(self, current: f32, target: f32, dt: f32) -> f32 {
        let tau = if target > current {
            self.attack_seconds
        } else {
            self.release_seconds
        }
        .max(1e-4);
        let alpha = 1.0 - (-dt.max(0.0) / tau).exp();
        current + (target - current) * alpha.clamp(0.0, 1.0)
    }
}
