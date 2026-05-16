use crate::m3::M3Theme;
use crate::style::with_alpha;
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

fn switch_visuals(theme: &M3Theme, on: bool) -> (Color32, Stroke, Color32, f32) {
    let c = &theme.colors;
    if on {
        (c.primary, Stroke::NONE, c.on_primary, 12.0)
    } else {
        (
            c.surface_variant,
            Stroke::new(2.0, c.outline),
            c.outline,
            12.0 * 0.67,
        )
    }
}

fn checkbox_visuals(theme: &M3Theme, checked: bool) -> (Color32, Color32, Stroke) {
    let c = &theme.colors;
    if checked {
        (c.primary, c.on_primary, Stroke::NONE)
    } else {
        (
            Color32::TRANSPARENT,
            c.on_surface_variant,
            Stroke::new(2.0, c.on_surface_variant),
        )
    }
}

fn radio_visuals(theme: &M3Theme, selected: bool) -> (Color32, Color32) {
    let c = &theme.colors;
    if selected {
        (c.primary, c.primary)
    } else {
        (c.on_surface_variant, Color32::TRANSPARENT)
    }
}

fn chip_visuals(theme: &M3Theme, selected: bool) -> (Color32, Color32, Stroke) {
    let c = &theme.colors;
    if selected {
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
    }
}

fn slider_visuals(
    theme: &M3Theme,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
) -> (Color32, Color32, Color32, f32) {
    let c = &theme.colors;
    let start = *range.start();
    let end = *range.end();
    let denom = (end - start).max(f32::EPSILON);
    let t = ((value - start) / denom).clamp(0.0, 1.0);
    (c.surface_variant, c.primary, c.on_primary, t)
}

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
        let (rect, response) = ui.allocate_exact_size(Vec2::new(52.0, 32.0), Sense::click());
        if response.clicked() {
            *self.value = !*self.value;
        }
        if ui.is_rect_visible(rect) {
            let on = *self.value;
            let t = ui.ctx().animate_bool_with_time(self.id, on, 0.15);
            let (track_color, track_border, thumb_color, thumb_size) = switch_visuals(&theme, on);
            let painter = ui.painter();
            let rounding = CornerRadius::same(16);
            painter.rect_filled(rect, rounding, track_color);
            if track_border.width > 0.0 {
                painter.rect_stroke(rect, rounding, track_border, egui::StrokeKind::Outside);
            }
            let thumb_x = rect.left() + 16.0 + t * (52.0 - 32.0);
            let thumb_center = Pos2::new(thumb_x, rect.center().y);
            painter.circle_filled(thumb_center, thumb_size, thumb_color);
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
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
        if response.clicked() {
            *self.value = !*self.value;
        }
        if ui.is_rect_visible(rect) {
            let checked = *self.value || self.indeterminate;
            let (bg, fg, border) = checkbox_visuals(&theme, checked);
            let painter = ui.painter();
            let rounding = CornerRadius::same(2);
            painter.rect_filled(rect, rounding, bg);
            if border.width > 0.0 {
                painter.rect_stroke(rect, rounding, border, egui::StrokeKind::Outside);
            }
            if checked {
                if self.indeterminate {
                    let y = rect.center().y;
                    painter.line_segment(
                        [
                            Pos2::new(rect.left() + 4.0, y),
                            Pos2::new(rect.right() - 4.0, y),
                        ],
                        Stroke::new(2.0, fg),
                    );
                } else {
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
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
        if ui.is_rect_visible(rect) {
            let center = rect.center();
            let painter = ui.painter();
            let (ring_color, dot_color) = radio_visuals(&theme, self.selected);
            painter.circle_stroke(center, 9.0, Stroke::new(2.0, ring_color));
            if self.selected {
                painter.circle_filled(center, 5.0, dot_color);
            }
            if response.hovered() {
                painter.circle_filled(center, 14.0, with_alpha(ring_color, 0.08));
            }
        }
        response
    }
}

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
        let (bg, fg, border) = chip_visuals(&theme, self.selected);
        let text_galley = ui.painter().layout_no_wrap(
            self.label.to_string(),
            egui::FontId::proportional(14.0),
            fg,
        );
        let icon_w = if self.icon.is_some() { 26.0 } else { 0.0 };
        let trail_w = if self.trailing_icon.is_some() {
            26.0
        } else {
            0.0
        };
        let total_w = 24.0 + icon_w + text_galley.size().x + trail_w + 24.0;
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(total_w, 32.0),
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let rounding = CornerRadius::same(8);
            painter.rect_filled(rect, rounding, bg);
            if border.width > 0.0 {
                painter.rect_stroke(rect, rounding, border, egui::StrokeKind::Outside);
            }
            if response.hovered() && self.enabled {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.08));
            }
            let mut x = rect.left() + 12.0;
            let cy = rect.center().y;
            if let Some(icon) = self.icon {
                painter.text(
                    Pos2::new(x + 9.0, cy),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(18.0),
                    fg,
                );
                x += 26.0;
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
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::drag());
        let track_h = 4.0_f32;
        let thumb_r = 10.0_f32;
        if response.dragged() {
            let delta = response.drag_delta().x;
            let range_size = *self.range.end() - *self.range.start();
            let track_width = (rect.width() - thumb_r * 2.0).max(1.0);
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
            let (_, _, _, t) = slider_visuals(&theme, *self.value, self.range.clone());
            let track_y = rect.center().y;
            let track_rect = Rect::from_min_size(
                Pos2::new(rect.left() + thumb_r, track_y - track_h / 2.0),
                Vec2::new(rect.width() - thumb_r * 2.0, track_h),
            );
            let thumb_x = track_rect.left() + t * track_rect.width();
            let thumb_center = Pos2::new(thumb_x, track_y);
            let painter = ui.painter();
            let rounding = CornerRadius::same(2);
            painter.rect_filled(track_rect, rounding, c.surface_variant);
            painter.rect_filled(
                Rect::from_min_max(track_rect.min, Pos2::new(thumb_x, track_rect.max.y)),
                rounding,
                c.primary,
            );
            painter.circle_filled(thumb_center, thumb_r, c.primary);
            if response.hovered() || response.dragged() {
                painter.circle_filled(thumb_center, thumb_r + 6.0, with_alpha(c.primary, 0.12));
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase8_input_control_visuals_are_token_deterministic() {
        let theme = M3Theme::light();
        let (track_on, border_on, thumb_on, thumb_on_size) = switch_visuals(&theme, true);
        assert_eq!(track_on, theme.colors.primary);
        assert_eq!(border_on, Stroke::NONE);
        assert_eq!(thumb_on, theme.colors.on_primary);
        assert_eq!(thumb_on_size, 12.0);

        let (track_off, border_off, thumb_off, thumb_off_size) = switch_visuals(&theme, false);
        assert_eq!(track_off, theme.colors.surface_variant);
        assert_eq!(border_off.width, 2.0);
        assert_eq!(border_off.color, theme.colors.outline);
        assert_eq!(thumb_off, theme.colors.outline);
        assert!((thumb_off_size - 8.04).abs() < f32::EPSILON);

        let (checkbox_bg, checkbox_fg, checkbox_border) = checkbox_visuals(&theme, true);
        assert_eq!(checkbox_bg, theme.colors.primary);
        assert_eq!(checkbox_fg, theme.colors.on_primary);
        assert_eq!(checkbox_border, Stroke::NONE);

        let (radio_ring, radio_dot) = radio_visuals(&theme, true);
        assert_eq!(radio_ring, theme.colors.primary);
        assert_eq!(radio_dot, theme.colors.primary);
    }

    #[test]
    fn phase8_chip_and_slider_endpoints_are_token_deterministic() {
        let theme = M3Theme::light();
        let (selected_bg, selected_fg, selected_border) = chip_visuals(&theme, true);
        assert_eq!(selected_bg, theme.colors.secondary_container);
        assert_eq!(selected_fg, theme.colors.on_secondary_container);
        assert_eq!(selected_border, Stroke::NONE);

        let (plain_bg, plain_fg, plain_border) = chip_visuals(&theme, false);
        assert_eq!(plain_bg, Color32::TRANSPARENT);
        assert_eq!(plain_fg, theme.colors.on_surface_variant);
        assert_eq!(plain_border.width, 1.0);
        assert_eq!(plain_border.color, theme.colors.outline);

        let (inactive, active, tick, t) = slider_visuals(&theme, 25.0, 0.0..=100.0);
        assert_eq!(inactive, theme.colors.surface_variant);
        assert_eq!(active, theme.colors.primary);
        assert_eq!(tick, theme.colors.on_primary);
        assert_eq!(t, 0.25);
    }
}
