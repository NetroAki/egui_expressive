use egui::{Response, Ui};
pub struct ProgressOverlay<'a> {
    pub progress: &'a mut f32,
}
impl<'a> egui::Widget for ProgressOverlay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.add(egui::ProgressBar::new((*self.progress).clamp(0.0, 1.0)));
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}
