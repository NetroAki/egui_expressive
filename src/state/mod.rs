//! Typed persistent state and state machines.

use egui::Context;
use std::marker::PhantomData;

/// Bounded durable-persistence slot metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceSlot {
    pub key: String,
    pub max_bytes: usize,
    pub version: u32,
}

impl PersistenceSlot {
    pub fn new(key: impl Into<String>, max_bytes: usize) -> Self {
        Self {
            key: key.into(),
            max_bytes,
            version: 1,
        }
    }

    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    pub fn accepts(&self, bytes: usize) -> bool {
        bytes <= self.max_bytes
    }
}

/// Registry for bounded app-owned persistence. The library tracks keys, caps,
/// and versions without dictating the storage backend.
#[derive(Debug, Clone, Default)]
pub struct PersistenceRegistry {
    slots: Vec<PersistenceSlot>,
}

impl PersistenceRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, slot: PersistenceSlot) {
        self.slots.push(slot);
    }
    pub fn get(&self, key: &str) -> Option<&PersistenceSlot> {
        self.slots.iter().find(|s| s.key == key)
    }
    pub fn validate(&self, key: &str, bytes: usize) -> bool {
        self.get(key).is_some_and(|slot| slot.accepts(bytes))
    }
    pub fn iter(&self) -> impl Iterator<Item = &PersistenceSlot> {
        self.slots.iter()
    }
}

/// Single-producer/single-consumer style fixed-size bridge for audio/control
/// data snapshots. This is intentionally allocation-free after construction.
#[derive(Debug, Clone)]
pub struct AudioUiBridge<T: Copy + Default, const N: usize> {
    buffer: [T; N],
    write_index: usize,
    len: usize,
}

impl<T: Copy + Default, const N: usize> AudioUiBridge<T, N> {
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); N],
            write_index: 0,
            len: 0,
        }
    }
    pub fn push(&mut self, value: T) {
        if N == 0 {
            return;
        }
        self.buffer[self.write_index] = value;
        self.write_index = (self.write_index + 1) % N;
        self.len = (self.len + 1).min(N);
    }
    pub fn latest(&self) -> Option<T> {
        if self.len == 0 || N == 0 {
            None
        } else {
            Some(self.buffer[(self.write_index + N - 1) % N])
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Copy + Default, const N: usize> Default for AudioUiBridge<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod primitive_state_tests {
    use super::*;

    #[test]
    fn persistence_registry_enforces_byte_caps() {
        let mut registry = PersistenceRegistry::new();
        registry.register(PersistenceSlot::new("layout.panels", 8).version(2));
        assert!(registry.validate("layout.panels", 8));
        assert!(!registry.validate("layout.panels", 9));
        assert_eq!(registry.get("layout.panels").unwrap().version, 2);
    }

    #[test]
    fn audio_ui_bridge_keeps_latest_fixed_capacity() {
        let mut bridge: AudioUiBridge<i32, 2> = AudioUiBridge::new();
        assert!(bridge.is_empty());
        bridge.push(1);
        bridge.push(2);
        bridge.push(3);
        assert_eq!(bridge.len(), 2);
        assert_eq!(bridge.latest(), Some(3));
    }
}

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
