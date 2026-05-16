/// Snapshot entry stored by an [`UndoStack`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoEntry<T> {
    pub snapshot: T,
    pub label: Option<String>,
    pub merge_key: Option<String>,
}

impl<T> UndoEntry<T> {
    pub fn new(snapshot: T) -> Self {
        Self {
            snapshot,
            label: None,
            merge_key: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn merge_key(mut self, merge_key: impl Into<String>) -> Self {
        self.merge_key = Some(merge_key.into());
        self
    }
}

/// Unbounded snapshot-based undo/redo history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoStack<T> {
    entries: Vec<UndoEntry<T>>,
    cursor: usize,
}

impl<T> UndoStack<T> {
    pub fn new(initial: T) -> Self {
        Self {
            entries: vec![UndoEntry::new(initial)],
            cursor: 0,
        }
    }

    pub fn current(&self) -> &T {
        &self.entries[self.cursor].snapshot
    }

    pub fn current_entry(&self) -> &UndoEntry<T> {
        &self.entries[self.cursor]
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }

    pub fn push(&mut self, entry: UndoEntry<T>) {
        if self.can_redo() {
            self.entries.truncate(self.cursor + 1);
        }

        let can_merge = entry.merge_key.is_some()
            && self.entries[self.cursor].merge_key == entry.merge_key
            && self.cursor > 0;

        if can_merge {
            self.entries[self.cursor] = entry;
        } else {
            self.entries.push(entry);
            self.cursor = self.entries.len() - 1;
        }
    }

    pub fn push_snapshot(&mut self, snapshot: T) {
        self.push(UndoEntry::new(snapshot));
    }

    pub fn undo(&mut self) -> Option<&T> {
        if !self.can_undo() {
            return None;
        }
        self.cursor -= 1;
        Some(self.current())
    }

    pub fn redo(&mut self) -> Option<&T> {
        if !self.can_redo() {
            return None;
        }
        self.cursor += 1;
        Some(self.current())
    }

    pub fn clear_to(&mut self, snapshot: T) {
        self.entries.clear();
        self.entries.push(UndoEntry::new(snapshot));
        self.cursor = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_redo_move_cursor_through_snapshots() {
        let mut stack = UndoStack::new(0);
        stack.push_snapshot(1);
        stack.push_snapshot(2);

        assert_eq!(stack.current(), &2);
        assert_eq!(stack.undo(), Some(&1));
        assert_eq!(stack.undo(), Some(&0));
        assert_eq!(stack.undo(), None);
        assert_eq!(stack.redo(), Some(&1));
    }

    #[test]
    fn push_after_undo_invalidates_redo_history() {
        let mut stack = UndoStack::new("a");
        stack.push_snapshot("b");
        stack.push_snapshot("c");
        stack.undo();

        stack.push_snapshot("d");

        assert_eq!(stack.current(), &"d");
        assert!(!stack.can_redo());
        assert_eq!(stack.len(), 3);
    }

    #[test]
    fn matching_merge_key_replaces_current_entry() {
        let mut stack = UndoStack::new(0);
        stack.push(UndoEntry::new(1).merge_key("typing"));
        stack.push(UndoEntry::new(2).merge_key("typing"));

        assert_eq!(stack.current(), &2);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.undo(), Some(&0));
    }

    #[test]
    fn clear_to_resets_history() {
        let mut stack = UndoStack::new(0);
        stack.push_snapshot(1);
        stack.clear_to(9);

        assert_eq!(stack.current(), &9);
        assert_eq!(stack.len(), 1);
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }
}
