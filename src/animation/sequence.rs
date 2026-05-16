use super::*;

/// A single step in an animation sequence.
#[derive(Debug, Clone)]
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
            initial: 0.0,
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
            // Convert overshoot to time so it maps correctly to the next step's duration
            let overshoot_time = (mem.step_progress - 1.0) * current_step.duration;

            // Snap to current step's target
            if mem.step_idx < self.steps.len() {
                mem.current = self.steps[mem.step_idx].target;
            }

            // Advance to next step
            if mem.step_idx + 1 < self.steps.len() {
                mem.step_idx += 1;
                let next_duration = self.steps[mem.step_idx].duration.max(1e-6);
                mem.step_progress = (overshoot_time / next_duration).min(1.0);
            } else {
                // Sequence complete, stop playing
                mem.playing = false;
                mem.step_progress = 0.0;
                // Snap to final target on completion
                if let Some(last_step) = self.steps.last() {
                    mem.current = last_step.target;
                }
            }
        }

        // Compute output for current step (only while playing)
        if mem.playing && mem.step_idx < self.steps.len() {
            let step = &self.steps[mem.step_idx];

            // Determine start value (previous step's target or initial value at sequence start)
            let start_value = if mem.step_idx == 0 {
                mem.initial
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
            initial: 0.0,
        };

        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));
    }

    /// Start or restart the sequence from the beginning.
    ///
    /// # Arguments
    /// * `ctx` - egui context
    pub fn play(&self, ctx: &Context) {
        let mem_id = self.id.with("__seq_mem");

        // Load current state to preserve current value as the animation start
        let current: f32 = ctx
            .memory(|m| m.data.get_temp::<SeqMem>(mem_id))
            .map(|m| m.current)
            .unwrap_or(0.0);

        let mem = SeqMem {
            step_idx: 0,
            step_progress: 0.0,
            current,
            playing: true,
            initial: current, // capture current value as step 0 start
        };

        ctx.memory_mut(|m| m.data.insert_temp(mem_id, mem));
        ctx.request_repaint();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
