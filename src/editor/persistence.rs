//! Serializable editor view-state snapshots.

use egui::{Pos2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditorViewSnapshot {
    pub pan: Vec2,
    pub zoom: f32,
    pub cursor: Pos2,
}

impl Default for EditorViewSnapshot {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            cursor: Pos2::ZERO,
        }
    }
}

/// Snapshot bundle for editor interaction state that can be stored in `UndoStack`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorInteractionSnapshot<K> {
    pub view: EditorViewSnapshot,
    pub selected_ids: Vec<K>,
}

impl<K> EditorInteractionSnapshot<K> {
    pub fn new(view: EditorViewSnapshot, selected_ids: impl Into<Vec<K>>) -> Self {
        Self {
            view,
            selected_ids: selected_ids.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::UndoStack;

    #[test]
    fn interaction_snapshot_round_trips_through_undo_stack() {
        let initial = EditorInteractionSnapshot::new(EditorViewSnapshot::default(), vec![1u64]);
        let next = EditorInteractionSnapshot::new(EditorViewSnapshot::default(), vec![2u64]);
        let mut history = UndoStack::new(initial.clone());

        history.push_snapshot(next.clone());

        assert_eq!(history.current(), &next);
        assert_eq!(history.undo(), Some(&initial));
    }
}
