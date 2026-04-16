use super::{M3Elevation, M3Theme};
use crate::style::with_alpha;
use egui::{
    CornerRadius, Frame, Id, Margin, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
    Widget,
};

// ─── M3Dialog ────────────────────────────────────────────────────────────────

pub struct M3Dialog<'a> {
    title: Option<&'a str>,
    body: Option<&'a str>,
    confirm_label: &'a str,
    cancel_label: Option<&'a str>,
    icon: Option<char>,
    open: &'a mut bool,
}

impl<'a> M3Dialog<'a> {
    pub fn new(open: &'a mut bool) -> Self {
        Self {
            title: None,
            body: None,
            confirm_label: "OK",
            cancel_label: None,
            icon: None,
            open,
        }
    }
    pub fn title(mut self, t: &'a str) -> Self {
        self.title = Some(t);
        self
    }
    pub fn body(mut self, b: &'a str) -> Self {
        self.body = Some(b);
        self
    }
    pub fn confirm(mut self, label: &'a str) -> Self {
        self.confirm_label = label;
        self
    }
    pub fn cancel(mut self, label: &'a str) -> Self {
        self.cancel_label = Some(label);
        self
    }
    pub fn icon(mut self, i: char) -> Self {
        self.icon = Some(i);
        self
    }

    /// Show the dialog. Returns true if the confirm button was clicked.
    pub fn show(self, ctx: &egui::Context) -> bool {
        if !*self.open {
            return false;
        }

        let theme = M3Theme::load(ctx);
        let c = &theme.colors;
        let mut confirmed = false;

        // Scrim overlay
        let screen = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::new("m3_dialog_scrim"),
        ));
        painter.rect_filled(screen, CornerRadius::ZERO, with_alpha(c.scrim, 0.32));

        // Dialog window
        egui::Window::new("__m3_dialog")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(312.0, 0.0))
            .frame(
                Frame::NONE
                    .fill(c.surface_container_high)
                    .corner_radius(CornerRadius::same(28u8))
                    .inner_margin(Margin::same(24_i8)),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Icon
                    if let Some(icon) = self.icon {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(icon.to_string())
                                .size(24.0)
                                .color(c.secondary),
                        );
                        ui.add_space(16.0);
                    }

                    // Title
                    if let Some(title) = self.title {
                        ui.label(RichText::new(title).size(24.0).color(c.on_surface));
                        ui.add_space(16.0);
                    }

                    // Body
                    if let Some(body) = self.body {
                        ui.label(RichText::new(body).size(14.0).color(c.on_surface_variant));
                        ui.add_space(24.0);
                    }

                    // Buttons (right-aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Confirm button
                        let confirm_resp = ui
                            .add(super::components::M3Button::new(self.confirm_label).text_only());
                        if confirm_resp.clicked() {
                            confirmed = true;
                            *self.open = false;
                        }

                        // Cancel button
                        if let Some(cancel_label) = self.cancel_label {
                            let cancel_resp =
                                ui.add(super::components::M3Button::new(cancel_label).text_only());
                            if cancel_resp.clicked() {
                                *self.open = false;
                            }
                        }
                    });
                });
            });

        confirmed
    }
}

// ─── M3Snackbar ─────────────────────────────────────────────────────────────

/// Snackbar state — store this in your app state.
#[derive(Clone, Default)]
pub struct M3SnackbarState {
    pub message: String,
    pub action_label: Option<String>,
    pub visible: bool,
    pub show_until: Option<f64>, // egui time
}

impl M3SnackbarState {
    pub fn show(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.action_label = None;
        self.visible = true;
        self.show_until = None; // will be set on first render
    }

    pub fn show_with_action(&mut self, message: impl Into<String>, action: impl Into<String>) {
        self.message = message.into();
        self.action_label = Some(action.into());
        self.visible = true;
        self.show_until = None;
    }

    pub fn dismiss(&mut self) {
        self.visible = false;
    }
}

pub struct M3Snackbar<'a> {
    state: &'a mut M3SnackbarState,
    duration_secs: f64,
}

impl<'a> M3Snackbar<'a> {
    pub fn new(state: &'a mut M3SnackbarState) -> Self {
        Self {
            state,
            duration_secs: 4.0,
        }
    }
    pub fn duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Render the snackbar. Call this every frame.
    /// Returns true if the action button was clicked.
    pub fn show(self, ctx: &egui::Context) -> bool {
        if !self.state.visible {
            return false;
        }

        let now = ctx.input(|i| i.time);

        // Set expiry on first show
        if self.state.show_until.is_none() {
            self.state.show_until = Some(now + self.duration_secs);
        }

        // Auto-dismiss
        if let Some(until) = self.state.show_until {
            if now >= until {
                self.state.visible = false;
                return false;
            }
        }

        let theme = M3Theme::load(ctx);
        let c = &theme.colors;
        let mut action_clicked = false;

        let screen = ctx.screen_rect();
        let snack_w = (screen.width() - 32.0).min(600.0);
        let snack_h = 48.0_f32;
        let snack_rect = Rect::from_min_size(
            Pos2::new(
                screen.center().x - snack_w / 2.0,
                screen.bottom() - snack_h - 16.0,
            ),
            Vec2::new(snack_w, snack_h),
        );

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            Id::new("m3_snackbar"),
        ));
        painter.rect_filled(snack_rect, CornerRadius::same(4u8), c.inverse_surface);

        // Message
        painter.text(
            Pos2::new(snack_rect.left() + 16.0, snack_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &self.state.message,
            egui::FontId::proportional(14.0),
            c.inverse_on_surface,
        );

        // Action button
        if let Some(action) = &self.state.action_label {
            let action_x = snack_rect.right() - 16.0;
            let action_rect = Rect::from_center_size(
                Pos2::new(action_x - 30.0, snack_rect.center().y),
                Vec2::new(60.0, 36.0),
            );
            painter.text(
                action_rect.center(),
                egui::Align2::CENTER_CENTER,
                action.as_str(),
                egui::FontId::proportional(14.0),
                c.inverse_primary,
            );

            // Click detection via Area
            let action_response = ctx.input(|i| {
                i.pointer.button_clicked(egui::PointerButton::Primary)
                    && i.pointer
                        .interact_pos()
                        .map(|p| action_rect.contains(p))
                        .unwrap_or(false)
            });
            if action_response {
                action_clicked = true;
                self.state.visible = false;
            }
        }

        ctx.request_repaint();
        action_clicked
    }
}

// ─── M3FAB ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug)]
pub enum M3FabSize {
    Small, // 40px
    #[default]
    Regular, // 56px
    Large, // 96px
}

pub struct M3Fab<'a> {
    icon: char,
    label: Option<&'a str>, // Some = Extended FAB
    size: M3FabSize,
    color_variant: M3FabColor,
}

#[derive(Clone, Copy, Default, Debug)]
pub enum M3FabColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Surface,
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
            M3FabSize::Small => 40.0_f32,
            M3FabSize::Regular => 56.0,
            M3FabSize::Large => 96.0,
        };

        let icon_size = match self.size {
            M3FabSize::Small => 18.0_f32,
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
                // Extended: icon + label
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
                // Regular: icon only
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

// ─── M3DropdownMenu ──────────────────────────────────────────────────────────

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

            // Outlined container
            painter.rect_stroke(
                rect,
                CornerRadius::same(4u8),
                Stroke::new(1.0, c.outline),
                egui::StrokeKind::Outside,
            );

            // Label (floating, always small since there's a value)
            painter.text(
                Pos2::new(rect.left() + 16.0, rect.top() + 10.0),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(12.0),
                c.on_surface_variant,
            );

            // Selected value
            painter.text(
                Pos2::new(rect.left() + 16.0, rect.center().y + 6.0),
                egui::Align2::LEFT_CENTER,
                current,
                egui::FontId::proportional(16.0),
                c.on_surface,
            );

            // Dropdown arrow
            painter.text(
                Pos2::new(rect.right() - 24.0, rect.center().y),
                egui::Align2::CENTER_CENTER,
                "▾",
                egui::FontId::proportional(16.0),
                c.on_surface_variant,
            );
        }

        // Popup menu
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

            // Close on click outside
            if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                let click_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(pos) = click_pos {
                    if !popup_rect.contains(pos) && !rect.contains(pos) {
                        ui.ctx().data_mut(|d| d.insert_temp(open_id, false));
                    }
                }
            }
        }

        response
    }
}
