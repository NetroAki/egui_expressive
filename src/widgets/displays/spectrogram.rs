use egui::{Color32, Response, Sense, Ui, Vec2};
pub struct SpectrogramDisplay<'a> {
    pub rows: &'a [Vec<f32>],
    pub size: Vec2,
}
impl<'a> SpectrogramDisplay<'a> {
    pub fn new(rows: &'a [Vec<f32>]) -> Self {
        Self {
            rows,
            size: Vec2::new(320.0, 120.0),
        }
    }
}
impl<'a> egui::Widget for SpectrogramDisplay<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::hover());
        if self.rows.is_empty() {
            return resp;
        }
        let rh = rect.height() / self.rows.len() as f32;
        for (r, row) in self.rows.iter().enumerate() {
            let cw = rect.width() / row.len().max(1) as f32;
            for (c, v) in row.iter().enumerate() {
                let col = Color32::from_rgb((v.clamp(0.0, 1.0) * 255.0) as u8, 80, 120);
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        egui::Pos2::new(rect.min.x + c as f32 * cw, rect.min.y + r as f32 * rh),
                        egui::Vec2::new(cw, rh),
                    ),
                    0.0,
                    col,
                );
            }
        }
        resp
    }
}
