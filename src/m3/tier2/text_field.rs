use crate::m3::M3Theme;
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

#[derive(Clone, Copy, Default, Debug)]
pub enum M3TextFieldVariant {
    #[default]
    Filled,
    Outlined,
}

fn text_field_active_color(theme: &M3Theme, error: bool) -> Color32 {
    if error {
        theme.colors.error
    } else {
        theme.colors.primary
    }
}

fn text_field_border(theme: &M3Theme, focused: bool, error: bool) -> Stroke {
    let color = if focused {
        text_field_active_color(theme, error)
    } else {
        theme.colors.outline
    };
    Stroke::new(if focused { 2.0 } else { 1.0 }, color)
}

fn text_field_label_layout(field_rect: Rect, leading_icon: bool, float_t: f32) -> (Pos2, f32) {
    let label_size_large = 16.0_f32;
    let label_size_small = 12.0_f32;
    let label_size = label_size_large + float_t * (label_size_small - label_size_large);
    let label_y_bottom = field_rect.top() + 20.0;
    let label_y_top = field_rect.top() + 8.0;
    let label_y = label_y_bottom + float_t * (label_y_top - label_y_bottom);
    let label_x = field_rect.left() + if leading_icon { 52.0 } else { 16.0 };
    (Pos2::new(label_x, label_y), label_size)
}

fn text_field_text_rect(field_rect: Rect, leading_icon: bool, trailing_icon: bool) -> Rect {
    let leading_pad = if leading_icon { 52.0 } else { 16.0 };
    let trailing_pad = if trailing_icon { 48.0 } else { 16.0 };
    Rect::from_min_size(
        Pos2::new(field_rect.left() + leading_pad, field_rect.top() + 24.0),
        Vec2::new(field_rect.width() - leading_pad - trailing_pad, 24.0),
    )
}

pub struct M3TextField<'a> {
    value: &'a mut String,
    label: &'a str,
    hint: Option<&'a str>,
    variant: M3TextFieldVariant,
    id: Id,
    password: bool,
    enabled: bool,
    error: Option<&'a str>,
    leading_icon: Option<char>,
    trailing_icon: Option<char>,
}

impl<'a> M3TextField<'a> {
    pub fn new(id: impl std::hash::Hash, label: &'a str, value: &'a mut String) -> Self {
        Self {
            value,
            label,
            hint: None,
            variant: M3TextFieldVariant::Filled,
            id: Id::new(id),
            password: false,
            enabled: true,
            error: None,
            leading_icon: None,
            trailing_icon: None,
        }
    }
    pub fn outlined(mut self) -> Self {
        self.variant = M3TextFieldVariant::Outlined;
        self
    }
    pub fn hint(mut self, h: &'a str) -> Self {
        self.hint = Some(h);
        self
    }
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
    pub fn error(mut self, e: &'a str) -> Self {
        self.error = Some(e);
        self
    }
    pub fn leading_icon(mut self, i: char) -> Self {
        self.leading_icon = Some(i);
        self
    }
    pub fn trailing_icon(mut self, i: char) -> Self {
        self.trailing_icon = Some(i);
        self
    }
}

impl Widget for M3TextField<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let height = 56.0_f32;
        let width = ui.available_width();
        let has_value = !self.value.is_empty();
        let focused_id = self.id.with("focused");
        let is_focused = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(focused_id).unwrap_or(false));
        let float_t = ui
            .ctx()
            .animate_bool_with_time(self.id, has_value || is_focused, 0.15);
        let extra_h = if self.error.is_some() { 20.0 } else { 0.0 };
        let (rect, outer_response) =
            ui.allocate_exact_size(Vec2::new(width, height + extra_h), Sense::hover());
        let field_rect = Rect::from_min_size(rect.min, Vec2::new(width, height));

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let active_color = text_field_active_color(&theme, self.error.is_some());
            let border = text_field_border(&theme, is_focused, self.error.is_some());
            match self.variant {
                M3TextFieldVariant::Filled => {
                    let bg = c.surface_container_highest;
                    let rounding = CornerRadius {
                        nw: 4,
                        ne: 4,
                        sw: 0,
                        se: 0,
                    };
                    painter.rect_filled(field_rect, rounding, bg);
                    painter.line_segment(
                        [
                            Pos2::new(field_rect.left(), field_rect.bottom()),
                            Pos2::new(field_rect.right(), field_rect.bottom()),
                        ],
                        border,
                    );
                }
                M3TextFieldVariant::Outlined => {
                    painter.rect_stroke(
                        field_rect,
                        CornerRadius::same(4u8),
                        border,
                        egui::StrokeKind::Outside,
                    );
                }
            }

            let label_color = if is_focused {
                active_color
            } else {
                c.on_surface_variant
            };
            let (label_pos, label_size) =
                text_field_label_layout(field_rect, self.leading_icon.is_some(), float_t);
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(label_size),
                label_color,
            );

            if let Some(icon) = self.leading_icon {
                painter.text(
                    Pos2::new(field_rect.left() + 24.0, field_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(24.0),
                    c.on_surface_variant,
                );
            }
            if let Some(icon) = self.trailing_icon {
                painter.text(
                    Pos2::new(field_rect.right() - 24.0, field_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(24.0),
                    c.on_surface_variant,
                );
            }
            if let Some(err) = self.error {
                painter.text(
                    Pos2::new(field_rect.left() + 16.0, field_rect.bottom() + 4.0),
                    egui::Align2::LEFT_TOP,
                    err,
                    egui::FontId::proportional(12.0),
                    c.error,
                );
            }
        }

        let text_rect = text_field_text_rect(
            field_rect,
            self.leading_icon.is_some(),
            self.trailing_icon.is_some(),
        );
        let mut text_edit = egui::TextEdit::singleline(self.value)
            .frame(egui::Frame::NONE)
            .desired_width(text_rect.width())
            .text_color(c.on_surface);
        if self.password {
            text_edit = text_edit.password(true);
        }
        let text_response = ui.put(text_rect, text_edit);
        ui.ctx()
            .data_mut(|d| d.insert_temp(focused_id, text_response.has_focus()));
        outer_response | text_response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase8_text_field_visuals_are_token_deterministic() {
        let theme = M3Theme::light();
        assert_eq!(text_field_active_color(&theme, false), theme.colors.primary);
        assert_eq!(text_field_active_color(&theme, true), theme.colors.error);

        let focused = text_field_border(&theme, true, false);
        assert_eq!(focused.width, 2.0);
        assert_eq!(focused.color, theme.colors.primary);

        let error = text_field_border(&theme, true, true);
        assert_eq!(error.width, 2.0);
        assert_eq!(error.color, theme.colors.error);

        let idle = text_field_border(&theme, false, true);
        assert_eq!(idle.width, 1.0);
        assert_eq!(idle.color, theme.colors.outline);
    }

    #[test]
    fn phase8_text_field_label_and_padding_endpoints_are_deterministic() {
        let field_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(200.0, 56.0));
        let (resting_pos, resting_size) = text_field_label_layout(field_rect, false, 0.0);
        assert_eq!(resting_pos, Pos2::new(26.0, 40.0));
        assert_eq!(resting_size, 16.0);

        let (floating_pos, floating_size) = text_field_label_layout(field_rect, true, 1.0);
        assert_eq!(floating_pos, Pos2::new(62.0, 28.0));
        assert_eq!(floating_size, 12.0);

        let text_rect = text_field_text_rect(field_rect, true, true);
        assert_eq!(text_rect.left(), 62.0);
        assert_eq!(text_rect.top(), 44.0);
        assert_eq!(text_rect.width(), 100.0);
    }
}
