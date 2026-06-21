use egui::{
    Align, Color32, CornerRadius, CursorIcon, Id, Margin, Pos2, Response, RichText, Sense, Ui, Vec2,
};

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
            size: clamped_panel_size(self.size.unwrap_or(Vec2::new(320.0, 180.0))),
            docked: false,
        };
        let state = self.state.unwrap_or(&mut fallback);
        state.size = clamped_panel_size(state.size);
        let drag_anchor_id = self.id.with("drag_pointer_offset");
        let resize_anchor_id = self.id.with("resize_anchor");
        let area = egui::Area::new(self.id)
            .movable(false)
            .fixed_pos(state.pos)
            .order(egui::Order::Foreground);
        let area_response = area.show(ui.ctx(), |ui| {
            egui::Frame::window(ui.style())
                .fill(Color32::from_rgb(17, 22, 34))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(89, 144, 224)))
                .corner_radius(CornerRadius::same(14))
                .show(ui, |ui| {
                    ui.set_min_size(state.size);
                    let (drag_handle, dock_clicked) = egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(90, 150, 255, 26))
                        .stroke(egui::Stroke::new(
                            1.0,
                            Color32::from_rgba_unmultiplied(120, 175, 255, 72),
                        ))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let drag_width = (ui.available_width() - 92.0).max(96.0);
                                let drag = ui
                                    .add_sized(
                                        [drag_width, 24.0],
                                        egui::Label::new(
                                            RichText::new(format!("⋮⋮  {}", self.title))
                                                .strong()
                                                .color(Color32::from_rgb(241, 245, 255)),
                                        )
                                        .sense(Sense::click_and_drag()),
                                    )
                                    .on_hover_cursor(CursorIcon::Grab);
                                let dock_clicked = ui
                                    .small_button(if state.docked { "Undock" } else { "Dock" })
                                    .clicked();
                                (drag, dock_clicked)
                            })
                            .inner
                        })
                        .inner;
                    let drag_cursor = if drag_handle.dragged() {
                        CursorIcon::Grabbing
                    } else {
                        CursorIcon::Grab
                    };
                    let drag_handle = drag_handle.on_hover_cursor(drag_cursor);
                    if drag_handle.drag_started() {
                        if let Some(pointer) = ui.ctx().pointer_interact_pos() {
                            ui.ctx()
                                .data_mut(|d| d.insert_temp(drag_anchor_id, pointer - state.pos));
                        }
                    }
                    if drag_handle.dragged() {
                        if let Some(pointer) = ui.ctx().pointer_interact_pos() {
                            let pointer_offset = ui
                                .ctx()
                                .data(|d| d.get_temp::<Vec2>(drag_anchor_id))
                                .unwrap_or(pointer - state.pos);
                            state.pos = pointer - pointer_offset;
                        }
                    }
                    if dock_clicked {
                        state.docked = !state.docked;
                    }
                    ui.separator();
                    add(ui);
                    ui.add_space(8.0);
                    let resize = ui
                        .with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(16.0), Sense::drag());
                            let response = response.on_hover_cursor(CursorIcon::ResizeNwSe);
                            let stroke = egui::Stroke::new(1.5, Color32::from_rgb(129, 183, 255));
                            ui.painter().line_segment(
                                [
                                    rect.right_bottom() - Vec2::new(12.0, 4.0),
                                    rect.right_bottom() - Vec2::new(4.0, 12.0),
                                ],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [
                                    rect.right_bottom() - Vec2::new(8.0, 2.0),
                                    rect.right_bottom() - Vec2::new(2.0, 8.0),
                                ],
                                stroke,
                            );
                            response
                        })
                        .inner;
                    if resize.drag_started() {
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(resize_anchor_id, state.size));
                    }
                    if resize.dragged() {
                        let anchor = ui
                            .ctx()
                            .data(|d| d.get_temp::<Vec2>(resize_anchor_id))
                            .unwrap_or(state.size);
                        state.size = clamped_panel_size(anchor + resize.drag_delta());
                    }
                })
                .response
        });
        area_response.inner.union(area_response.response)
    }
}

fn clamped_panel_size(size: Vec2) -> Vec2 {
    Vec2::new(size.x.max(160.0), size.y.max(96.0))
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

    #[test]
    fn floating_panel_size_clamps_to_minimums() {
        assert_eq!(
            clamped_panel_size(Vec2::new(40.0, 20.0)),
            Vec2::new(160.0, 96.0)
        );
    }
}
