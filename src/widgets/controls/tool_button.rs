use egui::{Response, Ui, Vec2};

/// Generic active/disabled tool button with action id metadata.
pub struct ToolButton<'a> {
    pub action_id: String,
    label: String,
    active: Option<&'a mut bool>,
    enabled: bool,
    size: Vec2,
}

impl<'a> ToolButton<'a> {
    pub fn new(action_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            active: None,
            enabled: true,
            size: Vec2::new(32.0, 28.0),
        }
    }
    pub fn active(mut self, active: &'a mut bool) -> Self {
        self.active = Some(active);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl<'a> egui::Widget for ToolButton<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::click());
        if self.enabled && response.clicked() {
            if let Some(active) = self.active.as_deref_mut() {
                *active = !*active;
            }
        }
        let selected = self.active.as_deref().copied().unwrap_or(false);
        let visuals = ui.visuals();
        let fill = if selected {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.bg_fill
        } else {
            visuals.widgets.inactive.bg_fill
        };
        ui.painter().rect_filled(rect, 4.0, fill);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &self.label,
            egui::FontId::proportional(12.0),
            visuals.text_color(),
        );
        response.on_hover_text(self.action_id)
    }
}
