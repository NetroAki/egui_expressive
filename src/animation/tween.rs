use super::*;

/// Tween animation handle for smooth value transitions.
///
/// A `Tween` animates a value from its current state toward a target using
/// a specified easing function over a given duration.
pub struct Tween {
    /// Unique identifier for this tween.
    id: Id,
    /// Animation duration in seconds.
    duration: f32,
    /// Easing function to apply.
    easing: Easing,
}

impl Tween {
    /// Create a new tween animation.
    ///
    /// # Arguments
    /// * `id` - Unique identifier (use `ui.id().with("name")`)
    /// * `duration` - Animation duration in seconds
    /// * `easing` - Easing curve to apply
    pub fn new(id: Id, duration: f32, easing: Easing) -> Self {
        Self {
            id,
            duration,
            easing,
        }
    }

    /// Animate a 32-bit float value toward a target.
    ///
    /// Returns the current animated value. When the target changes, the animation
    /// resets and animates from the current position to the new target.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    /// * `target` - Target value to animate toward
    /// * `default` - Default value when no animation is active (used on first frame)
    pub fn animate_f32(&self, ctx: &Context, target: f32, default: f32) -> f32 {
        let mem_id = self.id.with("__tween_mem");

        // Load or initialize memory state
        let mut mem: TweenMem = ctx.memory(|m| m.data.get_temp(mem_id)).unwrap_or(TweenMem {
            from: default,
            start_dt_acc: 0.0,
            last_target: target,
        });

        if self.duration <= 1e-6 {
            // Zero duration: snap to target immediately
            ctx.memory_mut(|m| {
                m.data.insert_temp(
                    mem_id,
                    TweenMem {
                        from: target,
                        start_dt_acc: 0.0,
                        last_target: target,
                    },
                )
            });
            return target;
        }

        let dt = ctx.input(|i| i.unstable_dt);

        // Detect target change and reset animation
        if (mem.last_target - target).abs() > 1e-6 {
            mem.from = mem.from
                + (mem.last_target - mem.from)
                    * self
                        .easing
                        .apply((mem.start_dt_acc / self.duration).clamp(0.0, 1.0));
            mem.start_dt_acc = 0.0;
            mem.last_target = target;
        }

        // Advance animation
        mem.start_dt_acc += dt;

        let raw_t = (mem.start_dt_acc / self.duration).clamp(0.0, 1.0);
        let eased_t = self.easing.apply(raw_t);

        let result = mem.from + (target - mem.from) * eased_t;

        // Save memory state
        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));

        // Request repaint while animating
        if raw_t < 1.0 {
            ctx.request_repaint();
        }

        result
    }

    /// Animate a color value toward a target.
    ///
    /// Interpolates each RGBA channel separately and recomposes the result.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    /// * `target` - Target color to animate toward
    /// * `default` - Default color when no animation is active
    pub fn animate_color(&self, ctx: &Context, target: Color32, default: Color32) -> Color32 {
        // Decompose to f32 channels
        let from = color_to_f32(default);
        let to = color_to_f32(target);

        // Animate each channel
        let r = Tween {
            id: self.id.with("__tc_r"),
            ..*self
        }
        .animate_f32(ctx, to[0], from[0]);
        let g = Tween {
            id: self.id.with("__tc_g"),
            ..*self
        }
        .animate_f32(ctx, to[1], from[1]);
        let b = Tween {
            id: self.id.with("__tc_b"),
            ..*self
        }
        .animate_f32(ctx, to[2], from[2]);
        let a = Tween {
            id: self.id.with("__tc_a"),
            ..*self
        }
        .animate_f32(ctx, to[3], from[3]);

        // Recompose
        Color32::from_rgba_unmultiplied(
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
            a.clamp(0.0, 255.0) as u8,
        )
    }
}

/// Convert Color32 to array of f32 channels (returns 0-255 range as f32).
#[inline]
pub(crate) fn color_to_f32(c: Color32) -> [f32; 4] {
    let [r, g, b, a] = c.to_array();
    [r as f32, g as f32, b as f32, a as f32]
}
