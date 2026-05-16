use super::*;

/// CSS-like transition config: duration + easing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub duration: f32,
    pub easing: Easing,
}

impl Transition {
    pub fn new(duration: f32, easing: Easing) -> Self {
        Self { duration, easing }
    }

    /// 120ms ease — matches the mockup's default hover transition.
    pub fn hover() -> Self {
        Self {
            duration: 0.12,
            easing: Easing::EaseInOut,
        }
    }

    /// 200ms ease-out — for panel open/close.
    pub fn panel() -> Self {
        Self {
            duration: 0.20,
            easing: Easing::EaseOut,
        }
    }

    /// 80ms ease-in — for press/active feedback.
    pub fn press() -> Self {
        Self {
            duration: 0.08,
            easing: Easing::EaseIn,
        }
    }
}

/// Animate a `f32` value toward `target` using the given transition.
/// Stores state in egui memory under `id`.
pub fn transition_f32(
    ctx: &egui::Context,
    id: egui::Id,
    target: f32,
    default: f32,
    t: Transition,
) -> f32 {
    let tween = Tween::new(id, t.duration, t.easing);
    tween.animate_f32(ctx, target, default)
}

/// Animate a `Color32` value toward `target` using the given transition.
pub fn transition_color(
    ctx: &egui::Context,
    id: egui::Id,
    target: egui::Color32,
    default: egui::Color32,
    t: Transition,
) -> egui::Color32 {
    let tween = Tween::new(id, t.duration, t.easing);
    tween.animate_color(ctx, target, default)
}
