use egui::{Color32, Response, Sense, Ui, Vec2};
pub struct SpectrumDisplay<'a> {
    pub bins: &'a [f32],
    pub size: Vec2,
}
impl<'a> SpectrumDisplay<'a> {
    pub fn new(bins: &'a [f32]) -> Self {
        Self {
            bins,
            size: Vec2::new(320.0, 100.0),
        }
    }
}
impl<'a> egui::Widget for SpectrumDisplay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::hover());
        let bw = rect.width() / self.bins.len().max(1) as f32;
        for (i, b) in self.bins.iter().enumerate() {
            let h = rect.height() * b.clamp(0.0, 1.0);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::Pos2::new(rect.min.x + i as f32 * bw, rect.max.y - h),
                    egui::Vec2::new((bw - 1.0).max(1.0), h),
                ),
                0.0,
                Color32::from_rgb(180, 120, 220),
            );
        }
        resp
    }
}
