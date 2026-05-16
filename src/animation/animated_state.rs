use super::*;

/// A value that automatically animates toward a target using spring physics.
///
/// Store in egui memory via `StateSlot` or as part of your widget state.
/// Call `set()` to change the target, `get()` every frame to read the current value.
pub struct AnimatedState<T: crate::style::Lerp + Clone + Copy + PartialEq + 'static + Send + Sync> {
    id: egui::Id,
    pub target: T,
    stiffness: f32,
    damping: f32,
}

impl<T: crate::style::Lerp + Clone + Copy + PartialEq + 'static + Send + Sync> AnimatedState<T> {
    /// Create with spring physics. `initial` is both the starting and target value.
    pub fn spring(id: egui::Id, initial: T) -> Self {
        Self {
            id,
            target: initial,
            stiffness: 200.0,
            damping: 20.0,
        }
    }

    /// Adjust spring parameters.
    pub fn with_spring(mut self, stiffness: f32, damping: f32) -> Self {
        self.stiffness = stiffness;
        self.damping = damping;
        self
    }

    /// Set the target. Animation starts on next `get()` call.
    pub fn set(&mut self, target: T) {
        self.target = target;
    }

    /// Get the current animated value. Must be called every frame.
    /// Internally drives a spring from the stored `from` value toward `target`.
    pub fn get(&self, ctx: &egui::Context) -> T {
        let from_id = self.id.with("__as_from");
        let last_target_id = self.id.with("__as_last_target");
        let spring_id = self.id.with("__as_spring");

        // Check if target changed since last get() by comparing stored last_target
        let last_target: Option<T> = ctx.data(|d| d.get_temp(last_target_id));
        let target_changed = last_target != Some(self.target);

        if target_changed {
            // Capture current visible value WITHOUT advancing the spring further.
            // Read the stored spring position directly to avoid a one-frame overshoot toward the old target.
            let from: T = ctx.data(|d| d.get_temp(from_id)).unwrap_or(self.target);
            let spring = Spring::new(spring_id, self.stiffness, self.damping);
            // Read current spring t from stored state (don't call animate which would advance it)
            let stored_t: f32 = ctx.data(|d| {
                d.get_temp::<crate::animation::SpringMem>(spring_id.with("__spring_mem"))
                    .map(|m| m.position)
                    .unwrap_or(0.0)
            });
            let prev_target = last_target.unwrap_or(self.target);
            let current = T::lerp(&from, &prev_target, stored_t.clamp(0.0, 1.0));
            // Store new from = current visible value, update last_target
            ctx.data_mut(|d| {
                d.insert_temp(from_id, current);
                d.insert_temp(last_target_id, self.target);
            });
            // Reset spring to 0
            spring.reset_to(ctx, 0.0);
            ctx.request_repaint();
            return current;
        }

        // Normal animation
        let from: T = ctx.data(|d| d.get_temp(from_id)).unwrap_or(self.target);
        let spring = Spring::new(spring_id, self.stiffness, self.damping);
        let t = spring.animate(ctx, 1.0, 0.0);

        // When animation settles, update from to current target
        if t >= 1.0 {
            ctx.data_mut(|d| d.insert_temp(from_id, self.target));
        }

        T::lerp(&from, &self.target, t.clamp(0.0, 1.0))
    }

    /// Snap to target immediately, no animation.
    pub fn snap(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.insert_temp(self.id.with("__as_from"), self.target);
            d.insert_temp(self.id.with("__as_last_target"), self.target);
        });
        let spring = Spring::new(self.id.with("__as_spring"), self.stiffness, self.damping);
        spring.reset_to(ctx, 1.0);
    }
}

/// Animated f32 value.
pub type AnimatedF32 = AnimatedState<f32>;
/// Animated Color32 value.
pub type AnimatedColor = AnimatedState<egui::Color32>;
/// Animated Vec2 value.
pub type AnimatedVec2 = AnimatedState<egui::Vec2>;
