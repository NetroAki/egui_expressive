use crate::m3::M3Theme;
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Ui, Vec2};

type HeaderFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

fn nav_item_colors(theme: &M3Theme, selected: bool) -> (Color32, Color32) {
    let c = &theme.colors;
    if selected {
        (c.on_secondary_container, c.on_surface)
    } else {
        (c.on_surface_variant, c.on_surface_variant)
    }
}

fn navigation_bar_pill_rect(item_rect: Rect) -> Rect {
    Rect::from_center_size(
        Pos2::new(item_rect.center().x, item_rect.top() + 16.0),
        Vec2::new(64.0, 32.0),
    )
}

fn navigation_rail_pill_rect(item_rect: Rect) -> Rect {
    Rect::from_center_size(
        Pos2::new(item_rect.center().x, item_rect.top() + 20.0),
        Vec2::new(56.0, 32.0),
    )
}

fn nav_badge_text(count: u32) -> String {
    if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    }
}

pub struct M3NavItem<'a> {
    pub label: &'a str,
    pub icon: char,
    pub badge: Option<u32>,
}

impl<'a> M3NavItem<'a> {
    pub fn new(label: &'a str, icon: char) -> Self {
        Self {
            label,
            icon,
            badge: None,
        }
    }
    pub fn badge(mut self, n: u32) -> Self {
        self.badge = Some(n);
        self
    }
}

pub struct M3NavigationBar<'a> {
    items: Vec<M3NavItem<'a>>,
    selected: &'a mut usize,
    height: f32,
}

impl<'a> M3NavigationBar<'a> {
    pub fn new(selected: &'a mut usize) -> Self {
        Self {
            items: Vec::new(),
            selected,
            height: 80.0,
        }
    }
    pub fn item(mut self, item: M3NavItem<'a>) -> Self {
        self.items.push(item);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let n = self.items.len().max(1);
        let item_w = ui.available_width() / n as f32;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), self.height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::ZERO, c.surface_container);
            for (i, item) in self.items.iter().enumerate() {
                let item_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + i as f32 * item_w, rect.top()),
                    Vec2::new(item_w, self.height),
                );
                let is_selected = *self.selected == i;
                let (icon_color, label_color) = nav_item_colors(&theme, is_selected);
                if is_selected {
                    let pill = navigation_bar_pill_rect(item_rect);
                    painter.rect_filled(pill, CornerRadius::same(16u8), c.secondary_container);
                }
                painter.text(
                    Pos2::new(item_rect.center().x, item_rect.top() + 16.0),
                    egui::Align2::CENTER_CENTER,
                    item.icon.to_string(),
                    egui::FontId::proportional(24.0),
                    icon_color,
                );
                painter.text(
                    Pos2::new(item_rect.center().x, item_rect.top() + 40.0),
                    egui::Align2::CENTER_CENTER,
                    item.label,
                    egui::FontId::proportional(12.0),
                    label_color,
                );
                if let Some(count) = item.badge {
                    let badge_text = nav_badge_text(count);
                    let badge_x = item_rect.center().x + 12.0;
                    let badge_y = item_rect.top() + 4.0;
                    painter.circle_filled(Pos2::new(badge_x, badge_y), 8.0, c.error);
                    painter.text(
                        Pos2::new(badge_x, badge_y),
                        egui::Align2::CENTER_CENTER,
                        badge_text,
                        egui::FontId::proportional(9.0),
                        c.on_error,
                    );
                }
                let item_response = ui.interact(item_rect, Id::new(("m3_nav", i)), Sense::click());
                if item_response.clicked() {
                    *self.selected = i;
                }
            }
        }
        response
    }
}

pub struct M3NavigationRail<'a> {
    items: Vec<M3NavItem<'a>>,
    selected: &'a mut usize,
    width: f32,
    header: Option<HeaderFn<'a>>,
}

impl<'a> M3NavigationRail<'a> {
    pub fn new(selected: &'a mut usize) -> Self {
        Self {
            items: Vec::new(),
            selected,
            width: 80.0,
            header: None,
        }
    }
    pub fn item(mut self, item: M3NavItem<'a>) -> Self {
        self.items.push(item);
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    pub fn header(mut self, f: impl FnOnce(&mut Ui) + 'a) -> Self {
        self.header = Some(Box::new(f));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let height = ui.available_height();
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(self.width, height), Sense::hover());
        let mut y = rect.top() + 8.0;
        if let Some(header_fn) = self.header {
            let header_rect =
                Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(self.width, 56.0));
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect));
            header_fn(&mut child_ui);
            y += 56.0 + 8.0;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::ZERO, c.surface_container);
            for (i, item) in self.items.iter().enumerate() {
                let item_h = 72.0_f32;
                let item_rect =
                    Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(self.width, item_h));
                let is_selected = *self.selected == i;
                let (icon_color, label_color) = nav_item_colors(&theme, is_selected);
                if is_selected {
                    let pill = navigation_rail_pill_rect(item_rect);
                    painter.rect_filled(pill, CornerRadius::same(16u8), c.secondary_container);
                }
                painter.text(
                    Pos2::new(item_rect.center().x, item_rect.top() + 20.0),
                    egui::Align2::CENTER_CENTER,
                    item.icon.to_string(),
                    egui::FontId::proportional(24.0),
                    icon_color,
                );
                painter.text(
                    Pos2::new(item_rect.center().x, item_rect.top() + 48.0),
                    egui::Align2::CENTER_CENTER,
                    item.label,
                    egui::FontId::proportional(12.0),
                    label_color,
                );
                let item_response = ui.interact(item_rect, Id::new(("m3_rail", i)), Sense::click());
                if item_response.clicked() {
                    *self.selected = i;
                }
                y += item_h;
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase8_navigation_item_visuals_are_token_deterministic() {
        let theme = M3Theme::light();
        let (selected_icon, selected_label) = nav_item_colors(&theme, true);
        assert_eq!(selected_icon, theme.colors.on_secondary_container);
        assert_eq!(selected_label, theme.colors.on_surface);

        let (plain_icon, plain_label) = nav_item_colors(&theme, false);
        assert_eq!(plain_icon, theme.colors.on_surface_variant);
        assert_eq!(plain_label, theme.colors.on_surface_variant);
    }

    #[test]
    fn phase8_navigation_pills_and_badges_are_deterministic() {
        let item_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(80.0, 72.0));
        let bar_pill = navigation_bar_pill_rect(item_rect);
        assert_eq!(bar_pill.size(), Vec2::new(64.0, 32.0));
        assert_eq!(bar_pill.center(), Pos2::new(50.0, 36.0));

        let rail_pill = navigation_rail_pill_rect(item_rect);
        assert_eq!(rail_pill.size(), Vec2::new(56.0, 32.0));
        assert_eq!(rail_pill.center(), Pos2::new(50.0, 40.0));

        assert_eq!(nav_badge_text(7), "7");
        assert_eq!(nav_badge_text(100), "99+");
    }
}
