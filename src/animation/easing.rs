#[derive(Clone, Copy, Debug, PartialEq)]
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
pub(crate) fn cubic_ease_in(t: f32) -> f32 {
    t * t * t
}

#[inline]
pub(crate) fn cubic_ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

#[inline]
pub(crate) fn cubic_ease_in_out(t: f32) -> f32 {
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
pub(crate) fn ease_in_back(t: f32) -> f32 {
    BACK_C3 * t * t * t - BACK_C1 * t * t
}

#[inline]
pub(crate) fn ease_out_back(t: f32) -> f32 {
    1.0 + BACK_C3 * (t - 1.0).powi(3) + BACK_C1 * (t - 1.0).powi(2)
}

#[inline]
pub(crate) fn ease_in_out_back(t: f32) -> f32 {
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
// BOUNCE_D1_2 and BOUNCE_D1_3 are part of the original formula but the
// implementation only uses the first 4 segments with d1=2.75 as divisor.
const BOUNCE_N1: f32 = 7.5625;
const BOUNCE_D1_1: f32 = 2.75;

pub(crate) fn ease_out_bounce(t: f32) -> f32 {
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

pub(crate) fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

// ---------------------------------------------------------------------------
// Cubic Bezier
// ---------------------------------------------------------------------------

/// Evaluate cubic bezier curve at parameter t using De Casteljau's algorithm.
///
/// Control points: P0=(0,0), P1=(p1x,p1y), P2=(p2x,p2y), P3=(1,1)
pub(crate) fn cubic_bezier(t: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32) -> f32 {
    // Find t_bezier from t_input using Newton-Raphson (8 iterations)
    let t_bezier = solve_t_for_x(t, p1x, p2x, 8);

    // Evaluate y at t_bezier
    eval_bezier_y(t_bezier, p1y, p2y)
}

/// Find t such that bezier_x(t) ≈ x_input using Newton-Raphson.
pub(crate) fn solve_t_for_x(x_input: f32, p1x: f32, p2x: f32, iterations: usize) -> f32 {
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
pub(crate) fn bezier_x(t: f32, p1x: f32, p2x: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    // Bx(t) = (1-t)³·0 + 3(1-t)²t·p1x + 3(1-t)t²·p2x + t³·1
    3.0 * mt2 * t * p1x + 3.0 * mt * t2 * p2x + t3
}

/// Derivative of X component: B'x(t) = 3(1-t)²p1x + 6(1-t)tp2x + 3t²
#[inline]
pub(crate) fn bezier_x_derivative(t: f32, p1x: f32, p2x: f32) -> f32 {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    3.0 * mt2 * p1x + 6.0 * mt * t * p2x + 3.0 * t2
}

/// Evaluate Y component of cubic bezier at t.
#[inline]
pub(crate) fn eval_bezier_y(t: f32, p1y: f32, p2y: f32) -> f32 {
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
