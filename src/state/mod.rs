#![allow(dead_code)]

//! Typed persistent state and state machines.

use egui::Context;
use std::marker::PhantomData;

/// A single persistent state slot with typed memory storage.
#[derive(Debug, Clone)]
pub struct StateSlot<T: Clone + Send + Sync + 'static> {
    id: egui::Id,
    _phantom: PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> StateSlot<T> {
    /// Create a new state slot with the given ID.
    pub fn new(id: egui::Id) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }

    /// Get the current value from context memory.
    pub fn get(&self, ctx: &Context) -> Option<T> {
        ctx.memory(|m| m.data.get_temp::<T>(self.id))
    }

    /// Get the current value, or insert a newly computed default if none exists.
    pub fn get_or_insert(&self, ctx: &Context, default: impl FnOnce() -> T) -> T {
        if let Some(value) = self.get(ctx) {
            value
        } else {
            let value = default();
            self.set(ctx, value.clone());
            value
        }
    }

    /// Set a new value in context memory.
    pub fn set(&self, ctx: &Context, value: T) {
        ctx.memory_mut(|m| m.data.insert_temp(self.id, value));
    }

    /// Update the value by applying a function to it.
    /// If no value exists, uses `default` as the initial value.
    pub fn update<F: FnOnce(T) -> T>(&self, ctx: &Context, default: T, f: F) {
        let value = self.get_or_insert(ctx, || default);
        let new_value = f(value);
        self.set(ctx, new_value);
    }

    /// Clear the value from context memory.
    pub fn clear(&self, ctx: &Context) {
        ctx.memory_mut(|m| m.data.remove::<T>(self.id));
    }
}

/// Generic state machine with transition tracking.
#[derive(Debug, Clone)]
pub struct StateMachine<S: Clone + PartialEq + Send + Sync + 'static> {
    slot: StateSlot<S>,
}

impl<S: Clone + PartialEq + Send + Sync + 'static> StateMachine<S> {
    /// Create a new state machine with the given ID.
    pub fn new(id: egui::Id) -> Self {
        Self {
            slot: StateSlot::new(id),
        }
    }

    /// Get the current state, or set to `default` if none exists.
    pub fn state(&self, ctx: &Context, default: S) -> S {
        self.slot.get_or_insert(ctx, || default)
    }

    /// Set a new state directly.
    pub fn set(&self, ctx: &Context, new_state: S) {
        self.slot.set(ctx, new_state);
    }

    /// Transition from one of the `from` states to `to`.
    /// Returns `true` if the transition occurred.
    pub fn transition(&self, ctx: &Context, from: &[S], to: S) -> bool {
        let current = self.state(ctx, to.clone());
        if from.iter().any(|s| s == &current) {
            self.set(ctx, to);
            true
        } else {
            false
        }
    }

    /// Returns `true` if the state changed this frame compared to the previous frame.
    pub fn changed(&self, ctx: &Context, default: &S) -> bool {
        let prev_id = self.slot.id.with("__prev");
        let prev_state: Option<S> = ctx.memory(|m| m.data.get_temp(prev_id));
        let current_state = self.state(ctx, default.clone());

        let changed = prev_state.as_ref() != Some(&current_state);

        // Update previous state for next frame
        ctx.memory_mut(|m| m.data.insert_temp(prev_id, current_state));

        changed
    }
}

/// Interaction state for a widget, capturing pointer and focus state.
#[derive(Debug, Clone, Copy, Default)]
pub struct InteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub dragging: bool,
    pub focused: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub secondary_clicked: bool,
}

impl InteractionState {
    /// Construct `InteractionState` from an egui response.
    pub fn from_response(r: &egui::Response) -> Self {
        Self {
            hovered: r.hovered(),
            pressed: r.is_pointer_button_down_on(),
            dragging: r.dragged(),
            focused: r.has_focus(),
            clicked: r.clicked(),
            double_clicked: r.double_clicked(),
            secondary_clicked: r.secondary_clicked(),
        }
    }

    /// Returns `true` if any interactive state is active.
    pub fn is_active(&self) -> bool {
        self.hovered || self.pressed || self.dragging || self.focused
    }

    /// Determine the visual variant based on interaction and widget state.
    pub fn variant(&self, selected: bool, disabled: bool) -> crate::style::VisualVariant {
        use crate::style::VisualVariant;
        if disabled {
            VisualVariant::Disabled
        } else if self.pressed || self.dragging {
            VisualVariant::Pressed
        } else if self.focused {
            VisualVariant::Focused
        } else if selected {
            VisualVariant::Selected
        } else if self.hovered {
            VisualVariant::Hovered
        } else {
            VisualVariant::Inactive
        }
    }
}
