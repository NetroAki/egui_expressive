use eframe::egui;

use super::app::RemoteWorkspaceShowcase;
use super::tokens::WorkspaceTokens;
use super::widgets::{
    primary_button, protocol_badge, secondary_button, section_title, status_label, status_meter,
    surface, StatusKind,
};

struct HostRecord {
    title: &'static str,
    detail: &'static str,
    protocols: &'static [&'static str],
}

const HOSTS: [HostRecord; 3] = [
    HostRecord {
        title: "Admin Devops Terminal",
        detail: "admin · personal · 10.0.0.15",
        protocols: &["SSH", "SFTP"],
    },
    HostRecord {
        title: "Windows Build Agent",
        detail: "ci · windows · 192.168.1.10",
        protocols: &["RDP"],
    },
    HostRecord {
        title: "Kiosk Debug Rig",
        detail: "lab · input-test · tailscale",
        protocols: &["VNC"],
    },
];

impl RemoteWorkspaceShowcase {
    pub(super) fn show_components(&mut self, ui: &mut egui::Ui) {
        let gap = 16.0;
        let available = ui.available_width();
        let host_width = (available - gap) * 0.58;
        let details_width = available - gap - host_width;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(
                egui::vec2(host_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_width(host_width);
                    surface(ui, self.tokens, |ui| {
                        ui.set_min_width(host_width - 36.0);
                        self.show_hosts(ui);
                    });
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(details_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_width(details_width);
                    surface(ui, self.tokens, |ui| {
                        ui.set_min_width(details_width - 36.0);
                        self.show_host_details(ui);
                    });
                },
            );
        });
    }

    fn show_hosts(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "VAULT", "Host rows", self.tokens);
        ui.add_space(10.0);
        for (index, host) in HOSTS.iter().enumerate() {
            let response = host_row(ui, host, self.selected_host == index, self.tokens);
            if response.clicked() {
                self.selected_host = index;
                self.connection_message = format!("Selected {}", host.title);
            }
            ui.add_space(8.0);
        }
    }

    fn show_host_details(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "CONNECTION", "Host details", self.tokens);
        ui.add_space(10.0);
        ui.columns(2, |fields| {
            field(&mut fields[0], "Address", &mut self.address, self.tokens);
            field(&mut fields[1], "Port", &mut self.port, self.tokens);
        });
        ui.add_space(8.0);
        ui.columns(2, |fields| {
            field(&mut fields[0], "Username", &mut self.username, self.tokens);
            vault_field(&mut fields[1], &mut self.vault, self.tokens);
        });
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if primary_button(ui, "Connect", self.tokens).clicked() {
                self.connection_message = format!("Connected to {}:{}", self.address, self.port);
            }
            secondary_button(ui, "Cancel", self.tokens);
        });
        ui.add_space(18.0);
        section_title(ui, "STATE GRAMMAR", "One consistent meter", self.tokens);
        ui.add_space(8.0);
        self.show_state_meter_gallery(ui);
    }

    fn show_state_meter_gallery(&mut self, ui: &mut egui::Ui) {
        for states in StatusKind::ALL.chunks(3) {
            ui.columns(3, |columns| {
                for (column, state) in columns.iter_mut().zip(states.iter().copied()) {
                    let response = status_label(column, state, state.label(), self.tokens)
                        .interact(egui::Sense::click());
                    if response.clicked() {
                        self.selected_state = state;
                    }
                    if self.selected_state == state {
                        column.painter().rect_stroke(
                            response.rect.expand(2.0),
                            WorkspaceTokens::RADIUS_MEDIUM as f32,
                            egui::Stroke::new(1.0, self.tokens.blue),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            });
            ui.add_space(8.0);
        }
    }
}

fn host_row(
    ui: &mut egui::Ui,
    host: &HostRecord,
    selected: bool,
    tokens: WorkspaceTokens,
) -> egui::Response {
    let accent = protocol_color(host.protocols[0], tokens);
    egui::Frame::NONE
        .fill(if selected {
            WorkspaceTokens::mix(tokens.panel_raised, tokens.mint, 0.075)
        } else {
            tokens.panel_raised
        })
        .stroke(egui::Stroke::new(
            1.0,
            if selected {
                WorkspaceTokens::mix(tokens.border_strong, tokens.mint, 0.42)
            } else {
                tokens.border
            },
        ))
        .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_min_height(46.0);
            ui.horizontal(|ui| {
                host_icon(ui, host.protocols[0], accent, tokens);
                if selected {
                    status_meter(ui, StatusKind::Live, tokens);
                }
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(host.title)
                            .strong()
                            .size(14.0)
                            .color(tokens.text),
                    );
                    ui.label(
                        egui::RichText::new(host.detail)
                            .size(11.0)
                            .color(tokens.muted),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for protocol in host.protocols.iter().rev() {
                        protocol_badge(ui, protocol, protocol_color(protocol, tokens), tokens);
                    }
                });
            });
        })
        .response
        .interact(egui::Sense::click())
}

fn host_icon(ui: &mut egui::Ui, label: &str, color: egui::Color32, tokens: WorkspaceTokens) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        WorkspaceTokens::RADIUS_MEDIUM as f32,
        WorkspaceTokens::mix(tokens.field, color, 0.78),
    );
    ui.painter().rect_stroke(
        rect,
        WorkspaceTokens::RADIUS_MEDIUM as f32,
        egui::Stroke::new(1.0, WorkspaceTokens::mix(tokens.border, color, 0.62)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(11.0),
        tokens.bg_deep,
    );
}

fn protocol_color(protocol: &str, tokens: WorkspaceTokens) -> egui::Color32 {
    match protocol {
        "SSH" => tokens.mint,
        "SFTP" => tokens.blue,
        "RDP" => tokens.lavender,
        _ => tokens.sand,
    }
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String, tokens: WorkspaceTokens) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .strong()
                .size(11.0)
                .color(tokens.muted),
        );
        egui::Frame::NONE
            .fill(tokens.field)
            .stroke(egui::Stroke::new(1.0, tokens.border_strong))
            .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
            .inner_margin(egui::Margin::symmetric(10, 5))
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(value)
                        .frame(egui::Frame::NONE)
                        .desired_width(f32::INFINITY)
                        .text_color(tokens.text),
                );
            });
    });
}

fn vault_field(ui: &mut egui::Ui, vault: &mut usize, tokens: WorkspaceTokens) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new("Vault")
                .strong()
                .size(11.0)
                .color(tokens.muted),
        );
        egui::ComboBox::from_id_salt("workspace_vault")
            .selected_text(["Team Vault", "Personal"][*vault])
            .show_ui(ui, |ui| {
                ui.selectable_value(vault, 0, "Team Vault");
                ui.selectable_value(vault, 1, "Personal");
            });
    });
}
