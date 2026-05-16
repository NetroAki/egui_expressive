//! Raw color and opacity utilities for `Tw`.

use egui::Color32;

use crate::tailwind::Tw;

impl Tw {
    pub fn bg(mut self, c: Color32) -> Self {
        self.bg = Some(c);
        self
    }
    pub fn bg_alpha(mut self, c: Color32, alpha: f32) -> Self {
        self.bg = Some(Color32::from_rgba_premultiplied(
            c.r(),
            c.g(),
            c.b(),
            (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
        ));
        self
    }
    pub fn text_color(mut self, c: Color32) -> Self {
        self.fg = Some(c);
        self
    }
    pub fn border_color(mut self, c: Color32) -> Self {
        self.border_color = Some(c);
        self
    }
    pub fn background(self, c: Color32) -> Self {
        self.bg(c)
    }
    pub fn foreground_color(self, c: Color32) -> Self {
        self.text_color(c)
    }
    pub fn opacity(mut self, o: f32) -> Self {
        self.opacity = o.clamp(0.0, 1.0);
        self
    }
    pub fn opacity_50(self) -> Self {
        self.opacity(0.5)
    }
    pub fn opacity_75(self) -> Self {
        self.opacity(0.75)
    }
}
