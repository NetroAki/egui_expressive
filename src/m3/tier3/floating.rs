use crate::m3::{M3Elevation, M3Theme};
use crate::style::with_alpha;
use egui::{CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

#[derive(Clone, Copy, Default, Debug)]
pub enum M3FabSize {
    Small,
    #[default]
    Regular,
    Large,
}

#[derive(Clone, Copy, Default, Debug)]
pub enum M3FabColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Surface,
}

pub struct M3Fab<'a> {
    icon: char,
    label: Option<&'a str>,
    size: M3FabSize,
    color_variant: M3FabColor,
}

impl<'a> M3Fab<'a> {
    pub fn new(icon: char) -> Self {
        Self {
            icon,
            label: None,
            size: M3FabSize::Regular,
            color_variant: M3FabColor::Primary,
        }
    }
    pub fn extended(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
    pub fn small(mut self) -> Self {
        self.size = M3FabSize::Small;
        self
    }
    pub fn large(mut self) -> Self {
        self.size = M3FabSize::Large;
        self
    }
    pub fn secondary(mut self) -> Self {
        self.color_variant = M3FabColor::Secondary;
        self
    }
    pub fn tertiary(mut self) -> Self {
        self.color_variant = M3FabColor::Tertiary;
        self
    }
    pub fn surface(mut self) -> Self {
        self.color_variant = M3FabColor::Surface;
        self
    }
}

impl Widget for M3Fab<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let (bg, fg) = match self.color_variant {
            M3FabColor::Primary => (c.primary_container, c.on_primary_container),
            M3FabColor::Secondary => (c.secondary_container, c.on_secondary_container),
            M3FabColor::Tertiary => (c.tertiary_container, c.on_tertiary_container),
            M3FabColor::Surface => (
                M3Elevation::Level3.surface_tint(c.surface_container_low, c.primary),
                c.primary,
            ),
        };

        let size_px = match self.size {
            M3FabSize::Small => 40.0,
            M3FabSize::Regular => 56.0,
            M3FabSize::Large => 96.0,
        };
        let icon_size = match self.size {
            M3FabSize::Small => 18.0,
            M3FabSize::Regular => 24.0,
            M3FabSize::Large => 36.0,
        };
        let rounding = match self.size {
            M3FabSize::Small => CornerRadius::same(12u8),
            M3FabSize::Regular => CornerRadius::same(16u8),
            M3FabSize::Large => CornerRadius::same(28u8),
        };
        let width = if let Some(label) = self.label {
            let galley = ui.painter().layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(14.0),
                fg,
            );
            16.0 + icon_size + 8.0 + galley.size().x + 20.0
        } else {
            size_px
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, size_px), Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, rounding, bg);
            if response.hovered() {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.08));
            }
            if response.is_pointer_button_down_on() {
                painter.rect_filled(rect, rounding, with_alpha(fg, 0.12));
            }

            if let Some(label) = self.label {
                let icon_x = rect.left() + 16.0 + icon_size / 2.0;
                painter.text(
                    Pos2::new(icon_x, rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    self.icon.to_string(),
                    egui::FontId::proportional(icon_size),
                    fg,
                );
                painter.text(
                    Pos2::new(icon_x + icon_size / 2.0 + 8.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(14.0),
                    fg,
                );
            } else {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    self.icon.to_string(),
                    egui::FontId::proportional(icon_size),
                    fg,
                );
            }
        }

        response
    }
}

pub struct M3DropdownMenu<'a> {
    selected: &'a mut usize,
    items: Vec<&'a str>,
    label: &'a str,
    id: Id,
}

impl<'a> M3DropdownMenu<'a> {
    pub fn new(id: impl std::hash::Hash, label: &'a str, selected: &'a mut usize) -> Self {
        Self {
            selected,
            items: Vec::new(),
            label,
            id: Id::new(id),
        }
    }
    pub fn item(mut self, item: &'a str) -> Self {
        self.items.push(item);
        self
    }
    pub fn items(mut self, items: Vec<&'a str>) -> Self {
        self.items = items;
        self
    }
}

impl Widget for M3DropdownMenu<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = M3Theme::load(ui.ctx());
        let c = &theme.colors;
        let height = 56.0_f32;
        let width = ui.available_width();
        let current = self.items.get(*self.selected).copied().unwrap_or("");
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_stroke(
                rect,
                CornerRadius::same(4u8),
                Stroke::new(1.0, c.outline),
                egui::StrokeKind::Outside,
            );
            painter.text(
                Pos2::new(rect.left() + 16.0, rect.top() + 10.0),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(12.0),
                c.on_surface_variant,
            );
            painter.text(
                Pos2::new(rect.left() + 16.0, rect.center().y + 6.0),
                egui::Align2::LEFT_CENTER,
                current,
                egui::FontId::proportional(16.0),
                c.on_surface,
            );
            painter.text(
                Pos2::new(rect.right() - 24.0, rect.center().y),
                egui::Align2::CENTER_CENTER,
                "▾",
                egui::FontId::proportional(16.0),
                c.on_surface_variant,
            );
        }

        let open_id = self.id.with("open");
        let is_open = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(open_id).unwrap_or(false));
        if response.clicked() {
            ui.ctx().data_mut(|d| d.insert_temp(open_id, !is_open));
        }

        if is_open {
            let popup_rect = Rect::from_min_size(
                Pos2::new(rect.left(), rect.bottom()),
                Vec2::new(rect.width(), (self.items.len() as f32 * 48.0).min(256.0)),
            );
            let popup_painter = ui.ctx().layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                self.id.with("popup"),
            ));
            popup_painter.rect_filled(
                popup_rect,
                CornerRadius::same(4u8),
                M3Elevation::Level2.surface_tint(
                    M3Theme::load(ui.ctx()).colors.surface_container,
                    M3Theme::load(ui.ctx()).colors.primary,
                ),
            );

            for (i, item) in self.items.iter().enumerate() {
                let item_rect = Rect::from_min_size(
                    Pos2::new(popup_rect.left(), popup_rect.top() + i as f32 * 48.0),
                    Vec2::new(popup_rect.width(), 48.0),
                );
                let theme = M3Theme::load(ui.ctx());
                let c = &theme.colors;
                let is_selected = *self.selected == i;
                if is_selected {
                    popup_painter.rect_filled(
                        item_rect,
                        CornerRadius::ZERO,
                        with_alpha(c.primary, 0.12),
                    );
                }
                popup_painter.text(
                    Pos2::new(item_rect.left() + 16.0, item_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    *item,
                    egui::FontId::proportional(16.0),
                    c.on_surface,
                );
                let item_response = ui.interact(item_rect, self.id.with(i), Sense::click());
                if item_response.clicked() {
                    *self.selected = i;
                    ui.ctx().data_mut(|d| d.insert_temp(open_id, false));
                }
            }

            if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if !popup_rect.contains(pos) && !rect.contains(pos) {
                        ui.ctx().data_mut(|d| d.insert_temp(open_id, false));
                    }
                }
            }
        }

        response
    }
}
