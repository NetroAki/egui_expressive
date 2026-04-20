use super::{M3Elevation, M3Theme};
use crate::style::with_alpha;
use egui::{CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Boxed header closure for [`M3NavigationRail`].
type HeaderFn<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;

// ─── M3TextField ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug)]
pub enum M3TextFieldVariant {
    #[default]
    Filled,
    Outlined,
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

        // Animate label float
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
            let active_color = if self.error.is_some() {
                c.error
            } else {
                c.primary
            };
            let border_color = if is_focused { active_color } else { c.outline };

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
                    // Bottom border
                    let border_w = if is_focused { 2.0 } else { 1.0 };
                    painter.line_segment(
                        [
                            Pos2::new(field_rect.left(), field_rect.bottom()),
                            Pos2::new(field_rect.right(), field_rect.bottom()),
                        ],
                        Stroke::new(border_w, border_color),
                    );
                }
                M3TextFieldVariant::Outlined => {
                    let border_w = if is_focused { 2.0 } else { 1.0 };
                    painter.rect_stroke(
                        field_rect,
                        CornerRadius::same(4u8),
                        Stroke::new(border_w, border_color),
                        egui::StrokeKind::Outside,
                    );
                }
            }

            // Floating label
            let label_size_large = 16.0_f32;
            let label_size_small = 12.0_f32;
            let label_size = label_size_large + float_t * (label_size_small - label_size_large);
            let label_y_bottom = field_rect.top() + 20.0;
            let label_y_top = field_rect.top() + 8.0;
            let label_y = label_y_bottom + float_t * (label_y_top - label_y_bottom);
            let label_color = if is_focused {
                active_color
            } else {
                c.on_surface_variant
            };
            let label_x = field_rect.left()
                + if self.leading_icon.is_some() {
                    52.0
                } else {
                    16.0
                };

            painter.text(
                Pos2::new(label_x, label_y),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(label_size),
                label_color,
            );

            // Leading icon
            if let Some(icon) = self.leading_icon {
                painter.text(
                    Pos2::new(field_rect.left() + 12.0 + 12.0, field_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(24.0),
                    c.on_surface_variant,
                );
            }

            // Trailing icon
            if let Some(icon) = self.trailing_icon {
                painter.text(
                    Pos2::new(field_rect.right() - 12.0 - 12.0, field_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(24.0),
                    c.on_surface_variant,
                );
            }

            // Error text
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

        // Actual text input — positioned inside the field
        let leading_pad = if self.leading_icon.is_some() {
            52.0
        } else {
            16.0
        };
        let trailing_pad = if self.trailing_icon.is_some() {
            48.0
        } else {
            16.0
        };
        let text_rect = Rect::from_min_size(
            Pos2::new(field_rect.left() + leading_pad, field_rect.top() + 24.0),
            Vec2::new(field_rect.width() - leading_pad - trailing_pad, 24.0),
        );

        let mut text_edit = egui::TextEdit::singleline(self.value)
            .frame(egui::Frame::NONE)
            .desired_width(text_rect.width())
            .text_color(c.on_surface);

        if self.password {
            text_edit = text_edit.password(true);
        }

        let text_response = ui.put(text_rect, text_edit);

        // Track focus
        let now_focused = text_response.has_focus();
        ui.ctx()
            .data_mut(|d| d.insert_temp(focused_id, now_focused));

        outer_response | text_response
    }
}

// ─── M3NavigationBar ─────────────────────────────────────────────────────────

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
                let icon_color = if is_selected {
                    c.on_secondary_container
                } else {
                    c.on_surface_variant
                };
                let label_color = if is_selected {
                    c.on_surface
                } else {
                    c.on_surface_variant
                };

                // Indicator pill for selected item
                if is_selected {
                    let pill = Rect::from_center_size(
                        Pos2::new(item_rect.center().x, item_rect.top() + 16.0),
                        Vec2::new(64.0, 32.0),
                    );
                    painter.rect_filled(pill, CornerRadius::same(16u8), c.secondary_container);
                }

                // Icon
                painter.text(
                    Pos2::new(item_rect.center().x, item_rect.top() + 16.0),
                    egui::Align2::CENTER_CENTER,
                    item.icon.to_string(),
                    egui::FontId::proportional(24.0),
                    icon_color,
                );

                // Label
                painter.text(
                    Pos2::new(item_rect.center().x, item_rect.top() + 40.0),
                    egui::Align2::CENTER_CENTER,
                    item.label,
                    egui::FontId::proportional(12.0),
                    label_color,
                );

                // Badge
                if let Some(count) = item.badge {
                    let badge_text = if count > 99 {
                        "99+".to_string()
                    } else {
                        count.to_string()
                    };
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

                // Click detection
                let item_response = ui.interact(item_rect, Id::new(("m3_nav", i)), Sense::click());
                if item_response.clicked() {
                    *self.selected = i;
                }
            }
        }

        response
    }
}

// ─── M3NavigationRail ─────────────────────────────────────────────────────────

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

        // Handle header first (needs mutable access to ui for new_child)
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
                let icon_color = if is_selected {
                    c.on_secondary_container
                } else {
                    c.on_surface_variant
                };
                let label_color = if is_selected {
                    c.on_surface
                } else {
                    c.on_surface_variant
                };

                if is_selected {
                    let pill = Rect::from_center_size(
                        Pos2::new(item_rect.center().x, item_rect.top() + 20.0),
                        Vec2::new(56.0, 32.0),
                    );
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

// ─── M3TopAppBar ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug)]
pub enum M3TopAppBarVariant {
    #[default]
    Small,
    CenterAligned,
    Medium,
    Large,
}

pub struct M3TopAppBar<'a> {
    title: &'a str,
    variant: M3TopAppBarVariant,
    navigation_icon: Option<char>,
    actions: Vec<(char, &'a str)>, // (icon, tooltip)
    scrolled: bool,
}

impl<'a> M3TopAppBar<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            variant: M3TopAppBarVariant::Small,
            navigation_icon: None,
            actions: Vec::new(),
            scrolled: false,
        }
    }
    pub fn center_aligned(mut self) -> Self {
        self.variant = M3TopAppBarVariant::CenterAligned;
        self
    }
    pub fn medium(mut self) -> Self {
        self.variant = M3TopAppBarVariant::Medium;
        self
    }
    pub fn large(mut self) -> Self {
        self.variant = M3TopAppBarVariant::Large;
        self
    }
    pub fn navigation_icon(mut self, icon: char) -> Self {
        self.navigation_icon = Some(icon);
        self
    }
    pub fn action(mut self, icon: char, tooltip: &'a str) -> Self {
        self.actions.push((icon, tooltip));
        self
    }
    pub fn scrolled(mut self, s: bool) -> Self {
        self.scrolled = s;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;

        let height = match self.variant {
            M3TopAppBarVariant::Small | M3TopAppBarVariant::CenterAligned => 64.0_f32,
            M3TopAppBarVariant::Medium => 112.0,
            M3TopAppBarVariant::Large => 152.0,
        };

        let bg = if self.scrolled {
            M3Elevation::Level2.surface_tint(c.surface, c.primary)
        } else {
            c.surface
        };

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::ZERO, bg);

            let icon_size = 24.0_f32;
            let icon_pad = 12.0_f32;
            let mut x = rect.left() + icon_pad;

            // Navigation icon
            if let Some(nav_icon) = self.navigation_icon {
                painter.text(
                    Pos2::new(x + icon_size / 2.0, rect.top() + 32.0),
                    egui::Align2::CENTER_CENTER,
                    nav_icon.to_string(),
                    egui::FontId::proportional(icon_size),
                    c.on_surface,
                );
                x += icon_size + icon_pad;
            }

            // Title
            let title_font = match self.variant {
                M3TopAppBarVariant::Small | M3TopAppBarVariant::CenterAligned => {
                    egui::FontId::proportional(22.0)
                }
                M3TopAppBarVariant::Medium => egui::FontId::proportional(24.0),
                M3TopAppBarVariant::Large => egui::FontId::proportional(28.0),
            };

            let title_y = match self.variant {
                M3TopAppBarVariant::Small | M3TopAppBarVariant::CenterAligned => rect.top() + 32.0,
                M3TopAppBarVariant::Medium => rect.bottom() - 24.0,
                M3TopAppBarVariant::Large => rect.bottom() - 28.0,
            };

            let title_pos = match self.variant {
                M3TopAppBarVariant::CenterAligned => Pos2::new(rect.center().x, title_y),
                _ => Pos2::new(x, title_y),
            };

            let title_align = match self.variant {
                M3TopAppBarVariant::CenterAligned => egui::Align2::CENTER_CENTER,
                _ => egui::Align2::LEFT_CENTER,
            };

            painter.text(title_pos, title_align, self.title, title_font, c.on_surface);

            // Action icons (right side)
            let mut ax = rect.right() - icon_pad;
            for (icon, _tooltip) in self.actions.iter().rev() {
                ax -= icon_size;
                painter.text(
                    Pos2::new(ax + icon_size / 2.0, rect.top() + 32.0),
                    egui::Align2::CENTER_CENTER,
                    icon.to_string(),
                    egui::FontId::proportional(icon_size),
                    c.on_surface_variant,
                );
                ax -= icon_pad;
            }
        }

        response
    }
}

// ─── M3ListItem ───────────────────────────────────────────────────────────────

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

        let lines = if self.supporting.is_some() {
            if self.trailing_supporting.is_some() {
                3
            } else {
                2
            }
        } else {
            1
        };
        let height = match lines {
            1 => 56.0_f32,
            2 => 72.0,
            _ => 88.0,
        };

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            if self.selected {
                painter.rect_filled(rect, CornerRadius::ZERO, with_alpha(c.primary, 0.12));
            }
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
                x += 40.0 + 16.0;
            }

            let text_x = x;
            let headline_y = if lines == 1 {
                rect.center().y
            } else {
                rect.top() + 20.0
            };

            painter.text(
                Pos2::new(text_x, headline_y),
                egui::Align2::LEFT_CENTER,
                self.headline,
                egui::FontId::proportional(16.0),
                c.on_surface,
            );

            if let Some(sup) = self.supporting {
                painter.text(
                    Pos2::new(text_x, rect.top() + 40.0),
                    egui::Align2::LEFT_CENTER,
                    sup,
                    egui::FontId::proportional(14.0),
                    c.on_surface_variant,
                );
            }

            if let Some(trail_sup) = self.trailing_supporting {
                painter.text(
                    Pos2::new(text_x, rect.top() + 60.0),
                    egui::Align2::LEFT_CENTER,
                    trail_sup,
                    egui::FontId::proportional(12.0),
                    c.on_surface_variant,
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
