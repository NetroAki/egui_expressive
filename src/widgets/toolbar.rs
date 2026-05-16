use egui::{Response, Sense, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarItemKind {
    Button,
    Toggle,
    Spacer,
    Spring,
    Overflow,
}

pub struct ToolbarItem {
    pub id: String,
    pub label: String,
    pub kind: ToolbarItemKind,
    pub icon: Option<char>,
    pub active: bool,
    pub enabled: bool,
    pub width: f32,
}

pub struct ToolbarStrip<'a> {
    items: &'a mut [ToolbarItem],
    pub dragged: Option<usize>,
}

impl ToolbarItem {
    pub fn button(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: ToolbarItemKind::Button,
            icon: None,
            active: false,
            enabled: true,
            width: 64.0,
        }
    }
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<'a> ToolbarStrip<'a> {
    pub fn new(items: &'a mut [ToolbarItem]) -> Self {
        Self {
            items,
            dragged: None,
        }
    }
    pub fn dragged(mut self, index: Option<usize>) -> Self {
        self.dragged = index;
        self
    }
}

impl<'a> egui::Widget for ToolbarStrip<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut response = ui.allocate_response(Vec2::ZERO, Sense::hover());
        ui.horizontal(|ui| {
            for (index, item) in self.items.iter_mut().enumerate() {
                match item.kind {
                    ToolbarItemKind::Button
                    | ToolbarItemKind::Toggle
                    | ToolbarItemKind::Overflow => {
                        let label = match item.icon {
                            Some(icon) => format!("{} {}", icon, item.label),
                            None => item.label.clone(),
                        };
                        let button = egui::Button::new(label).selected(item.active);
                        let resp = ui
                            .add_enabled(item.enabled, button)
                            .on_hover_cursor(egui::CursorIcon::Grab);
                        if resp.drag_started() {
                            response.mark_changed();
                        }
                        if self.dragged == Some(index) {
                            ui.painter().rect_stroke(
                                resp.rect,
                                3.0,
                                ui.visuals().selection.stroke,
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                    ToolbarItemKind::Spacer => {
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(item.width, 20.0), Sense::drag());
                        if resp.dragged() {
                            item.width = (item.width + resp.drag_delta().x).max(4.0);
                            response.mark_changed();
                        }
                        ui.painter().vline(
                            rect.center().x,
                            rect.y_range(),
                            ui.visuals().widgets.noninteractive.bg_stroke,
                        );
                    }
                    ToolbarItemKind::Spring => ui.add_space(ui.available_width()),
                }
            }
        });
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_items_track_icon_active_enabled_state() {
        let item = ToolbarItem::button("snap", "Snap")
            .icon('S')
            .active(true)
            .enabled(false);
        assert_eq!(item.icon, Some('S'));
        assert!(item.active);
        assert!(!item.enabled);
    }
}
