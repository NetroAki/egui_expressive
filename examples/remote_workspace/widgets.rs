use eframe::egui;
use egui_expressive::widgets::{
    InteractiveFill, PaintedIconButton, PaintedIconButtonStyle, SegmentedBarMeter, SurfaceButton,
    SurfaceButtonStyle,
};
use egui_expressive::{TypeLabel, TypeSpec};

use super::tokens::WorkspaceTokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Default,
    Live,
    Hover,
    Focus,
    Loading,
    Error,
    Empty,
    Info,
}

impl StatusKind {
    pub const ALL: [Self; 6] = [
        Self::Default,
        Self::Hover,
        Self::Focus,
        Self::Loading,
        Self::Error,
        Self::Empty,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Live => "Live",
            Self::Hover => "Hover",
            Self::Focus => "Focus visible",
            Self::Loading => "Loading",
            Self::Error => "Error",
            Self::Empty => "Empty with next step",
            Self::Info => "Info",
        }
    }

    fn heights(self) -> [f32; 3] {
        match self {
            Self::Default | Self::Info => [7.0, 10.0, 7.0],
            Self::Live | Self::Loading => [5.0, 9.0, 13.0],
            Self::Hover => [7.0, 13.0, 7.0],
            Self::Focus => [11.0, 14.0, 11.0],
            Self::Error => [13.0, 4.0, 13.0],
            Self::Empty => [7.0, 10.0, 7.0],
        }
    }

    fn color(self, tokens: WorkspaceTokens) -> egui::Color32 {
        match self {
            Self::Info | Self::Hover => tokens.info,
            Self::Loading => tokens.sand,
            Self::Error => tokens.rose,
            _ => tokens.mint,
        }
    }
}

pub fn status_meter(
    ui: &mut egui::Ui,
    kind: StatusKind,
    tokens: WorkspaceTokens,
) -> egui::Response {
    ui.add(
        SegmentedBarMeter::new(kind.heights(), kind.color(tokens))
            .outlined(kind == StatusKind::Empty),
    )
}

pub fn status_label(
    ui: &mut egui::Ui,
    kind: StatusKind,
    text: &str,
    tokens: WorkspaceTokens,
) -> egui::Response {
    let signal = kind.color(tokens);
    egui::Frame::NONE
        .fill(WorkspaceTokens::mix(tokens.panel_raised, signal, 0.045))
        .stroke(egui::Stroke::new(
            1.0,
            WorkspaceTokens::mix(tokens.border, signal, 0.22),
        ))
        .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_SMALL))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                status_meter(ui, kind, tokens);
                ui.label(egui::RichText::new(text).color(tokens.text).size(12.0));
            });
        })
        .response
}

pub fn section_title(ui: &mut egui::Ui, eyebrow: &str, title: &str, tokens: WorkspaceTokens) {
    ui.add(TypeLabel::new(
        eyebrow,
        TypeSpec::micro_label().color(tokens.blue),
    ));
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(title)
            .size(19.0)
            .strong()
            .color(tokens.text),
    );
}

pub fn surface(
    ui: &mut egui::Ui,
    tokens: WorkspaceTokens,
    contents: impl FnOnce(&mut egui::Ui),
) -> egui::InnerResponse<()> {
    egui::Frame::NONE
        .fill(tokens.panel)
        .stroke(egui::Stroke::new(1.0, tokens.border))
        .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_LARGE))
        .inner_margin(egui::Margin::same(18))
        .show(ui, contents)
}

pub fn raised_surface(
    ui: &mut egui::Ui,
    tokens: WorkspaceTokens,
    contents: impl FnOnce(&mut egui::Ui),
) -> egui::InnerResponse<()> {
    egui::Frame::NONE
        .fill(tokens.panel_raised)
        .stroke(egui::Stroke::new(1.0, tokens.border))
        .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_MEDIUM))
        .inner_margin(egui::Margin::same(12))
        .show(ui, contents)
}

pub fn primary_button(ui: &mut egui::Ui, label: &str, tokens: WorkspaceTokens) -> egui::Response {
    workspace_button(ui, label, true, tokens)
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str, tokens: WorkspaceTokens) -> egui::Response {
    workspace_button(ui, label, false, tokens)
}

fn workspace_button(
    ui: &mut egui::Ui,
    label: &str,
    primary: bool,
    tokens: WorkspaceTokens,
) -> egui::Response {
    let base = if primary {
        tokens.blue_fill
    } else {
        tokens.panel_raised
    };
    let style = SurfaceButtonStyle {
        size: egui::vec2(108.0, 40.0),
        fill: InteractiveFill {
            idle: base,
            hovered: WorkspaceTokens::mix(base, tokens.blue, if primary { 0.14 } else { 0.08 }),
            pressed: WorkspaceTokens::mix(base, tokens.bg_deep, 0.18),
        },
        stroke: egui::Stroke::new(
            1.0,
            if primary {
                WorkspaceTokens::mix(tokens.blue_fill, tokens.blue, 0.32)
            } else {
                tokens.border_strong
            },
        ),
        focus_stroke: egui::Stroke::new(2.0, tokens.mint),
        corner_radius: WorkspaceTokens::RADIUS_MEDIUM as f32,
        text_color: if primary {
            egui::Color32::WHITE
        } else {
            tokens.text
        },
        font_id: egui::FontId::proportional(13.5),
    };
    ui.add(SurfaceButton::new(label, style))
}

pub fn protocol_badge(
    ui: &mut egui::Ui,
    label: &str,
    color: egui::Color32,
    tokens: WorkspaceTokens,
) {
    egui::Frame::NONE
        .fill(WorkspaceTokens::mix(tokens.field, color, 0.10))
        .stroke(egui::Stroke::new(
            1.0,
            WorkspaceTokens::mix(tokens.border, color, 0.42),
        ))
        .corner_radius(egui::CornerRadius::same(WorkspaceTokens::RADIUS_SMALL))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).strong().size(10.0).color(color));
        });
}

pub fn color_swatch(
    ui: &mut egui::Ui,
    name: &str,
    token: &str,
    color: egui::Color32,
    tokens: WorkspaceTokens,
) {
    raised_surface(ui, tokens, |ui| {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 54.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, WorkspaceTokens::RADIUS_MEDIUM as f32, color);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(name).strong().color(tokens.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(token)
                        .monospace()
                        .size(10.0)
                        .color(tokens.muted),
                );
            });
        });
        ui.label(
            egui::RichText::new(token_description(token))
                .size(10.5)
                .color(tokens.quiet),
        );
    });
}

fn token_description(token: &str) -> &'static str {
    match token {
        "bg" => "Near-black workspace",
        "panel" => "Primary content surface",
        "panel-raised" => "Rows, tabs, inspectors",
        "accent" => "Live terminal state",
        "primary" => "Connect and transfer actions",
        "rdp" => "Remote desktop protocol",
        "vnc" => "Screen-sharing protocol",
        "danger" => "Errors and destructive state",
        _ => "Secondary interface copy",
    }
}

#[derive(Clone, Copy)]
pub enum NavIcon {
    Vault,
    Key,
    Transfer,
    Settings,
}

pub fn nav_button(
    ui: &mut egui::Ui,
    icon: NavIcon,
    label: &str,
    selected: bool,
    tokens: WorkspaceTokens,
) -> egui::Response {
    let fill = if selected {
        WorkspaceTokens::mix(tokens.panel_raised, tokens.mint, 0.05)
    } else {
        egui::Color32::TRANSPARENT
    };
    let style = PaintedIconButtonStyle {
        size: egui::vec2(42.0, 42.0),
        fill: InteractiveFill {
            idle: fill,
            hovered: if selected { fill } else { tokens.panel },
            pressed: WorkspaceTokens::mix(tokens.panel_raised, tokens.bg_deep, 0.20),
        },
        stroke: if selected {
            egui::Stroke::new(1.0, tokens.border_strong)
        } else {
            egui::Stroke::NONE
        },
        focus_stroke: egui::Stroke::new(2.0, tokens.mint),
        corner_radius: WorkspaceTokens::RADIUS_MEDIUM as f32,
        icon_color: if selected { tokens.text } else { tokens.quiet },
    };
    ui.add(PaintedIconButton::new(
        style,
        move |painter: &egui::Painter, rect: egui::Rect, color: egui::Color32| {
            paint_nav_icon(painter, rect.center(), icon, color);
        },
    ))
    .on_hover_text(label)
}

fn paint_nav_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    icon: NavIcon,
    color: egui::Color32,
) {
    let stroke = egui::Stroke::new(1.7, color);
    match icon {
        NavIcon::Vault => {
            let body =
                egui::Rect::from_center_size(center + egui::vec2(0.0, 1.5), egui::vec2(18.0, 13.0));
            painter.rect_stroke(body, 1.0, stroke, egui::StrokeKind::Inside);
            painter.line_segment(
                [
                    center + egui::vec2(-5.0, -7.0),
                    center + egui::vec2(-5.0, -10.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(5.0, -7.0),
                    center + egui::vec2(5.0, -10.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-5.0, -10.0),
                    center + egui::vec2(5.0, -10.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-3.0, 1.0),
                    center + egui::vec2(3.0, 1.0),
                ],
                stroke,
            );
        }
        NavIcon::Key => {
            painter.circle_stroke(center + egui::vec2(-4.0, -4.0), 4.0, stroke);
            painter.line_segment(
                [
                    center + egui::vec2(-1.0, -1.0),
                    center + egui::vec2(8.0, 8.0),
                ],
                stroke,
            );
            painter.line_segment(
                [center + egui::vec2(4.0, 4.0), center + egui::vec2(7.0, 1.0)],
                stroke,
            );
        }
        NavIcon::Transfer => {
            painter.line_segment(
                [
                    center + egui::vec2(-8.0, -5.0),
                    center + egui::vec2(8.0, -5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(8.0, -5.0),
                    center + egui::vec2(4.0, -9.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(8.0, 5.0),
                    center + egui::vec2(-8.0, 5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    center + egui::vec2(-8.0, 5.0),
                    center + egui::vec2(-4.0, 9.0),
                ],
                stroke,
            );
        }
        NavIcon::Settings => {
            painter.circle_stroke(center, 5.0, stroke);
            for delta in [
                egui::vec2(0.0, -10.0),
                egui::vec2(10.0, 0.0),
                egui::vec2(0.0, 10.0),
                egui::vec2(-10.0, 0.0),
            ] {
                painter.line_segment([center + delta * 0.68, center + delta], stroke);
            }
        }
    }
}
