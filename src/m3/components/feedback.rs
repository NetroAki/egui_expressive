use crate::m3::M3Theme;
use egui::{
    Color32, CornerRadius, Id, Margin, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
    Widget,
};

pub struct M3LinearProgress {
    value: Option<f32>,
    id: Id,
    height: f32,
}
impl M3LinearProgress {
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            id: Id::new("m3_linear_progress"),
            height: 4.0,
        }
    }
    pub fn indeterminate(id: impl std::hash::Hash) -> Self {
        Self {
            value: None,
            id: Id::new(id),
            height: 4.0,
        }
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}
impl Widget for M3LinearProgress {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), self.height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let rounding = CornerRadius::same((self.height / 2.0) as u8);
            let painter = ui.painter();
            painter.rect_filled(rect, rounding, c.surface_variant);
            match self.value {
                Some(v) => {
                    let fill_w = rect.width() * v;
                    painter.rect_filled(
                        Rect::from_min_size(rect.min, Vec2::new(fill_w, self.height)),
                        rounding,
                        c.primary,
                    );
                }
                None => {
                    let duration = 1.5;
                    let phase = ((self.id.value() % 1000) as f64 / 1000.0) * duration;
                    let t = (((ui.input(|i| i.time) + phase) % duration) / duration) as f32;
                    let bar_w = rect.width() * 0.4;
                    let x = rect.left() + (rect.width() + bar_w) * t - bar_w;
                    let x0 = x.max(rect.left());
                    let x1 = (x + bar_w).min(rect.right());
                    if x1 > x0 {
                        painter.rect_filled(
                            Rect::from_min_max(
                                Pos2::new(x0, rect.top()),
                                Pos2::new(x1, rect.top() + self.height),
                            ),
                            rounding,
                            c.primary,
                        );
                    }
                    ui.ctx().request_repaint();
                }
            }
        }
        response
    }
}

pub struct M3CircularProgress {
    value: Option<f32>,
    id: Id,
    size: f32,
    stroke_width: f32,
}
impl M3CircularProgress {
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            id: Id::new("m3_circ"),
            size: 48.0,
            stroke_width: 4.0,
        }
    }
    pub fn indeterminate(id: impl std::hash::Hash) -> Self {
        Self {
            value: None,
            id: Id::new(id),
            size: 48.0,
            stroke_width: 4.0,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_width = w;
        self
    }
}
impl Widget for M3CircularProgress {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            let center = rect.center();
            let radius = self.size / 2.0 - self.stroke_width / 2.0;
            let painter = ui.painter();
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(self.stroke_width, c.surface_variant),
            );
            let (start_angle, sweep) = match self.value {
                Some(v) => (-std::f32::consts::FRAC_PI_2, v * std::f32::consts::TAU),
                None => {
                    let duration = 1.2;
                    let phase = ((self.id.value() % 1000) as f64 / 1000.0) * duration;
                    let t = (((ui.input(|i| i.time) + phase) % duration) / duration) as f32;
                    let start = t * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                    ui.ctx().request_repaint();
                    (start, std::f32::consts::PI * 1.5)
                }
            };
            let n = 64;
            let points: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let angle = start_angle + sweep * (i as f32 / n as f32);
                    Pos2::new(
                        center.x + radius * angle.cos(),
                        center.y + radius * angle.sin(),
                    )
                })
                .collect();
            if points.len() >= 2 {
                for i in 0..points.len() - 1 {
                    painter.line_segment(
                        [points[i], points[i + 1]],
                        Stroke::new(self.stroke_width, c.primary),
                    );
                }
            }
        }
        response
    }
}

pub struct M3Badge {
    count: Option<u32>,
    color: Option<Color32>,
}
impl M3Badge {
    pub fn dot() -> Self {
        Self {
            count: None,
            color: None,
        }
    }
    pub fn count(n: u32) -> Self {
        Self {
            count: Some(n),
            color: None,
        }
    }
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }
}
impl Widget for M3Badge {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let bg = self.color.unwrap_or(c.error);
        let fg = c.on_error;
        let size = match self.count {
            None => Vec2::splat(6.0),
            Some(n) => {
                let text = if n > 999 {
                    "999+".to_string()
                } else {
                    n.to_string()
                };
                let galley =
                    ui.painter()
                        .layout_no_wrap(text, egui::FontId::proportional(11.0), fg);
                Vec2::new((galley.size().x + 8.0).max(16.0), 16.0)
            }
        };
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let rounding = CornerRadius::same((size.y / 2.0) as u8);
            painter.rect_filled(rect, rounding, bg);
            if let Some(n) = self.count {
                let text = if n > 999 {
                    "999+".to_string()
                } else {
                    n.to_string()
                };
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(11.0),
                    fg,
                );
            }
        }
        response
    }
}

pub struct M3Divider {
    vertical: bool,
    inset: f32,
    thickness: f32,
}
impl M3Divider {
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            inset: 0.0,
            thickness: 1.0,
        }
    }
    pub fn vertical() -> Self {
        Self {
            vertical: true,
            inset: 0.0,
            thickness: 1.0,
        }
    }
    pub fn inset(mut self, i: f32) -> Self {
        self.inset = i;
        self
    }
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t;
        self
    }
}
impl Widget for M3Divider {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let (rect, response) = if self.vertical {
            ui.allocate_exact_size(
                Vec2::new(self.thickness, ui.available_height()),
                Sense::hover(),
            )
        } else {
            ui.allocate_exact_size(
                Vec2::new(ui.available_width(), self.thickness),
                Sense::hover(),
            )
        };
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            if self.vertical {
                let x = rect.center().x;
                painter.line_segment(
                    [
                        Pos2::new(x, rect.top() + self.inset),
                        Pos2::new(x, rect.bottom() - self.inset),
                    ],
                    Stroke::new(self.thickness, c.outline_variant),
                );
            } else {
                let y = rect.center().y;
                painter.line_segment(
                    [
                        Pos2::new(rect.left() + self.inset, y),
                        Pos2::new(rect.right() - self.inset, y),
                    ],
                    Stroke::new(self.thickness, c.outline_variant),
                );
            }
        }
        response
    }
}

pub struct M3Tooltip<'a> {
    text: &'a str,
}
impl<'a> M3Tooltip<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}
impl<'a> M3Tooltip<'a> {
    pub fn show_on_hover(self, ui: &Ui, response: &Response) {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let text = self.text;
        response.clone().on_hover_ui(|ui| {
            let frame = egui::Frame::NONE
                .fill(c.inverse_surface)
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(8, 4));
            frame.show(ui, |ui| {
                ui.label(RichText::new(text).color(c.inverse_on_surface).size(12.0));
            });
        });
    }
}
