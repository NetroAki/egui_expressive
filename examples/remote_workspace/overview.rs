use eframe::egui;
use egui_expressive::Tw;

use super::app::RemoteWorkspaceShowcase;
use super::tokens::WorkspaceTokens;
use super::widgets::{
    color_swatch, section_title, status_label, status_meter, surface, StatusKind,
};

impl RemoteWorkspaceShowcase {
    pub(super) fn show_intro(&mut self, ui: &mut egui::Ui) {
        let gap = 24.0;
        let available = ui.available_width();
        let left_width = (available - gap) * 0.55;
        let right_width = available - gap - left_width;
        let hero_height = 472.0;
        let inner_height = 436.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, hero_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_width(left_width);
                    surface(ui, self.tokens, |ui| {
                        ui.set_min_width(left_width - 36.0);
                        ui.set_min_height(inner_height);
                        self.show_intro_copy(ui);
                    });
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(right_width, hero_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_width(right_width);
                    surface(ui, self.tokens, |ui| {
                        ui.set_min_width(right_width - 36.0);
                        ui.set_min_height(inner_height);
                        self.show_terminal(ui);
                    });
                },
            );
        });
    }

    fn show_intro_copy(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            status_label(ui, StatusKind::Live, "Reference dark mode", self.tokens);
            status_label(ui, StatusKind::Info, "Desktop workspace", self.tokens);
            status_label(
                ui,
                StatusKind::Default,
                "SSH · SFTP · RDP · VNC",
                self.tokens,
            );
        });
        ui.add_space(20.0);
        ui.add(
            egui::Label::new(
                egui::RichText::new("Tokens and components for a focused remote workspace.")
                    .size(42.0)
                    .strong()
                    .color(self.tokens.text),
            )
            .wrap(),
        );
        ui.add_space(14.0);
        ui.add(
            egui::Label::new(
                egui::RichText::new("The visual system follows the supplied references: deep navy application chrome, muted slate controls, pastel semantic accents, compact host rows, and high-density panes built for sustained technical work.")
                    .size(15.0)
                    .color(self.tokens.muted),
            )
            .wrap(),
        );
    }

    fn show_terminal(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(self.tokens.panel)
            .stroke(egui::Stroke::new(1.0, self.tokens.border))
            .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    session_tab(ui, "acme-api-dev-us-west", true, self.tokens);
                    session_tab(ui, "Windows Server", false, self.tokens);
                });
            });
        ui.add_space(7.0);
        egui::Frame::NONE
            .fill(self.tokens.field)
            .stroke(egui::Stroke::new(1.0, self.tokens.border))
            .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
            .show(ui, |ui| {
                Tw::new()
                    .p(16.0)
                    .bg(self.tokens.field)
                    .rounded(WorkspaceTokens::RADIUS_MEDIUM as f32)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        for (label, value, color) in terminal_rows(self.tokens) {
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [112.0, 18.0],
                                    egui::Label::new(
                                        egui::RichText::new(label)
                                            .monospace()
                                            .size(12.5)
                                            .color(self.tokens.blue),
                                    ),
                                );
                                ui.label(
                                    egui::RichText::new(value)
                                        .monospace()
                                        .size(12.5)
                                        .color(color),
                                );
                            });
                        }
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new("stan@acme-prod:~$ df -h")
                                .monospace()
                                .size(13.0)
                                .strong()
                                .color(self.tokens.mint),
                        );
                    });
            });
    }

    pub(super) fn show_tokens(&mut self, ui: &mut egui::Ui) {
        surface(ui, self.tokens, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    section_title(ui, "REFERENCE-SAMPLED", "Color and surface tokens", self.tokens);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Semantic roles are sampled from the supplied workspace references and separated by purpose: blue for primary actions, green for active terminal state, and slate for application structure.")
                            .size(12.5)
                            .color(self.tokens.muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    status_label(ui, StatusKind::Info, "Reference sampled", self.tokens);
                });
            });
            ui.add_space(14.0);
            token_row(
                ui,
                self.tokens,
                &[
                    ("Workspace", "bg", self.tokens.bg),
                    ("Panel", "panel", self.tokens.panel),
                    ("Raised panel", "panel-raised", self.tokens.panel_raised),
                    ("Terminal mint", "accent", self.tokens.mint),
                    ("Action blue", "primary", self.tokens.blue),
                ],
            );
            ui.add_space(8.0);
            token_row(
                ui,
                self.tokens,
                &[
                    ("Remote lavender", "rdp", self.tokens.lavender),
                    ("Screen sand", "vnc", self.tokens.sand),
                    ("Danger rose", "danger", self.tokens.rose),
                ],
            );
        });
    }
}

fn session_tab(ui: &mut egui::Ui, label: &str, active: bool, tokens: WorkspaceTokens) {
    egui::Frame::NONE
        .fill(if active {
            WorkspaceTokens::mix(tokens.panel_raised, tokens.mint, 0.07)
        } else {
            tokens.panel_raised
        })
        .stroke(egui::Stroke::new(
            1.0,
            if active {
                WorkspaceTokens::mix(tokens.border, tokens.mint, 0.34)
            } else {
                tokens.border
            },
        ))
        .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if active {
                    status_meter(ui, StatusKind::Live, tokens);
                }
                ui.label(
                    egui::RichText::new(label)
                        .strong()
                        .size(11.5)
                        .color(if active { tokens.mint } else { tokens.muted }),
                );
            });
        });
}

fn terminal_rows(tokens: WorkspaceTokens) -> [(&'static str, &'static str, egui::Color32); 7] {
    [
        ("Logged as:", "stan@acme-prod", tokens.mint),
        ("OS:", "Ubuntu 22.04.4 LTS", tokens.text),
        ("IP address:", "137.184.95.44", tokens.rose),
        ("Uptime:", "23 weeks, 3 days", tokens.text),
        ("Memory:", "RAM - 349M used, 607M available", tokens.text),
        ("", "[███████░░░░░░░░░░░░░░░░░░]", tokens.muted),
        ("Services:", "▲ UFW  ▲ Nginx  ▲ SSH", tokens.mint),
    ]
}

fn token_row(ui: &mut egui::Ui, tokens: WorkspaceTokens, values: &[(&str, &str, egui::Color32)]) {
    ui.columns(5, |columns| {
        for (column, &(name, token, color)) in columns.iter_mut().zip(values) {
            color_swatch(column, name, token, color, tokens);
        }
    });
}
