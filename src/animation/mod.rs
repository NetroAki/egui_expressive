#![allow(dead_code)]

//! Animation helpers: easing curves, tweens, spring physics, and animation sequences.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use egui_expressive::animation::{Easing, Tween, Spring};
//!
//! // Tween a value
//! let tween = Tween::new(ui.id(), 0.3, Easing::EaseOut);
//! let value = tween.animate_f32(ctx, target, default);
//!
//! // Spring to a target
//! let spring = Spring::new(ui.id(), 200.0, 20.0);
//! let value = spring.animate(ctx, target, default);
//! ```

use egui::{Color32, Context, Id};

/// Easing curve type for animations.
///
/// Each variant represents a different timing curve that controls how the animation
/// progresses from start to finish. The `apply` method maps a linear input `t` (0..1)
/// to an eased output value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Linear progression, no easing applied.
    Linear,

    /// Ease-in: starts slow, accelerates (cubic: t³).
    EaseIn,

    /// Ease-out: starts fast, decelerates (cubic: 1 - (1-t)³).
    EaseOut,

    /// Ease-in-out: slow start and end, fast middle (cubic).
    EaseInOut,

    /// Ease-in with overshoot on entry (c1=1.70158).
    EaseInBack,

    /// Ease-out with overshoot on exit (c1=1.70158).
    EaseOutBack,

    /// Combined ease-in-out with overshoot (c1=1.70158, c2=2.594).
    EaseInOutBack,

    /// Bounce effect on exit (multi-segment).
    EaseOutBounce,

    /// Bounce effect on entry (inverted bounce).
    EaseInBounce,

    /// Custom cubic bezier curve with control points (p1x, p1y, p2x, p2y).
    /// Matches CSS `cubic-bezier(p1x, p1y, p2x, p2y)`.
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// Apply the easing function to a normalized time value.
    ///
    /// # Arguments
    /// * `t` - Normalized time in range [0.0, 1.0]
    ///
    /// # Returns
    /// Eased value in range [0.0, 1.0]
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Easing::Linear => t,

            Easing::EaseIn => cubic_ease_in(t),
            Easing::EaseOut => cubic_ease_out(t),
            Easing::EaseInOut => cubic_ease_in_out(t),

            Easing::EaseInBack => ease_in_back(t),
            Easing::EaseOutBack => ease_out_back(t),
            Easing::EaseInOutBack => ease_in_out_back(t),

            Easing::EaseOutBounce => ease_out_bounce(t),
            Easing::EaseInBounce => ease_in_bounce(t),

            Easing::CubicBezier(p1x, p1y, p2x, p2y) => cubic_bezier(t, p1x, p1y, p2x, p2y),
        }
    }
}

// ---------------------------------------------------------------------------
// Easing function implementations
// ---------------------------------------------------------------------------

#[inline]
fn cubic_ease_in(t: f32) -> f32 {
    t * t * t
}

#[inline]
fn cubic_ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

#[inline]
fn cubic_ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// ---------------------------------------------------------------------------
// Back easing (overshoot)
// ---------------------------------------------------------------------------

// c1 = 1.70158
const BACK_C1: f32 = 1.70158;
// c3 = c1 + 1
const BACK_C3: f32 = BACK_C1 + 1.0;
// c2 = c1 * 1.525
const BACK_C2: f32 = BACK_C1 * 1.525;

#[inline]
fn ease_in_back(t: f32) -> f32 {
    BACK_C3 * t * t * t - BACK_C1 * t * t
}

#[inline]
fn ease_out_back(t: f32) -> f32 {
    1.0 + BACK_C3 * (t - 1.0).powi(3) + BACK_C1 * (t - 1.0).powi(2)
}

#[inline]
fn ease_in_out_back(t: f32) -> f32 {
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((BACK_C2 + 1.0) * 2.0 * t - BACK_C2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((BACK_C2 + 1.0) * (2.0 * t - 2.0) + BACK_C2) + 2.0) / 2.0
    }
}

// ---------------------------------------------------------------------------
// Bounce easing
// ---------------------------------------------------------------------------

// Multi-segment bounce using n1=7.5625, d1=2.75
const BOUNCE_N1: f32 = 7.5625;
const BOUNCE_D1_1: f32 = 2.75;
const BOUNCE_D1_2: f32 = BOUNCE_D1_1 * 2.0;
const BOUNCE_D1_3: f32 = BOUNCE_D1_1 * 2.5;

fn ease_out_bounce(t: f32) -> f32 {
    if t < 1.0 / BOUNCE_D1_1 {
        BOUNCE_N1 * t * t
    } else if t < 2.0 / BOUNCE_D1_1 {
        let t = t - 1.5 / BOUNCE_D1_1;
        BOUNCE_N1 * t * t + 0.75
    } else if t < 2.5 / BOUNCE_D1_1 {
        let t = t - 2.25 / BOUNCE_D1_1;
        BOUNCE_N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / BOUNCE_D1_1;
        BOUNCE_N1 * t * t + 0.984375
    }
}

fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

// ---------------------------------------------------------------------------
// Cubic Bezier
// ---------------------------------------------------------------------------

/// Evaluate cubic bezier curve at parameter t using De Casteljau's algorithm.
///
/// Control points: P0=(0,0), P1=(p1x,p1y), P2=(p2x,p2y), P3=(1,1)
fn cubic_bezier(t: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32) -> f32 {
    // Find t_bezier from t_input using Newton-Raphson (8 iterations)
    let t_bezier = solve_t_for_x(t, p1x, p2x, 8);

    // Evaluate y at t_bezier
    eval_bezier_y(t_bezier, p1y, p2y)
}

/// Find t such that bezier_x(t) ≈ x_input using Newton-Raphson.
fn solve_t_for_x(x_input: f32, p1x: f32, p2x: f32, iterations: usize) -> f32 {
    let mut t = x_input;

    for _ in 0..iterations {
        let x_current = bezier_x(t, p1x, p2x);
        let dx = bezier_x_derivative(t, p1x, p2x);

        if dx.abs() < 1e-9 {
            break;
        }

        t -= (x_current - x_input) / dx;
        t = t.clamp(0.0, 1.0);
    }

    t
}

/// Evaluate X component of cubic bezier at t.
#[inline]
fn bezier_x(t: f32, p1x: f32, p2x: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // Bx(t) = (1-t)³·0 + 3(1-t)²t·p1x + 3(1-t)t²·p2x + t³·1
    3.0 * mt2 * t * p1x + 3.0 * mt * t2 * p2x + t3
}

/// Derivative of X component: B'x(t) = 3(1-t)²p1x + 6(1-t)tp2x + 3t²
#[inline]
fn bezier_x_derivative(t: f32, p1x: f32, p2x: f32) -> f32 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    3.0 * mt2 * p1x + 6.0 * mt * t * p2x + 3.0 * t2
}

/// Evaluate Y component of cubic bezier at t.
#[inline]
fn eval_bezier_y(t: f32, p1y: f32, p2y: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // By(t) = (1-t)³·0 + 3(1-t)²t·p1y + 3(1-t)t²·p2y + t³·1
    3.0 * mt2 * t * p1y + 3.0 * mt * t2 * p2y + t3
}

// ---------------------------------------------------------------------------
// Internal memory structures
// ---------------------------------------------------------------------------

/// State for a tween animation, stored in egui memory.
#[derive(Clone, Debug)]
struct TweenMem {
    /// Starting value for the current animation segment.
    from: f32,
    /// Accumulated time (in seconds) since animation started.
    start_dt_acc: f32,
    /// The last target value we were animating toward.
    last_target: f32,
}

/// State for a spring simulation, stored in egui memory.
#[derive(Clone, Debug)]
struct SpringMem {
    /// Current position of the spring.
    position: f32,
    /// Current velocity of the spring.
    velocity: f32,
    /// The last target value (to detect target changes).
    last_target: f32,
}

/// State for an animation sequence, stored in egui memory.
#[derive(Clone, Debug)]
struct SeqMem {
    /// Index of the currently playing step.
    step_idx: usize,
    /// Progress within the current step (0..1).
    step_progress: f32,
    /// Current output value.
    current: f32,
    /// Whether the sequence is actively playing.
    playing: bool,
}

// ---------------------------------------------------------------------------
// Tween
// ---------------------------------------------------------------------------

/// Tween animation handle for smooth value transitions.
///
/// A `Tween` animates a value from its current state toward a target using
/// a specified easing function over a given duration.
///
/// # Example
///
/// ```rust,no_run
/// use egui_expressive::animation::{Easing, Tween};
///
/// fn my_widget(ui: &mut egui::Ui, target: f32, current: f32) -> f32 {
///     let tween = Tween::new(ui.id().with("my_tween"), 0.3, Easing::EaseOut);
///     tween.animate_f32(ui.ctx(), target, current)
/// }
/// ```
#[derive(Debug, Clone)]
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
        let r = self.animate_f32(ctx, to[0], from[0]);
        let g = self.animate_f32(ctx, to[1], from[1]);
        let b = self.animate_f32(ctx, to[2], from[2]);
        let a = self.animate_f32(ctx, to[3], from[3]);

        // Recompose
        Color32::from_rgba_unmultiplied(
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
            a.clamp(0.0, 255.0) as u8,
        )
    }
}

/// Convert Color32 to array of f32 channels (0-255 → 0-1).
#[inline]
fn color_to_f32(c: Color32) -> [f32; 4] {
    let [r, g, b, a] = c.to_array();
    [r as f32, g as f32, b as f32, a as f32]
}

// ---------------------------------------------------------------------------
// Spring
// ---------------------------------------------------------------------------

/// Spring physics animation for natural-feeling motion.
///
/// A `Spring` simulates a mass-spring-damper system to produce smooth,
/// physically-based animations that overshoot and settle at the target.
///
/// # Parameters
/// * `stiffness` - Spring constant (higher = faster response)
/// * `damping` - Energy loss per frame (higher = less oscillation)
/// * `mass` - Inertia (typically 1.0)
///
/// # Example
///
/// ```rust,no_run
/// use egui_expressive::animation::Spring;
///
/// fn my_widget(ui: &mut egui::Ui, target: f32, current: f32) -> f32 {
///     let spring = Spring::new(ui.id().with("my_spring"), 200.0, 20.0);
///     spring.animate(ui.ctx(), target, current)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Spring {
    /// Unique identifier for this spring.
    id: Id,
    /// Spring stiffness constant (e.g., 200.0).
    pub stiffness: f32,
    /// Damping coefficient (e.g., 20.0).
    pub damping: f32,
    /// Mass of the spring (typically 1.0).
    pub mass: f32,
}

impl Spring {
    /// Create a new spring animation.
    ///
    /// # Arguments
    /// * `id` - Unique identifier
    /// * `stiffness` - Spring constant (e.g., 200.0)
    /// * `damping` - Damping coefficient (e.g., 20.0)
    pub fn new(id: Id, stiffness: f32, damping: f32) -> Self {
        Self {
            id,
            stiffness,
            damping,
            mass: 1.0,
        }
    }

    /// Animate a value toward the target using spring physics.
    ///
    /// Returns the current spring position. The animation automatically
    /// settles when the position and velocity are close to the target.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    /// * `target` - Target value
    /// * `default` - Initial value when no spring state exists
    pub fn animate(&self, ctx: &Context, target: f32, default: f32) -> f32 {
        let mem_id = self.id.with("__spring_mem");

        // Load or initialize memory state
        let mut mem: SpringMem = ctx
            .memory(|m| m.data.get_temp(mem_id))
            .unwrap_or(SpringMem {
                position: default,
                velocity: 0.0,
                last_target: target,
            });

        // Handle target change
        if (mem.last_target - target).abs() > 1e-6 {
            mem.last_target = target;
        }

        let dt = ctx.input(|i| i.unstable_dt).min(0.05); // Cap at 50ms for stability

        // Spring physics integration (semi-implicit Euler)
        let displacement = target - mem.position;
        let spring_force = self.stiffness * displacement;
        let damping_force = -self.damping * mem.velocity;
        let total_force = spring_force + damping_force;

        let acceleration = total_force / self.mass;
        mem.velocity += acceleration * dt;
        mem.position += mem.velocity * dt;

        // Check if settled
        let pos_diff = (mem.position - target).abs();
        let vel_mag = mem.velocity.abs();
        let settled = pos_diff < 0.001 && vel_mag < 0.001;

        if settled {
            mem.position = target;
            mem.velocity = 0.0;
        }

        // Extract result before saving state (mem will be moved)
        let result = mem.position;

        // Save memory state
        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));

        // Request repaint while not settled
        if !settled {
            ctx.request_repaint();
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Animation Sequence
// ---------------------------------------------------------------------------

/// A single step in an animation sequence.
#[derive(Clone, Debug)]
pub struct AnimStep {
    /// Target value for this step.
    pub target: f32,
    /// Duration of this step in seconds.
    pub duration: f32,
    /// Easing function for this step.
    pub easing: Easing,
}

/// A sequence of animation steps played in order.
///
/// # Example
///
/// ```rust,ignore
/// use egui_expressive::animation::{AnimSequence, AnimStep, Easing};
///
/// let steps = vec![
///     AnimStep { target: 1.0, duration: 0.3, easing: Easing::EaseOut },
///     AnimStep { target: 0.0, duration: 0.2, easing: Easing::EaseIn },
/// ];
/// let seq = AnimSequence::new(ui.id().with("my_seq"), steps);
/// let value = seq.animate(ui.ctx());
/// ```
#[derive(Debug, Clone)]
pub struct AnimSequence {
    /// Unique identifier for this sequence.
    id: Id,
    /// Steps to play in order.
    steps: Vec<AnimStep>,
}

impl AnimSequence {
    /// Create a new animation sequence.
    ///
    /// # Arguments
    /// * `id` - Unique identifier
    /// * `steps` - Ordered list of animation steps
    pub fn new(id: Id, steps: Vec<AnimStep>) -> Self {
        Self { id, steps }
    }

    /// Get the current animated value.
    ///
    /// Returns the output of the current step's easing, advancing through
    /// steps automatically as they complete.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    pub fn animate(&self, ctx: &Context) -> f32 {
        let mem_id = self.id.with("__seq_mem");

        // Load or initialize memory state
        let mut mem: SeqMem = ctx.memory(|m| m.data.get_temp(mem_id)).unwrap_or(SeqMem {
            step_idx: 0,
            step_progress: 0.0,
            current: 0.0,
            playing: true,
        });

        // Nothing to animate if no steps
        if self.steps.is_empty() {
            return mem.current;
        }

        // Auto-start if not playing
        if !mem.playing {
            return mem.current;
        }

        let dt = ctx.input(|i| i.unstable_dt);
        let current_step = &self.steps[mem.step_idx];

        // Advance progress within current step
        let step_dt = if current_step.duration > 0.0 {
            dt / current_step.duration
        } else {
            1.0
        };

        mem.step_progress += step_dt;

        // Handle step completion
        if mem.step_progress >= 1.0 {
            mem.step_progress = 0.0;

            // Snap to current step's target
            if mem.step_idx < self.steps.len() {
                mem.current = self.steps[mem.step_idx].target;
            }

            // Advance to next step
            if mem.step_idx + 1 < self.steps.len() {
                mem.step_idx += 1;
            } else {
                // Sequence complete, stop playing
                mem.playing = false;
            }
        }

        // Compute output for current step
        if mem.step_idx < self.steps.len() {
            let step = &self.steps[mem.step_idx];

            // Determine start value (previous step's target or initial current)
            let start_value = if mem.step_idx == 0 {
                mem.current
            } else {
                self.steps[mem.step_idx - 1].target
            };

            let eased_t = step.easing.apply(mem.step_progress);
            mem.current = start_value + (step.target - start_value) * eased_t;
        }

        // Extract result before saving state (mem will be moved)
        let result = mem.current;
        let playing = mem.playing;

        // Save memory state
        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));

        // Request repaint while playing
        if playing {
            ctx.request_repaint();
        }

        result
    }

    /// Reset the sequence to the beginning without playing.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    pub fn reset(&self, ctx: &Context) {
        let mem_id = self.id.with("__seq_mem");

        let mem = SeqMem {
            step_idx: 0,
            step_progress: 0.0,
            current: 0.0,
            playing: false,
        };

        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));
    }

    /// Start or restart the sequence from the beginning.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    pub fn play(&self, ctx: &Context) {
        let mem_id = self.id.with("__seq_mem");

        // Load current state to preserve current value
        let current: f32 = ctx
            .memory(|m| m.data.get_temp::<SeqMem>(mem_id))
            .map(|m| m.current)
            .unwrap_or(0.0);

        let mem = SeqMem {
            step_idx: 0,
            step_progress: 0.0,
            current,
            playing: true,
        };

        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));
        ctx.request_repaint();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        assert!((Easing::Linear.apply(0.0) - 0.0).abs() < 1e-6);
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 1e-6);
        assert!((Easing::Linear.apply(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_easing_ease_in() {
        let t = 0.5;
        let result = Easing::EaseIn.apply(t);
        let expected = t * t * t;
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_easing_ease_out() {
        let t = 0.5;
        let result = Easing::EaseOut.apply(t);
        let expected = 1.0 - (1.0 - t).powi(3);
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_easing_ease_in_out() {
        let t = 0.25;
        let result = Easing::EaseInOut.apply(t);
        let expected = 4.0 * t * t * t;
        assert!((result - expected).abs() < 1e-6);

        let t = 0.75;
        let result = Easing::EaseInOut.apply(t);
        let expected = 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0;
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_easing_in_out_bounce() {
        // EaseOutBounce(0) = 0
        assert!((Easing::EaseOutBounce.apply(0.0) - 0.0).abs() < 1e-4);
        // EaseOutBounce(1) = 1
        assert!((Easing::EaseOutBounce.apply(1.0) - 1.0).abs() < 1e-4);
        // EaseInBounce should be inverted
        assert!((Easing::EaseInBounce.apply(0.0) - 0.0).abs() < 1e-4);
        assert!((Easing::EaseInBounce.apply(1.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_easing_cubic_bezier() {
        // Linear bezier should return t value at midpoint
        let result = Easing::CubicBezier(0.0, 0.0, 1.0, 1.0).apply(0.5);
        assert!((result - 0.5).abs() < 0.15); // Simplified implementation approximation

        // Ease-in bezier should be less than 0.5 at midpoint
        let ease_in = Easing::CubicBezier(0.42, 0.0, 1.0, 1.0).apply(0.5);
        assert!(ease_in < 0.5);

        // Ease-out bezier should be greater than 0.5 at midpoint
        let ease_out = Easing::CubicBezier(0.0, 0.0, 0.58, 1.0).apply(0.5);
        assert!(ease_out > 0.5);
    }

    #[test]
    fn test_color_roundtrip() {
        // Test that color conversion produces valid f32 values in 0-255 range
        let c = Color32::from_rgba_unmultiplied(255, 255, 255, 255);
        let f = color_to_f32(c);
        // All components should be valid u8 values cast to f32
        for component in f {
            assert!(component >= 0.0 && component <= 255.0);
        }
    }
}

// ─── Transition Shorthand ─────────────────────────────────────────────────────

/// CSS-like transition config: duration + easing.
#[derive(Clone, Copy, Debug)]
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

// ---------------------------------------------------------------------------
// AnimatedState<T> — animate-on-change wrapper
// ---------------------------------------------------------------------------

/// A value that automatically animates toward a target using spring physics.
///
/// Store in egui memory via `StateSlot` or as part of your widget state.
/// Call `set()` to change the target, `get()` every frame to read the current value.
///
/// # Example
/// ```rust,ignore
/// let mut anim = AnimatedF32::spring(ui.id().with("opacity"), 0.0);
/// anim.set(if hovered { 1.0 } else { 0.0 });
/// let opacity = anim.get(ui.ctx());
/// ```
pub struct AnimatedState<T: crate::style::Lerp + Clone + Copy + 'static + Send + Sync> {
    id: egui::Id,
    pub target: T,
    stiffness: f32,
    damping: f32,
}

impl<T: crate::style::Lerp + Clone + Copy + 'static + Send + Sync> AnimatedState<T> {
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
        // Retrieve stored "from" value (defaults to target = no animation)
        let from: T = ctx
            .data(|d| d.get_temp(self.id.with("__as_from")))
            .unwrap_or(self.target);

        // Animate t: 0.0 → 1.0 using spring
        let spring = Spring::new(self.id.with("__as_spring"), self.stiffness, self.damping);
        let t = spring.animate(ctx, 1.0, 0.0);

        // When animation settles (t ≈ 1.0), update "from" to current target
        if (t - 1.0).abs() < 0.001 {
            ctx.data_mut(|d| d.insert_temp(self.id.with("__as_from"), self.target));
        }

        T::lerp(&from, &self.target, t)
    }

    /// Snap to target immediately, no animation.
    pub fn snap(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(self.id.with("__as_from"), self.target));
        // Reset spring
        ctx.data_mut(|d| {
            d.insert_temp::<f32>(self.id.with("__as_spring").with("__spring_pos"), 1.0)
        });
        ctx.data_mut(|d| {
            d.insert_temp::<f32>(self.id.with("__as_spring").with("__spring_vel"), 0.0)
        });
    }
}

/// Animated f32 value.
pub type AnimatedF32 = AnimatedState<f32>;
/// Animated Color32 value.
pub type AnimatedColor = AnimatedState<egui::Color32>;
/// Animated Vec2 value.
pub type AnimatedVec2 = AnimatedState<egui::Vec2>;
