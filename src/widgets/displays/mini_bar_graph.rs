use egui::{Color32, Response, Sense, Ui, Vec2};
pub struct MiniBarGraph<'a> {
    pub values: &'a [f32],
    pub size: Vec2,
}
impl<'a> MiniBarGraph<'a> {
    pub fn new(values: &'a [f32]) -> Self {
        Self {
            values,
            size: Vec2::new(120.0, 40.0),
        }
    }
}
impl<'a> egui::Widget for MiniBarGraph<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::hover());
        let bw = rect.width() / self.values.len().max(1) as f32;
        for (i, v) in self.values.iter().enumerate() {
            let h = rect.height() * v.clamp(0.0, 1.0);
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::Pos2::new(rect.min.x + i as f32 * bw, rect.max.y - h),
                    egui::Vec2::new((bw - 1.0).max(1.0), h),
                ),
                0.0,
                Color32::from_rgb(80, 190, 120),
            );
        }
        resp
    }
}
