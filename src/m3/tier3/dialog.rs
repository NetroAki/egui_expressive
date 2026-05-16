use crate::m3::{M3Button, M3Theme};
use crate::style::with_alpha;
use egui::{CornerRadius, Frame, Id, Margin, Pos2, Rect, RichText, Vec2};

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

    pub fn show(self, ctx: &egui::Context) -> bool {
        if !*self.open {
            return false;
        }

        let theme = M3Theme::load(ctx);
        let c = &theme.colors;
        let mut confirmed = false;

        let screen = ctx.viewport_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::new("m3_dialog_scrim"),
        ));
        painter.rect_filled(screen, CornerRadius::ZERO, with_alpha(c.scrim, 0.32));

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
                    if let Some(icon) = self.icon {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(icon.to_string())
                                .size(24.0)
                                .color(c.secondary),
                        );
                        ui.add_space(16.0);
                    }
                    if let Some(title) = self.title {
                        ui.label(RichText::new(title).size(24.0).color(c.on_surface));
                        ui.add_space(16.0);
                    }
                    if let Some(body) = self.body {
                        ui.label(RichText::new(body).size(14.0).color(c.on_surface_variant));
                        ui.add_space(24.0);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let confirm_resp = ui.add(M3Button::new(self.confirm_label).text_only());
                        if confirm_resp.clicked() {
                            confirmed = true;
                            *self.open = false;
                        }
                        if let Some(cancel_label) = self.cancel_label {
                            let cancel_resp = ui.add(M3Button::new(cancel_label).text_only());
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

#[derive(Clone, Default)]
pub struct M3SnackbarState {
    pub message: String,
    pub action_label: Option<String>,
    pub visible: bool,
    pub show_until: Option<f64>,
}

impl M3SnackbarState {
    pub fn show(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.action_label = None;
        self.visible = true;
        self.show_until = None;
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
}
impl<'a> M3Snackbar<'a> {
    pub fn show(self, ctx: &egui::Context) -> bool {
        if !self.state.visible {
            return false;
        }
        let now = ctx.input(|i| i.time);
        if self.state.show_until.is_none() {
            self.state.show_until = Some(now + self.duration_secs);
        }
        if let Some(until) = self.state.show_until {
            if now >= until {
                self.state.visible = false;
                return false;
            }
        }

        let theme = M3Theme::load(ctx);
        let c = &theme.colors;
        let mut action_clicked = false;
        let screen = ctx.viewport_rect();
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

        painter.text(
            Pos2::new(snack_rect.left() + 16.0, snack_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &self.state.message,
            egui::FontId::proportional(14.0),
            c.inverse_on_surface,
        );

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
