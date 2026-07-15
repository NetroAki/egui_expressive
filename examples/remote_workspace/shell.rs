use eframe::egui;
use egui_expressive::{InteractiveFill, PaintedIconButton, PaintedIconButtonStyle};

use super::app::RemoteWorkspaceShowcase;
use super::tokens::WorkspaceTokens;
use super::widgets::{nav_button, status_meter, NavIcon, StatusKind};

impl RemoteWorkspaceShowcase {
    pub(super) fn show_navigation(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            egui::Frame::NONE
                .fill(self.tokens.panel_raised)
                .stroke(egui::Stroke::new(1.0, self.tokens.border_strong))
                .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("⌘")
                            .strong()
                            .size(19.0)
                            .color(self.tokens.mint),
                    );
                });
            ui.add_space(22.0);
            for (index, icon, label) in [
                (0, NavIcon::Vault, "Vaults"),
                (1, NavIcon::Key, "Keys"),
                (2, NavIcon::Transfer, "Transfers"),
                (3, NavIcon::Settings, "Settings"),
            ] {
                if nav_button(ui, icon, label, self.selected_nav == index, self.tokens).clicked() {
                    self.selected_nav = index;
                }
                ui.add_space(7.0);
            }
        });
    }

    pub(super) fn show_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (index, label) in ["Vaults", "SFTP", "Split view"].into_iter().enumerate() {
                let selected = self.selected_tab == index;
                let response = egui::Frame::NONE
                    .fill(if selected {
                        WorkspaceTokens::mix(self.tokens.panel_raised, self.tokens.mint, 0.075)
                    } else {
                        self.tokens.panel
                    })
                    .stroke(egui::Stroke::new(
                        1.0,
                        if selected {
                            WorkspaceTokens::mix(self.tokens.border_strong, self.tokens.mint, 0.38)
                        } else {
                            self.tokens.panel
                        },
                    ))
                    .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
                    .inner_margin(egui::Margin::symmetric(12, 7))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if selected {
                                status_meter(ui, StatusKind::Live, self.tokens);
                            }
                            ui.label(egui::RichText::new(label).strong().color(if selected {
                                self.tokens.mint
                            } else {
                                self.tokens.muted
                            }));
                        });
                    })
                    .response
                    .interact(egui::Sense::click());
                if response.clicked() {
                    self.selected_tab = index;
                }
            }
            let style = PaintedIconButtonStyle {
                size: egui::vec2(32.0, 32.0),
                fill: InteractiveFill {
                    idle: egui::Color32::TRANSPARENT,
                    hovered: self.tokens.panel_raised,
                    pressed: self.tokens.field,
                },
                stroke: egui::Stroke::NONE,
                focus_stroke: egui::Stroke::new(2.0, self.tokens.mint),
                corner_radius: WorkspaceTokens::RADIUS_MEDIUM as f32,
                icon_color: self.tokens.muted,
            };
            ui.add(PaintedIconButton::new(
                style,
                |painter: &egui::Painter, rect: egui::Rect, color: egui::Color32| {
                    let stroke = egui::Stroke::new(1.7, color);
                    painter.line_segment(
                        [
                            rect.center() + egui::vec2(-5.0, 0.0),
                            rect.center() + egui::vec2(5.0, 0.0),
                        ],
                        stroke,
                    );
                    painter.line_segment(
                        [
                            rect.center() + egui::vec2(0.0, -5.0),
                            rect.center() + egui::vec2(0.0, 5.0),
                        ],
                        stroke,
                    );
                },
            ))
            .on_hover_text("New workspace tab");
        });
    }
}
