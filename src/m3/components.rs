use super::{M3Elevation, M3Theme};
use crate::style::with_alpha;
use egui::{
    Color32, CornerRadius, Frame, Id, Margin, Pos2, Rect, Response, RichText, Sense, Stroke, Ui,
    Vec2, Widget,
};

// ─── M3Button ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum M3ButtonVariant {
    #[default]
    Filled,
    Tonal,
    Outlined,
    Text,
    Elevated,
}

pub struct M3Button<'a> {
    text: &'a str,
    variant: M3ButtonVariant,
    icon: Option<char>,
    enabled: bool,
    width: Option<f32>,
}

impl<'a> M3Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            variant: M3ButtonVariant::Filled,
            icon: None,
            enabled: true,
            width: None,
        }
    }
    pub fn tonal(mut self) -> Self {
        self.variant = M3ButtonVariant::Tonal;
        self
    }
    pub fn outlined(mut self) -> Self {
        self.variant = M3ButtonVariant::Outlined;
        self
    }
    pub fn text_only(mut self) -> Self {
        self.variant = M3ButtonVariant::Text;
        self
    }
    pub fn elevated(mut self) -> Self {
        self.variant = M3ButtonVariant::Elevated;
        self
    }
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
}

impl Widget for M3Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;

        let (bg, fg, stroke) = match self.variant {
            M3ButtonVariant::Filled => (c.primary, c.on_primary, Stroke::NONE),
            M3ButtonVariant::Tonal => (
                c.secondary_container,
                c.on_secondary_container,
                Stroke::NONE,
            ),
            M3ButtonVariant::Outlined => {
                (Color32::TRANSPARENT, c.primary, Stroke::new(1.0, c.outline))
            }
            M3ButtonVariant::Text => (Color32::TRANSPARENT, c.primary, Stroke::NONE),
            M3ButtonVariant::Elevated => {
                let bg = M3Elevation::Level1.surface_tint(c.surface_container_low, c.primary);
                (bg, c.primary, Stroke::NONE)
            }
        };

        let disabled_alpha = if self.enabled { 1.0 } else { 0.38 };
        let bg = with_alpha(bg, if self.enabled { 1.0 } else { 0.12 });
        let fg = with_alpha(fg, disabled_alpha);

        let desired_size = Vec2::new(self.width.unwrap_or(0.0), 40.0);

        let (rect, response) = ui.allocate_at_least(
            Vec2::new(desired_size.x.max(64.0), 40.0),
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if ui.is_rect_visible(rect) {
            let rounding = CornerRadius::same(20); // M3: full pill shape
            let painter = ui.painter();

            // Background
            painter.rect_filled(rect, rounding, bg);

            // Border (outlined variant)
            if stroke.width > 0.0 {
                painter.rect_stroke(rect, rounding, stroke, egui::StrokeKind::Outside);
            }

            // State layer
            if response.hovered() && self.enabled {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.08));
            }
            if response.is_pointer_button_down_on() && self.enabled {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.12));
            }

            // Content
            let content_rect = rect.shrink2(Vec2::new(24.0, 0.0));
            let center = content_rect.center();

            if let Some(icon) = self.icon {
                // Icon + text
                let icon_str = icon.to_string();
                let text_galley = ui.painter().layout_no_wrap(
                    self.text.to_string(),
                    egui::FontId::proportional(14.0),
                    fg,
                );
                let total_w = 18.0 + 8.0 + text_galley.size().x;
                let start_x = center.x - total_w / 2.0;
                painter.text(
                    Pos2::new(start_x + 9.0, center.y),
                    egui::Align2::CENTER_CENTER,
                    icon_str,
                    egui::FontId::proportional(18.0),
                    fg,
                );
                painter.text(
                    Pos2::new(start_x + 18.0 + 8.0 + text_galley.size().x / 2.0, center.y),
                    egui::Align2::CENTER_CENTER,
                    self.text,
                    egui::FontId::proportional(14.0),
                    fg,
                );
            } else {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    self.text,
                    egui::FontId::proportional(14.0),
                    fg,
                );
            }
        }

        response
    }
}

// ─── M3Card ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug)]
pub enum M3CardVariant {
    #[default]
    Elevated,
    Filled,
    Outlined,
}

pub struct M3Card {
    variant: M3CardVariant,
    padding: f32,
    width: Option<f32>,
}

impl Default for M3Card {
    fn default() -> Self {
        Self::new()
    }
}

impl M3Card {
    pub fn new() -> Self {
        Self {
            variant: M3CardVariant::Elevated,
            padding: 16.0,
            width: None,
        }
    }
    pub fn filled(mut self) -> Self {
        self.variant = M3CardVariant::Filled;
        self
    }
    pub fn outlined(mut self) -> Self {
        self.variant = M3CardVariant::Outlined;
        self
    }
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;

        let (bg, stroke) = match self.variant {
            M3CardVariant::Elevated => {
                let bg = M3Elevation::Level1.surface_tint(c.surface_container_low, c.primary);
                (bg, Stroke::NONE)
            }
            M3CardVariant::Filled => (c.surface_container_highest, Stroke::NONE),
            M3CardVariant::Outlined => (c.surface, Stroke::new(1.0, c.outline_variant)),
        };

        let frame = Frame::NONE
            .fill(bg)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(self.padding.clamp(0.0, 127.0).round() as i8));

        let resp = frame.show(ui, |ui| {
            if let Some(w) = self.width {
                ui.set_width(w);
            }
            content(ui);
        });

        resp.response
    }
}

// ─── M3Switch ───────────────────────────────────────────────────────────────

pub struct M3Switch<'a> {
    value: &'a mut bool,
    id: Id,
}

impl<'a> M3Switch<'a> {
    pub fn new(id: impl std::hash::Hash, value: &'a mut bool) -> Self {
        Self {
            value,
            id: Id::new(id),
        }
    }
}

impl Widget for M3Switch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;

        let track_w = 52.0_f32;
        let track_h = 32.0_f32;
        let thumb_r = 12.0_f32;

        let (rect, response) = ui.allocate_exact_size(Vec2::new(track_w, track_h), Sense::click());

        if response.clicked() {
            *self.value = !*self.value;
        }

        if ui.is_rect_visible(rect) {
            let on = *self.value;
            // Animate thumb position
            let t = ui.ctx().animate_bool_with_time(self.id, on, 0.15);

            let track_color = if on { c.primary } else { c.surface_variant };
            let track_border = if on {
                Stroke::NONE
            } else {
                Stroke::new(2.0, c.outline)
            };
            let thumb_color = if on { c.on_primary } else { c.outline };
            let thumb_size = if on { thumb_r } else { thumb_r * 0.67 };

            let painter = ui.painter();
            let rounding = CornerRadius::same((track_h / 2.0) as u8);

            // Track
            painter.rect_filled(rect, rounding, track_color);
            if track_border.width > 0.0 {
                painter.rect_stroke(rect, rounding, track_border, egui::StrokeKind::Outside);
            }

            // Thumb
            let thumb_x = rect.left() + track_h / 2.0 + t * (track_w - track_h);
            let thumb_center = Pos2::new(thumb_x, rect.center().y);
            painter.circle_filled(thumb_center, thumb_size, thumb_color);

            // Hover state layer
            if response.hovered() {
                painter.circle_filled(
                    thumb_center,
                    thumb_size + 8.0,
                    with_alpha(thumb_color, 0.08),
                );
            }
        }

        response
    }
}

// ─── M3Checkbox ─────────────────────────────────────────────────────────────

pub struct M3Checkbox<'a> {
    value: &'a mut bool,
    indeterminate: bool,
}

impl<'a> M3Checkbox<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self {
            value,
            indeterminate: false,
        }
    }
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }
}

impl Widget for M3Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let size = 18.0_f32;

        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
        if response.clicked() {
            *self.value = !*self.value;
        }

        if ui.is_rect_visible(rect) {
            let checked = *self.value || self.indeterminate;
            let (bg, fg, border) = if checked {
                (c.primary, c.on_primary, Stroke::NONE)
            } else {
                (
                    Color32::TRANSPARENT,
                    c.on_surface_variant,
                    Stroke::new(2.0, c.on_surface_variant),
                )
            };

            let rounding = CornerRadius::same(2);
            let painter = ui.painter();
            painter.rect_filled(rect, rounding, bg);
            if border.width > 0.0 {
                painter.rect_stroke(rect, rounding, border, egui::StrokeKind::Outside);
            }

            if checked {
                if self.indeterminate {
                    // Dash
                    let y = rect.center().y;
                    painter.line_segment(
                        [
                            Pos2::new(rect.left() + 4.0, y),
                            Pos2::new(rect.right() - 4.0, y),
                        ],
                        Stroke::new(2.0, fg),
                    );
                } else {
                    // Checkmark
                    let p1 = Pos2::new(rect.left() + 3.5, rect.center().y);
                    let p2 = Pos2::new(rect.left() + 7.0, rect.bottom() - 4.0);
                    let p3 = Pos2::new(rect.right() - 3.0, rect.top() + 4.0);
                    painter.line_segment([p1, p2], Stroke::new(2.0, fg));
                    painter.line_segment([p2, p3], Stroke::new(2.0, fg));
                }
            }

            if response.hovered() {
                painter.rect_filled(
                    rect.expand(4.0),
                    CornerRadius::same(6),
                    with_alpha(c.on_surface, 0.08),
                );
            }
        }

        response
    }
}

// ─── M3RadioButton ──────────────────────────────────────────────────────────

pub struct M3RadioButton {
    selected: bool,
}

impl M3RadioButton {
    pub fn new(selected: bool) -> Self {
        Self { selected }
    }
}

impl Widget for M3RadioButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let size = 20.0_f32;

        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

        if ui.is_rect_visible(rect) {
            let center = rect.center();
            let painter = ui.painter();
            let (ring_color, dot_color) = if self.selected {
                (c.primary, c.primary)
            } else {
                (c.on_surface_variant, Color32::TRANSPARENT)
            };

            painter.circle_stroke(center, size / 2.0 - 1.0, Stroke::new(2.0, ring_color));
            if self.selected {
                painter.circle_filled(center, size / 4.0, dot_color);
            }
            if response.hovered() {
                painter.circle_filled(center, size / 2.0 + 4.0, with_alpha(ring_color, 0.08));
            }
        }

        response
    }
}

// ─── M3Chip ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug)]
pub enum M3ChipVariant {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

pub struct M3Chip<'a> {
    label: &'a str,
    variant: M3ChipVariant,
    selected: bool,
    icon: Option<char>,
    trailing_icon: Option<char>,
    enabled: bool,
}

impl<'a> M3Chip<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: M3ChipVariant::Assist,
            selected: false,
            icon: None,
            trailing_icon: None,
            enabled: true,
        }
    }
    pub fn filter(mut self) -> Self {
        self.variant = M3ChipVariant::Filter;
        self
    }
    pub fn input(mut self) -> Self {
        self.variant = M3ChipVariant::Input;
        self
    }
    pub fn suggestion(mut self) -> Self {
        self.variant = M3ChipVariant::Suggestion;
        self
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    pub fn icon(mut self, i: char) -> Self {
        self.icon = Some(i);
        self
    }
    pub fn trailing_icon(mut self, i: char) -> Self {
        self.trailing_icon = Some(i);
        self
    }
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
}

impl Widget for M3Chip<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;

        let (bg, fg, border) = if self.selected {
            (
                c.secondary_container,
                c.on_secondary_container,
                Stroke::NONE,
            )
        } else {
            (
                Color32::TRANSPARENT,
                c.on_surface_variant,
                Stroke::new(1.0, c.outline),
            )
        };

        let height = 32.0_f32;
        let h_pad = 12.0_f32;

        // Measure text width
        let text_galley = ui.painter().layout_no_wrap(
            self.label.to_string(),
            egui::FontId::proportional(14.0),
            fg,
        );
        let icon_w = if self.icon.is_some() { 18.0 + 8.0 } else { 0.0 };
        let trail_w = if self.trailing_icon.is_some() {
            8.0 + 18.0
        } else {
            0.0
        };
        let total_w = h_pad + icon_w + text_galley.size().x + trail_w + h_pad;

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(total_w, height),
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if ui.is_rect_visible(rect) {
            let rounding = CornerRadius::same(8);
            let painter = ui.painter();
            painter.rect_filled(rect, rounding, bg);
            if border.width > 0.0 {
                painter.rect_stroke(rect, rounding, border, egui::StrokeKind::Outside);
            }
            if response.hovered() && self.enabled {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.08));
            }

            let mut x = rect.left() + h_pad;
            let cy = rect.center().y;

            if let Some(icon) = self.icon {
                painter.text(
                    Pos2::new(x + 9.0, cy),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(18.0),
                    fg,
                );
                x += 18.0 + 8.0;
            }

            painter.text(
                Pos2::new(x + text_galley.size().x / 2.0, cy),
                egui::Align2::CENTER_CENTER,
                self.label,
                egui::FontId::proportional(14.0),
                fg,
            );
            x += text_galley.size().x;

            if let Some(trail) = self.trailing_icon {
                x += 8.0;
                painter.text(
                    Pos2::new(x + 9.0, cy),
                    egui::Align2::CENTER_CENTER,
                    trail.to_string(),
                    egui::FontId::proportional(18.0),
                    fg,
                );
            }
        }

        response
    }
}

// ─── M3LinearProgress ────────────────────────────────────────────────────────

pub struct M3LinearProgress {
    value: Option<f32>, // None = indeterminate
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

            // Track
            painter.rect_filled(rect, rounding, c.surface_variant);

            match self.value {
                Some(v) => {
                    // Determinate
                    let fill_w = rect.width() * v;
                    let fill_rect = Rect::from_min_size(rect.min, Vec2::new(fill_w, self.height));
                    painter.rect_filled(fill_rect, rounding, c.primary);
                }
                None => {
                    // Indeterminate — animated sliding bar
                    let duration = 1.5;
                    let phase = ((self.id.value() % 1000) as f64 / 1000.0) * duration;
                    let t = (((ui.input(|i| i.time) + phase) % duration) / duration) as f32;
                    let bar_w = rect.width() * 0.4;
                    let x = rect.left() + (rect.width() + bar_w) * t - bar_w;
                    let x0 = x.max(rect.left());
                    let x1 = (x + bar_w).min(rect.right());
                    if x1 > x0 {
                        let fill_rect = Rect::from_min_max(
                            Pos2::new(x0, rect.top()),
                            Pos2::new(x1, rect.top() + self.height),
                        );
                        painter.rect_filled(fill_rect, rounding, c.primary);
                    }
                    ui.ctx().request_repaint();
                }
            }
        }

        response
    }
}

// ─── M3CircularProgress ──────────────────────────────────────────────────────

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

            // Track
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

            // Arc
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

// ─── M3Badge ────────────────────────────────────────────────────────────────

pub struct M3Badge {
    count: Option<u32>,
    color: Option<Color32>,
}

impl M3Badge {
    /// Small dot badge (no count)
    pub fn dot() -> Self {
        Self {
            count: None,
            color: None,
        }
    }
    /// Badge with count
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

// ─── M3Slider ───────────────────────────────────────────────────────────────

pub struct M3Slider<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    steps: Option<u32>,
    show_value: bool,
}

impl<'a> M3Slider<'a> {
    pub fn new(value: &'a mut f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            steps: None,
            show_value: false,
        }
    }
    pub fn steps(mut self, n: u32) -> Self {
        self.steps = Some(n);
        self
    }
    pub fn show_value(mut self, s: bool) -> Self {
        self.show_value = s;
        self
    }
}

impl Widget for M3Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;

        let track_h = 4.0_f32;
        let thumb_r = 10.0_f32;
        let height = thumb_r * 2.0 + 8.0;
        let width = ui.available_width();

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::drag());

        if response.dragged() {
            let delta = response.drag_delta().x;
            let range_size = *self.range.end() - *self.range.start();
            let track_width = (width - thumb_r * 2.0).max(1.0);
            *self.value = (*self.value + delta / track_width * range_size)
                .clamp(*self.range.start(), *self.range.end());

            if let Some(steps) = self.steps {
                if steps > 0 {
                    let step = range_size / steps as f32;
                    let v = *self.value - *self.range.start();
                    *self.value = (v / step).round() * step + *self.range.start();
                    *self.value = self.value.clamp(*self.range.start(), *self.range.end());
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let t = (*self.value - *self.range.start()) / (*self.range.end() - *self.range.start());
            let track_y = rect.center().y;
            let track_rect = Rect::from_min_size(
                Pos2::new(rect.left() + thumb_r, track_y - track_h / 2.0),
                Vec2::new(rect.width() - thumb_r * 2.0, track_h),
            );
            let thumb_x = track_rect.left() + t * track_rect.width();
            let thumb_center = Pos2::new(thumb_x, track_y);

            let painter = ui.painter();
            let rounding = CornerRadius::same(2);

            // Inactive track
            painter.rect_filled(track_rect, rounding, c.surface_variant);

            // Active track
            let active_rect =
                Rect::from_min_max(track_rect.min, Pos2::new(thumb_x, track_rect.max.y));
            painter.rect_filled(active_rect, rounding, c.primary);

            // Thumb
            painter.circle_filled(thumb_center, thumb_r, c.primary);

            // Hover state
            if response.hovered() || response.dragged() {
                painter.circle_filled(thumb_center, thumb_r + 6.0, with_alpha(c.primary, 0.12));
            }

            // Tick marks
            if let Some(steps) = self.steps {
                if steps > 0 {
                    for i in 0..=steps {
                        let tx = track_rect.left() + (i as f32 / steps as f32) * track_rect.width();
                        let tick_color = if tx <= thumb_x {
                            c.on_primary
                        } else {
                            c.on_surface_variant
                        };
                        painter.circle_filled(Pos2::new(tx, track_y), 2.0, tick_color);
                    }
                }
            }
        }

        response
    }
}

// ─── M3Divider ──────────────────────────────────────────────────────────────

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

// ─── M3Tooltip ───────────────────────────────────────────────────────────────

pub struct M3Tooltip<'a> {
    text: &'a str,
}

impl<'a> M3Tooltip<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    /// Show tooltip on hover of the given response.
    pub fn show_on_hover(self, ui: &Ui, response: &Response) {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let text = self.text;
        response.clone().on_hover_ui(|ui| {
            let frame = Frame::NONE
                .fill(c.inverse_surface)
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(8, 4));
            frame.show(ui, |ui| {
                ui.label(RichText::new(text).color(c.inverse_on_surface).size(12.0));
            });
        });
    }
}
