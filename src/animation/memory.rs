/// State for a tween animation, stored in egui memory.
#[derive(Clone, Debug)]
pub(crate) struct TweenMem {
    /// Starting value for the current animation segment.
    pub(crate) from: f32,
    /// Accumulated time (in seconds) since animation started.
    pub(crate) start_dt_acc: f32,
    /// The last target value we were animating toward.
    pub(crate) last_target: f32,
}

/// State for a spring simulation, stored in egui memory.
#[derive(Clone, Debug)]
pub(crate) struct SpringMem {
    /// Current position of the spring.
    pub(crate) position: f32,
    /// Current velocity of the spring.
    pub(crate) velocity: f32,
    /// The last target value (to detect target changes).
    pub(crate) last_target: f32,
}

/// State for an animation sequence, stored in egui memory.
#[derive(Clone, Debug)]
pub(crate) struct SeqMem {
    /// Index of the currently playing step.
    pub(crate) step_idx: usize,
    /// Progress within the current step (0..1).
    pub(crate) step_progress: f32,
    /// Current output value.
    pub(crate) current: f32,
    /// Whether the sequence is actively playing.
    pub(crate) playing: bool,
    /// Initial value at the start of the sequence (for step 0 start).
    pub(crate) initial: f32,
}
