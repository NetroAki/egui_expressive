//! Reduced-motion policy for animations and transitions.

const REDUCED_MOTION_ID: &str = "egui_expressive.reduced_motion";

/// User or app motion preference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionPreference {
    NoPreference,
    Reduce,
}

/// Animation timing policy resolved from the motion preference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionPolicy {
    pub preference: MotionPreference,
    pub duration_scale: f32,
}

impl MotionPolicy {
    pub fn new(preference: MotionPreference) -> Self {
        let duration_scale = match preference {
            MotionPreference::NoPreference => 1.0,
            MotionPreference::Reduce => 0.0,
        };
        Self {
            preference,
            duration_scale,
        }
    }

    pub fn from_ctx(ctx: &egui::Context) -> Self {
        if reduced_motion(ctx) {
            Self::new(MotionPreference::Reduce)
        } else {
            Self::new(MotionPreference::NoPreference)
        }
    }

    pub fn duration(self, seconds: f32) -> f32 {
        seconds * self.duration_scale
    }

    pub fn should_animate(self) -> bool {
        self.duration_scale > 0.0
    }
}

pub fn set_reduced_motion(ctx: &egui::Context, reduce: bool) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new(REDUCED_MOTION_ID), reduce));
}

pub fn reduced_motion(ctx: &egui::Context) -> bool {
    ctx.data(|data| {
        data.get_temp::<bool>(egui::Id::new(REDUCED_MOTION_ID))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_motion_zeroes_duration() {
        let policy = MotionPolicy::new(MotionPreference::Reduce);
        assert_eq!(policy.duration(0.2), 0.0);
        assert!(!policy.should_animate());
    }
}
