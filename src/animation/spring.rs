use super::*;

/// Spring physics animation for natural-feeling motion.
///
/// A `Spring` simulates a mass-spring-damper system to produce smooth,
/// physically-based animations that overshoot and settle at the target.
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

    /// Reset the spring to a target value immediately (for use by snap()).
    pub(crate) fn reset_to(&self, ctx: &Context, target: f32) {
        let mem_id = self.id.with("__spring_mem");
        ctx.memory_mut(|m| {
            m.data.insert_temp(
                mem_id,
                SpringMem {
                    position: target,
                    velocity: 0.0,
                    last_target: target,
                },
            )
        });
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
