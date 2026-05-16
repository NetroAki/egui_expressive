use crate::m3::M3Theme;
use crate::style::with_alpha;
use egui::{Color32, CornerRadius, Pos2, Response, Sense, Ui, Vec2, Widget};

fn list_item_line_count(supporting: bool, trailing_supporting: bool) -> usize {
    if supporting {
        if trailing_supporting {
            3
        } else {
            2
        }
    } else {
        1
    }
}

fn list_item_height(lines: usize) -> f32 {
    match lines {
        1 => 56.0,
        2 => 72.0,
        _ => 88.0,
    }
}

fn list_item_selected_overlay(theme: &M3Theme) -> Color32 {
    with_alpha(theme.colors.primary, 0.12)
}

fn list_item_text_colors(theme: &M3Theme) -> (Color32, Color32) {
    (theme.colors.on_surface, theme.colors.on_surface_variant)
}

pub struct M3ListItem<'a> {
    headline: &'a str,
    supporting: Option<&'a str>,
    trailing_supporting: Option<&'a str>,
    leading_icon: Option<char>,
    trailing_icon: Option<char>,
    selected: bool,
}

impl<'a> M3ListItem<'a> {
    pub fn new(headline: &'a str) -> Self {
        Self {
            headline,
            supporting: None,
            trailing_supporting: None,
            leading_icon: None,
            trailing_icon: None,
            selected: false,
        }
    }
    pub fn supporting(mut self, s: &'a str) -> Self {
        self.supporting = Some(s);
        self
    }
    pub fn trailing_supporting(mut self, s: &'a str) -> Self {
        self.trailing_supporting = Some(s);
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
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
}

impl Widget for M3ListItem<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let lines = list_item_line_count(
            self.supporting.is_some(),
            self.trailing_supporting.is_some(),
        );
        let height = list_item_height(lines);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            if self.selected {
                painter.rect_filled(rect, CornerRadius::ZERO, list_item_selected_overlay(&theme));
            }
            let (headline_color, supporting_color) = list_item_text_colors(&theme);
            if response.hovered() {
                painter.rect_filled(rect, CornerRadius::ZERO, with_alpha(c.on_surface, 0.08));
            }
            let mut x = rect.left() + 16.0;
            if let Some(icon) = self.leading_icon {
                painter.text(
                    Pos2::new(x + 12.0, rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(24.0),
                    c.on_surface_variant,
                );
                x += 56.0;
            }
            let headline_y = if lines == 1 {
                rect.center().y
            } else {
                rect.top() + 20.0
            };
            painter.text(
                Pos2::new(x, headline_y),
                egui::Align2::LEFT_CENTER,
                self.headline,
                egui::FontId::proportional(16.0),
                headline_color,
            );
            if let Some(sup) = self.supporting {
                painter.text(
                    Pos2::new(x, rect.top() + 40.0),
                    egui::Align2::LEFT_CENTER,
                    sup,
                    egui::FontId::proportional(14.0),
                    supporting_color,
                );
            }
            if let Some(trail_sup) = self.trailing_supporting {
                painter.text(
                    Pos2::new(x, rect.top() + 60.0),
                    egui::Align2::LEFT_CENTER,
                    trail_sup,
                    egui::FontId::proportional(12.0),
                    supporting_color,
                );
            }
            if let Some(trail_icon) = self.trailing_icon {
                painter.text(
                    Pos2::new(rect.right() - 28.0, rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    trail_icon.to_string(),
                    egui::FontId::proportional(24.0),
                    c.on_surface_variant,
                );
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase8_list_item_heights_map_to_line_count() {
        assert_eq!(list_item_line_count(false, false), 1);
        assert_eq!(list_item_line_count(true, false), 2);
        assert_eq!(list_item_line_count(true, true), 3);
        assert_eq!(list_item_height(1), 56.0);
        assert_eq!(list_item_height(2), 72.0);
        assert_eq!(list_item_height(3), 88.0);
    }

    #[test]
    fn phase8_list_item_selected_and_text_colors_are_token_deterministic() {
        let theme = M3Theme::light();
        let overlay = list_item_selected_overlay(&theme);
        assert_eq!(overlay, with_alpha(theme.colors.primary, 0.12));
        assert_eq!(overlay.a(), 30);

        let (headline, supporting) = list_item_text_colors(&theme);
        assert_eq!(headline, theme.colors.on_surface);
        assert_eq!(supporting, theme.colors.on_surface_variant);
    }
}
