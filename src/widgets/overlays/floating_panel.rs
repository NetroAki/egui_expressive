use egui::{Id, Pos2, Rect, Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatingPanelState {
    pub pos: Pos2,
    pub size: Vec2,
    pub docked: bool,
}

impl Default for FloatingPanelState {
    fn default() -> Self {
        Self {
            pos: Pos2::new(40.0, 40.0),
            size: Vec2::new(320.0, 180.0),
            docked: false,
        }
    }
}

pub struct FloatingPanel<'a> {
    title: &'a str,
    id: Id,
    state: Option<&'a mut FloatingPanelState>,
    pos: Option<Pos2>,
    size: Option<Vec2>,
}

impl<'a> FloatingPanel<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            id: Id::new(title),
            state: None,
            pos: None,
            size: None,
        }
    }
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Id::new(id);
        self
    }
    pub fn state(mut self, state: &'a mut FloatingPanelState) -> Self {
        self.state = Some(state);
        self
    }
    pub fn pos(mut self, pos: Pos2) -> Self {
        self.pos = Some(pos);
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }
    pub fn show(self, ui: &mut Ui, add: impl FnOnce(&mut Ui)) -> Response {
        let mut fallback = FloatingPanelState {
            pos: self.pos.unwrap_or(Pos2::new(40.0, 40.0)),
            size: self.size.unwrap_or(Vec2::new(320.0, 180.0)),
            docked: false,
        };
        let state = self.state.unwrap_or(&mut fallback);
        let area = egui::Area::new(self.id)
            .movable(false)
            .fixed_pos(state.pos)
            .order(egui::Order::Foreground);
        area.show(ui.ctx(), |ui| {
            let rect = Rect::from_min_size(ui.min_rect().min, state.size);
            let drag = ui.allocate_rect(rect, Sense::click_and_drag());
            if drag.dragged() {
                state.pos += drag.drag_delta();
            }
            egui::Frame::window(ui.style()).show(ui, |ui| {
                ui.set_min_size(state.size);
                ui.horizontal(|ui| {
                    ui.strong(self.title);
                    if ui
                        .small_button(if state.docked { "Undock" } else { "Dock" })
                        .clicked()
                    {
                        state.docked = !state.docked;
                    }
                });
                ui.separator();
                add(ui);
                let resize = ui.allocate_response(Vec2::splat(14.0), Sense::drag());
                if resize.dragged() {
                    state.size += resize.drag_delta();
                    state.size.x = state.size.x.max(160.0);
                    state.size.y = state.size.y.max(96.0);
                }
            });
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_panel_state_persists_geometry() {
        let state = FloatingPanelState::default();
        assert_eq!(state.pos, Pos2::new(40.0, 40.0));
        assert_eq!(state.size, Vec2::new(320.0, 180.0));
    }
}
