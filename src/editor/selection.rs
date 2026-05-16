//! Generic selected-ID model for editor surfaces.

use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Toggle,
    Add,
}

#[derive(Debug, Clone)]
pub struct SelectionModel<K> {
    selected: HashSet<K>,
    anchor: Option<K>,
}

impl<K> Default for SelectionModel<K> {
    fn default() -> Self {
        Self {
            selected: HashSet::new(),
            anchor: None,
        }
    }
}

impl<K> SelectionModel<K>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected(&self) -> &HashSet<K> {
        &self.selected
    }

    pub fn anchor(&self) -> Option<&K> {
        self.anchor.as_ref()
    }

    pub fn is_selected(&self, id: &K) -> bool {
        self.selected.contains(id)
    }

    pub fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    pub fn select_only(&mut self, id: K) {
        self.selected.clear();
        self.selected.insert(id.clone());
        self.anchor = Some(id);
    }

    pub fn add(&mut self, id: K) {
        self.selected.insert(id.clone());
        self.anchor = Some(id);
    }

    pub fn toggle(&mut self, id: K) {
        if !self.selected.remove(&id) {
            self.selected.insert(id.clone());
            self.anchor = Some(id);
        }
    }

    pub fn apply(&mut self, id: K, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => self.select_only(id),
            SelectionMode::Toggle => self.toggle(id),
            SelectionMode::Add => self.add(id),
        }
    }

    pub fn replace_all(&mut self, ids: impl IntoIterator<Item = K>) {
        self.selected = ids.into_iter().collect();
        self.anchor = self.selected.iter().next().cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_model_applies_modes() {
        let mut selection = SelectionModel::new();
        selection.apply(1u32, SelectionMode::Replace);
        assert!(selection.is_selected(&1));
        selection.apply(2, SelectionMode::Add);
        assert!(selection.is_selected(&1));
        assert!(selection.is_selected(&2));
        selection.apply(1, SelectionMode::Toggle);
        assert!(!selection.is_selected(&1));
    }
}
