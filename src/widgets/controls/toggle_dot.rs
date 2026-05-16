use super::dot_state::DotState;

/// A small colored dot toggle button.
pub struct ToggleDot<'a> {
    state: &'a mut DotState,
    size: f32,
}

impl<'a> ToggleDot<'a> {
    pub fn new(state: &'a mut DotState) -> Self {
        Self { state, size: 8.0 }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl<'a> egui::Widget for ToggleDot<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(self.size + 4.0), egui::Sense::click());
        if response.clicked() {
            *self.state = self.state.toggle();
        }
        let painter = ui.painter();
        let center = rect.center();
        let color = self.state.color();
        let border = if response.hovered() {
            ui.visuals().widgets.hovered.bg_stroke.color
        } else {
            ui.visuals().widgets.inactive.bg_stroke.color
        };
        painter.circle_filled(center, self.size * 0.5, color);
        painter.circle_stroke(center, self.size * 0.5, egui::Stroke::new(1.0, border));
        response
    }
}
