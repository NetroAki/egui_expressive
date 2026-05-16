use egui::{Color32, Pos2, Response, Sense, Shape, Stroke, Ui, Vec2};

pub struct Waveform<'a> {
    pub samples: &'a [f32],
    pub filled: bool,
    pub size: Vec2,
}

/// Compatibility alias for older code and audits that used the explicit
/// `WaveformDisplay` primitive name.
///
/// Prefer [`Waveform`] in new code. Keep this alias until the next major
/// release so downstream apps can migrate without a rendering fork.
pub type WaveformDisplay<'a> = Waveform<'a>;

impl<'a> Waveform<'a> {
    pub fn new(samples: &'a [f32]) -> Self {
        Self {
            samples,
            filled: true,
            size: Vec2::new(320.0, 80.0),
        }
    }
}

impl<'a> egui::Widget for Waveform<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::hover());
        if self.samples.is_empty() {
            return resp;
        }
        let mid = rect.center().y;
        let w = rect.width().max(1.0) as usize;
        let step = (self.samples.len() as f32 / w.max(1) as f32).max(1.0);
        let mut top = Vec::new();
        let mut bottom = Vec::new();
        for x in 0..w {
            let idx = ((x as f32 * step) as usize).min(self.samples.len() - 1);
            let amp = self.samples[idx].clamp(-1.0, 1.0);
            top.push(Pos2::new(
                rect.min.x + x as f32,
                mid - amp * rect.height() * 0.45,
            ));
            bottom.push(Pos2::new(
                rect.min.x + x as f32,
                mid + amp.abs() * rect.height() * 0.45,
            ));
        }
        let p = ui.painter();
        if self.filled {
            let mut poly = top.clone();
            bottom.reverse();
            poly.extend(bottom);
            p.add(Shape::convex_polygon(
                poly,
                Color32::from_rgb(60, 170, 220),
                Stroke::NONE,
            ));
        } else {
            p.add(Shape::line(
                top,
                Stroke::new(1.5, Color32::from_rgb(60, 170, 220)),
            ));
        }
        resp
    }
}
