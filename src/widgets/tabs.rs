use egui::{Response, Sense, Ui, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSetState {
    pub selected: usize,
}

impl TabSetState {
    pub fn new(selected: usize) -> Self {
        Self { selected }
    }

    pub fn selected_or_first(&self, len: usize) -> usize {
        if len == 0 {
            0
        } else {
            self.selected.min(len - 1)
        }
    }

    pub fn recover(&mut self, len: usize) {
        self.selected = self.selected_or_first(len);
    }
}

pub struct TabBar<'a> {
    selected: &'a mut usize,
    tabs: Vec<String>,
}

impl<'a> TabBar<'a> {
    pub fn new(selected: &'a mut usize, tabs: impl Into<Vec<String>>) -> Self {
        Self {
            selected,
            tabs: tabs.into(),
        }
    }
}

impl<'a> egui::Widget for TabBar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let resp = ui.allocate_response(Vec2::new(ui.available_width(), 24.0), Sense::hover());
        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter().enumerate() {
                let selected = *self.selected == i;
                if ui.selectable_label(selected, tab).clicked() {
                    *self.selected = i;
                }
            }
        });
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_state_falls_back_to_existing_tab() {
        let mut state = TabSetState::new(9);
        state.recover(3);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn tab_state_handles_empty_sets() {
        let mut state = TabSetState::new(2);
        state.recover(0);
        assert_eq!(state.selected, 0);
    }
}
