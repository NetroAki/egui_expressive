//! Interactive egui_expressive effect evaluator.
//!
//! Run with:
//! `cargo run --example effects_evaluator`
//! Optional gated compile coverage:
//! `cargo run --all-features --example effects_evaluator`

use eframe::egui;
use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use egui_expressive::draw::{
    clipped_layers_gpu_report, clipped_layers_mask_report, composite_layers_gpu_report,
    composite_layers_report, ClipMask,
};
use egui_expressive::{
    app_provided_backdrop_blur_report, blend_color, blur_image, box_shadow, dashed_path,
    dot_matrix, glow, gradient_path_mesh, gradient_rect, inner_shadow, linear_gradient_rect,
    mesh_gradient_patch, noise_rect, paint_image_slot, pattern_fill_path, radial_gradient_rect,
    radial_gradient_rect_stops, scan_lines, soft_glow, soft_inner_shadow, soft_shadow,
    transform_shape, vignette, with_blend_mode, with_clip_path, ArtboardScene, BlendLayer,
    BlendMode, BlurQuality, CheckboxField, ClipShape, DevToolsPanel, Elevation, Fader, GradientDir,
    Knob, LayeredPainter, M3Button, M3Chip, M3Slider, M3Switch, M3TextField, Meter,
    OffscreenRequest, PaintSource, PropRegistry, RenderCapabilities, RenderFeature, RenderQuality,
    RenderReport, RichStroke, SceneNode, SelectField, SelectOption, ShadowOffset, ShapeBuilder,
    SwitchField, TextAreaField, TextField, Transform2D, Tw,
};
use std::sync::Arc;

const CARD_SIZE: Vec2 = Vec2::new(184.0, 112.0);
const LEFT_WIDTH: f32 = 282.0;
const RIGHT_WIDTH: f32 = 320.0;

struct EvaluatorBackdropProvider;

impl egui_expressive::BackdropSnapshotProvider for EvaluatorBackdropProvider {
    fn capture_backdrop_snapshot(
        &self,
        request: &egui_expressive::BackdropCaptureRequest,
    ) -> Result<egui_expressive::BackdropSnapshot, egui_expressive::BackdropCaptureError> {
        let len = request.expected_len()?;
        let mut pixels = vec![0_u8; len];
        for px in pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&[32, 48, 84, 220]);
        }
        egui_expressive::BackdropSnapshot::new(
            request.requested_width,
            request.requested_height,
            pixels,
        )
    }
}

struct FeatureEntry {
    module: &'static str,
    status: FeatureStatus,
    surface: &'static str,
    evaluator: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeatureStatus {
    Interactive,
    ReportBacked,
    ContractOnly,
    FeatureGated,
}

const FEATURE_ENTRIES: &[FeatureEntry] = &[
    FeatureEntry {
        module: "accessibility",
        status: FeatureStatus::ContractOnly,
        surface: "roles, focus rings, motion policy",
        evaluator: "metadata/status rows; native OS handoff remains app-owned",
    },
    FeatureEntry {
        module: "animation",
        status: FeatureStatus::Interactive,
        surface: "Tween, Spring, Animated*",
        evaluator: "live-effect pulse + motion controls",
    },
    FeatureEntry {
        module: "backdrop",
        status: FeatureStatus::ReportBacked,
        surface: "app-provided/app-owned blur reports",
        evaluator: "fidelity matrix calls out source-contract requirements",
    },
    FeatureEntry {
        module: "blur",
        status: FeatureStatus::Interactive,
        surface: "soft shadows/glow/image blur",
        evaluator: "shadow, glow, Gaussian, image blur cards",
    },
    FeatureEntry {
        module: "codegen",
        status: FeatureStatus::ContractOnly,
        surface: "EffectDef/EffectType/BlendMode",
        evaluator: "effect enum/blend coverage + unsupported badges",
    },
    FeatureEntry {
        module: "compat",
        status: FeatureStatus::ContractOnly,
        surface: "HTML/SwiftUI/Tk/PyQt/Kivy aliases",
        evaluator: "compatibility boundary row",
    },
    FeatureEntry {
        module: "debug",
        status: FeatureStatus::FeatureGated,
        surface: "DebugOverlay/debug_label/debug_interaction",
        evaluator: "live DebugOverlay button plus cfg(debug) label",
    },
    FeatureEntry {
        module: "devtools",
        status: FeatureStatus::ContractOnly,
        surface: "Prop registry + DevToolsPanel",
        evaluator: "debug-build diagnostic row",
    },
    FeatureEntry {
        module: "draw",
        status: FeatureStatus::Interactive,
        surface: "gradients, shadows, clips, zstack, painter helpers",
        evaluator: "draw/effect cards + shared stack approximation",
    },
    FeatureEntry {
        module: "editor",
        status: FeatureStatus::Interactive,
        surface: "canvas interactions, snap/alignment",
        evaluator: "draggable/snap canvas cards",
    },
    FeatureEntry {
        module: "figma",
        status: FeatureStatus::ContractOnly,
        surface: "design-token import/export",
        evaluator: "import/codegen workflow row",
    },
    FeatureEntry {
        module: "forms",
        status: FeatureStatus::Interactive,
        surface: "fields, validation, inline edit",
        evaluator: "inspector controls and validation status row",
    },
    FeatureEntry {
        module: "icons",
        status: FeatureStatus::Interactive,
        surface: "Icon/IconButton/icon constants",
        evaluator: "label/icon-like control chips",
    },
    FeatureEntry {
        module: "interaction",
        status: FeatureStatus::Interactive,
        surface: "drag, focus, commands, feedback",
        evaluator: "click/drag/reorder/snap interactions",
    },
    FeatureEntry {
        module: "layout",
        status: FeatureStatus::Interactive,
        surface: "h/v/z stacks, split, app shell",
        evaluator: "three-pane evaluator layout",
    },
    FeatureEntry {
        module: "m3",
        status: FeatureStatus::Interactive,
        surface: "Material 3 components",
        evaluator: "feature matrix + control styling references",
    },
    FeatureEntry {
        module: "platform",
        status: FeatureStatus::ContractOnly,
        surface: "platform descriptors/artifacts",
        evaluator: "readiness/status row",
    },
    FeatureEntry {
        module: "render",
        status: FeatureStatus::ReportBacked,
        surface: "RenderReport, quality, issues",
        evaluator: "composite/clipped report cards + fidelity badges",
    },
    FeatureEntry {
        module: "responsive",
        status: FeatureStatus::Interactive,
        surface: "Breakpoints, Responsive values",
        evaluator: "responsive panes and coverage row",
    },
    FeatureEntry {
        module: "scene",
        status: FeatureStatus::ReportBacked,
        surface: "SceneNode/ArtboardScene/render_scene",
        evaluator: "scene/effect fidelity contract rows",
    },
    FeatureEntry {
        module: "state",
        status: FeatureStatus::Interactive,
        surface: "StateSlot, StateMachine, InteractionState",
        evaluator: "persistent app state for cards/settings",
    },
    FeatureEntry {
        module: "style",
        status: FeatureStatus::Interactive,
        surface: "DesignTokens, palettes, visual states",
        evaluator: "dark tokens, status badges, chips",
    },
    FeatureEntry {
        module: "surface",
        status: FeatureStatus::Interactive,
        surface: "LargeCanvas, ViewportCuller",
        evaluator: "scrollable/large canvas approximation",
    },
    FeatureEntry {
        module: "svg",
        status: FeatureStatus::ContractOnly,
        surface: "SVG path/color/ASE parsers",
        evaluator: "import/codegen workflow row",
    },
    FeatureEntry {
        module: "swiftui",
        status: FeatureStatus::ContractOnly,
        surface: "ViewModifier, Navigator, ScrollList",
        evaluator: "compatibility boundary row",
    },
    FeatureEntry {
        module: "tailwind",
        status: FeatureStatus::Interactive,
        surface: "Tw utility builder",
        evaluator: "Tailwind-style spacing/color tokens reflected in panels",
    },
    FeatureEntry {
        module: "theme",
        status: FeatureStatus::Interactive,
        surface: "Theme, borders, elevation, semantic colors",
        evaluator: "themed dark shell and badges",
    },
    FeatureEntry {
        module: "typography",
        status: FeatureStatus::Interactive,
        surface: "TypeScale/TypeSpec rich text",
        evaluator: "heading/body/chip text hierarchy",
    },
    FeatureEntry {
        module: "vectorize",
        status: FeatureStatus::ContractOnly,
        surface: "raster-to-scene tracing",
        evaluator: "visual validation workflow row",
    },
    FeatureEntry {
        module: "visual_diff",
        status: FeatureStatus::ReportBacked,
        surface: "image diff reports/tolerances",
        evaluator: "quality-review feature row",
    },
    FeatureEntry {
        module: "widgets",
        status: FeatureStatus::Interactive,
        surface: "knobs, faders, meters, data/layout widgets",
        evaluator: "inspector controls + feature family rows",
    },
    FeatureEntry {
        module: "daw",
        status: FeatureStatus::FeatureGated,
        surface: "DAW compatibility namespace",
        evaluator: "feature-gated compatibility row",
    },
    FeatureEntry {
        module: "creative-editors",
        status: FeatureStatus::FeatureGated,
        surface: "creative editor compatibility widgets",
        evaluator: "feature-gated compatibility row",
    },
    FeatureEntry {
        module: "clip-mask",
        status: FeatureStatus::FeatureGated,
        surface: "clipped_shape_cpu exact CPU mask",
        evaluator: "feature-gated clip-mask row + CPU mask report card",
    },
    FeatureEntry {
        module: "wgpu/gpu",
        status: FeatureStatus::FeatureGated,
        surface: "wgpu exact effects/backdrop",
        evaluator: "cfg(wgpu) init, report, source-layer, app-owned, shader-id paths",
    },
    FeatureEntry {
        module: "gpu-effects",
        status: FeatureStatus::FeatureGated,
        surface: "legacy/alias GPU effects feature",
        evaluator: "feature-gated production-fidelity row",
    },
    FeatureEntry {
        module: "native-backdrop",
        status: FeatureStatus::FeatureGated,
        surface: "native backdrop contract substrate",
        evaluator: "feature-gated native backdrop row",
    },
    FeatureEntry {
        module: "native-backdrop-x11",
        status: FeatureStatus::FeatureGated,
        surface: "X11 native-backdrop platform flag",
        evaluator: "feature-gated native backdrop row",
    },
    FeatureEntry {
        module: "native-backdrop-macos",
        status: FeatureStatus::FeatureGated,
        surface: "macOS native-backdrop platform flag",
        evaluator: "feature-gated native backdrop row",
    },
    FeatureEntry {
        module: "native-backdrop-windows",
        status: FeatureStatus::FeatureGated,
        surface: "Windows native-backdrop platform flag",
        evaluator: "feature-gated native backdrop row",
    },
    FeatureEntry {
        module: "native-backdrop-wayland",
        status: FeatureStatus::FeatureGated,
        surface: "Wayland native-backdrop platform flag",
        evaluator: "feature-gated native backdrop row",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectKind {
    DropShadow,
    InnerShadow,
    OuterGlow,
    InnerGlow,
    GaussianBlur,
    Feather,
    Noise,
    UnknownEffect,
    Bevel,
    LiveEffect,
    LinearGradient,
    RadialGradient,
    MeshGradient,
    PathGradient,
    PatternFill,
    ScanLines,
    DotMatrix,
    Vignette,
    RectGlow,
    ImageBlur,
    BlendComposite,
    CompositeReport,
    ClippedLayerReport,
    ClipMaskReport,
    BackdropReport,
    RenderCapability,
    DrawHelpers,
    ImageSlot,
    Transform,
    ClipPath,
}

impl EffectKind {
    fn label(self) -> &'static str {
        match self {
            Self::DropShadow => "Drop shadow",
            Self::InnerShadow => "Inner shadow",
            Self::OuterGlow => "Outer glow",
            Self::InnerGlow => "Inner glow",
            Self::GaussianBlur => "Gaussian blur",
            Self::Feather => "Feather",
            Self::Noise => "Noise / grain",
            Self::UnknownEffect => "Unknown effect",
            Self::Bevel => "Bevel",
            Self::LiveEffect => "Live effect",
            Self::LinearGradient => "Linear gradient",
            Self::RadialGradient => "Radial gradient",
            Self::MeshGradient => "Mesh gradient",
            Self::PathGradient => "Path gradient mesh",
            Self::PatternFill => "Pattern fill",
            Self::ScanLines => "Scan lines",
            Self::DotMatrix => "Dot matrix",
            Self::Vignette => "Vignette",
            Self::RectGlow => "Rect glow helper",
            Self::ImageBlur => "Image blur helper",
            Self::BlendComposite => "Blend composite",
            Self::CompositeReport => "Composite report",
            Self::ClippedLayerReport => "Clipped layer report",
            Self::ClipMaskReport => "Clip-mask report",
            Self::BackdropReport => "Backdrop report",
            Self::RenderCapability => "Render capabilities",
            Self::DrawHelpers => "Draw helper suite",
            Self::ImageSlot => "Image slot fallback",
            Self::Transform => "Transform",
            Self::ClipPath => "Clip path",
        }
    }

    fn short_note(self) -> &'static str {
        match self {
            Self::DropShadow => "CSS-style shadow stack",
            Self::InnerShadow => "Inset edge falloff",
            Self::OuterGlow => "Symmetric soft halo",
            Self::InnerGlow => "Inset colored bloom",
            Self::GaussianBlur => "Painter blur fallback",
            Self::Feather => "Soft edge fade",
            Self::Noise => "Deterministic grain overlay",
            Self::UnknownEffect => "Unrecognized codegen effect",
            Self::Bevel => "Highlight + shade edges",
            Self::LiveEffect => "Animated procedural pass",
            Self::LinearGradient => "Multi-stop linear mesh",
            Self::RadialGradient => "Radial mesh fill",
            Self::MeshGradient => "Bilinear patch mesh",
            Self::PathGradient => "Clipped path gradient",
            Self::PatternFill => "Vector pattern swatch",
            Self::ScanLines => "CRT line overlay",
            Self::DotMatrix => "Halftone dot overlay",
            Self::Vignette => "Dark radial edge falloff",
            Self::RectGlow => "Public glow() helper",
            Self::ImageBlur => "blur_image texture pass",
            Self::BlendComposite => "Blend-mode preview",
            Self::CompositeReport => "CPU blend report path",
            Self::ClippedLayerReport => "Masked blend report path",
            Self::ClipMaskReport => "CPU compound mask path",
            Self::BackdropReport => "Backdrop blur preflight",
            Self::RenderCapability => "Render feature matrix",
            Self::DrawHelpers => "Builders, dashes, layers",
            Self::ImageSlot => "Generated image fallback",
            Self::Transform => "2D affine transform",
            Self::ClipPath => "Polygon mask preview",
        }
    }

    fn default_color(self) -> Color32 {
        match self {
            Self::DropShadow => Color32::from_rgba_unmultiplied(4, 10, 24, 178),
            Self::InnerShadow => Color32::from_rgba_unmultiplied(0, 0, 0, 170),
            Self::OuterGlow => Color32::from_rgba_unmultiplied(75, 190, 255, 145),
            Self::InnerGlow => Color32::from_rgba_unmultiplied(255, 186, 73, 130),
            Self::GaussianBlur => Color32::from_rgba_unmultiplied(115, 120, 255, 120),
            Self::Feather => Color32::from_rgba_unmultiplied(255, 255, 255, 118),
            Self::Noise => Color32::from_rgba_unmultiplied(255, 255, 255, 80),
            Self::UnknownEffect => Color32::from_rgb(255, 93, 126),
            Self::Bevel => Color32::from_rgba_unmultiplied(255, 255, 255, 190),
            Self::LiveEffect => Color32::from_rgba_unmultiplied(150, 255, 210, 150),
            Self::LinearGradient => Color32::from_rgb(88, 166, 255),
            Self::RadialGradient => Color32::from_rgb(255, 132, 202),
            Self::MeshGradient => Color32::from_rgb(104, 220, 255),
            Self::PathGradient => Color32::from_rgb(255, 184, 92),
            Self::PatternFill => Color32::from_rgb(155, 255, 165),
            Self::ScanLines => Color32::from_rgba_unmultiplied(0, 0, 0, 80),
            Self::DotMatrix => Color32::from_rgba_unmultiplied(255, 255, 255, 92),
            Self::Vignette => Color32::from_rgba_unmultiplied(0, 0, 0, 160),
            Self::RectGlow => Color32::from_rgba_unmultiplied(104, 255, 226, 145),
            Self::ImageBlur => Color32::from_rgb(145, 164, 255),
            Self::BlendComposite => Color32::from_rgb(255, 96, 96),
            Self::CompositeReport => Color32::from_rgb(100, 196, 255),
            Self::ClippedLayerReport => Color32::from_rgb(190, 148, 255),
            Self::ClipMaskReport => Color32::from_rgb(120, 220, 255),
            Self::BackdropReport => Color32::from_rgb(104, 180, 255),
            Self::RenderCapability => Color32::from_rgb(255, 206, 94),
            Self::DrawHelpers => Color32::from_rgb(255, 148, 108),
            Self::ImageSlot => Color32::from_rgb(255, 112, 140),
            Self::Transform => Color32::from_rgb(255, 214, 102),
            Self::ClipPath => Color32::from_rgb(150, 128, 255),
        }
    }

    fn fidelity(self) -> EffectFidelity {
        match self {
            Self::Bevel | Self::InnerGlow | Self::LiveEffect | Self::UnknownEffect => {
                EffectFidelity::Unsupported
            }
            Self::Feather
            | Self::GaussianBlur
            | Self::ClipPath
            | Self::ClippedLayerReport
            | Self::BackdropReport => EffectFidelity::Approximate,
            _ => EffectFidelity::Exact,
        }
    }

    fn fidelity_note(self) -> &'static str {
        match self {
            Self::Bevel => "Unsupported by direct codegen callback emission; preview approximates highlight/shadow edges.",
            Self::InnerGlow => "Unsupported by direct codegen callback emission; preview uses inset soft-shadow fallback.",
            Self::LiveEffect => "Unsupported by direct codegen callback emission; preview uses a procedural animated sweep.",
            Self::UnknownEffect => "Unsupported: validates the codegen Unknown(String) fallback path visibly instead of silently ignoring it.",
            Self::Feather => "Approximate: layered translucent expansion, not a true alpha-mask feather.",
            Self::GaussianBlur => "Approximate/CPU: painter fallback plus blurred image helper for judging softness.",
            Self::ClipPath => "Approximate: visible polygon mask preview, not arbitrary native clipping.",
            Self::ClippedLayerReport => "Report-backed masked blend path; exactness depends on backend/resources and surfaced report issues.",
            Self::BackdropReport => "Report-backed preflight: exact backdrop blur requires an installed app-provided source contract/provider.",
            _ => "Exact public helper path or direct effect math is exercised in the preview.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectFidelity {
    Exact,
    Approximate,
    Unsupported,
}

impl EffectFidelity {
    fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Approximate => "approx",
            Self::Unsupported => "unsupported",
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::Exact => Color32::from_rgb(96, 238, 171),
            Self::Approximate => Color32::from_rgb(255, 198, 92),
            Self::Unsupported => Color32::from_rgb(255, 112, 128),
        }
    }
}

#[derive(Clone)]
struct EffectCard {
    id: u64,
    kind: EffectKind,
    enabled: bool,
    pos: Vec2,
    opacity: f32,
    color: Color32,
    secondary: Color32,
    radius: f32,
    spread: f32,
    offset: Vec2,
    amount: f32,
    scale: f32,
    angle: f32,
    seed: u32,
    blend_mode: BlendMode,
}

impl EffectCard {
    fn new(id: u64, kind: EffectKind, pos: Vec2) -> Self {
        let color = kind.default_color();
        Self {
            id,
            kind,
            enabled: true,
            pos,
            opacity: 1.0,
            color,
            secondary: Color32::from_rgb(21, 28, 48),
            radius: match kind {
                EffectKind::DropShadow => 24.0,
                EffectKind::OuterGlow
                | EffectKind::RectGlow
                | EffectKind::GaussianBlur
                | EffectKind::ImageBlur
                | EffectKind::Feather => 22.0,
                EffectKind::InnerShadow | EffectKind::InnerGlow => 18.0,
                EffectKind::Vignette => 0.72,
                _ => 12.0,
            },
            spread: match kind {
                EffectKind::DropShadow => 4.0,
                EffectKind::OuterGlow | EffectKind::RectGlow => 1.0,
                _ => 0.0,
            },
            offset: match kind {
                EffectKind::DropShadow => Vec2::new(12.0, 14.0),
                _ => Vec2::ZERO,
            },
            amount: match kind {
                EffectKind::Noise => 0.22,
                EffectKind::ScanLines => 0.55,
                EffectKind::DotMatrix => 0.62,
                EffectKind::CompositeReport
                | EffectKind::ClippedLayerReport
                | EffectKind::ClipMaskReport
                | EffectKind::BackdropReport
                | EffectKind::RenderCapability => 0.78,
                EffectKind::Bevel => 0.58,
                _ => 0.7,
            },
            scale: match kind {
                EffectKind::Noise => 3.0,
                EffectKind::PatternFill => 18.0,
                EffectKind::DotMatrix => 13.0,
                EffectKind::ScanLines => 4.0,
                EffectKind::PathGradient => 1.0,
                _ => 1.0,
            },
            angle: match kind {
                EffectKind::LinearGradient => 32.0,
                EffectKind::Bevel => 135.0,
                EffectKind::Transform => -9.0,
                _ => 0.0,
            },
            seed: id as u32 * 41 + 7,
            blend_mode: match kind {
                EffectKind::BlendComposite => BlendMode::Overlay,
                _ => BlendMode::Normal,
            },
        }
    }
}

struct EffectsEvaluatorApp {
    effects: Vec<EffectCard>,
    selected_id: u64,
    drag_origin: Option<(u64, Vec2)>,
    show_stack_numbers: bool,
    snap_to_grid: bool,
}

impl Default for EffectsEvaluatorApp {
    fn default() -> Self {
        let kinds = [
            EffectKind::DropShadow,
            EffectKind::OuterGlow,
            EffectKind::LinearGradient,
            EffectKind::InnerShadow,
            EffectKind::InnerGlow,
            EffectKind::GaussianBlur,
            EffectKind::Feather,
            EffectKind::Noise,
            EffectKind::UnknownEffect,
            EffectKind::Bevel,
            EffectKind::LiveEffect,
            EffectKind::RadialGradient,
            EffectKind::MeshGradient,
            EffectKind::PathGradient,
            EffectKind::PatternFill,
            EffectKind::ScanLines,
            EffectKind::DotMatrix,
            EffectKind::Vignette,
            EffectKind::RectGlow,
            EffectKind::ImageBlur,
            EffectKind::BlendComposite,
            EffectKind::CompositeReport,
            EffectKind::ClippedLayerReport,
            EffectKind::ClipMaskReport,
            EffectKind::BackdropReport,
            EffectKind::RenderCapability,
            EffectKind::DrawHelpers,
            EffectKind::ImageSlot,
            EffectKind::Transform,
            EffectKind::ClipPath,
        ];
        let effects = kinds
            .into_iter()
            .enumerate()
            .map(|(idx, kind)| {
                let col = (idx % 4) as f32;
                let row = (idx / 4) as f32;
                EffectCard::new(
                    idx as u64 + 1,
                    kind,
                    Vec2::new(24.0 + col * 152.0, 34.0 + row * 86.0),
                )
            })
            .collect::<Vec<_>>();
        Self {
            selected_id: 1,
            effects,
            drag_origin: None,
            show_stack_numbers: true,
            snap_to_grid: false,
        }
    }
}

impl eframe::App for EffectsEvaluatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self
            .effects
            .iter()
            .any(|effect| effect.enabled && matches!(effect.kind, EffectKind::LiveEffect))
        {
            ui.ctx().request_repaint();
        }
        install_visuals(ui.ctx());

        ui.vertical(|ui| {
            self.top_bar(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.set_height(ui.available_height());
                ui.scope(|ui| {
                    ui.set_width(LEFT_WIDTH);
                    self.stack_panel(ui);
                });
                ui.separator();
                ui.scope(|ui| {
                    ui.set_width((ui.available_width() - RIGHT_WIDTH - 16.0).max(360.0));
                    self.canvas(ui);
                });
                ui.separator();
                ui.scope(|ui| {
                    ui.set_width(RIGHT_WIDTH);
                    self.inspector(ui);
                });
            });
        });
    }
}

impl EffectsEvaluatorApp {
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_rgb(12, 16, 27))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(egui::Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("egui_expressive effect/render evaluator");
                        ui.label(
                            egui::RichText::new(
                                "Drag cards, reorder shared-source approximations, inspect every crate feature family, and check exact/approx/unsupported status.",
                            )
                            .color(Color32::from_rgb(169, 181, 205)),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Coverage matrix: {} public feature families from README/src/lib.rs.",
                                FEATURE_ENTRIES.len()
                            ))
                            .color(Color32::from_rgb(119, 245, 190)),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Reset grid").clicked() {
                            *self = Self::default();
                        }
                        if ui.button("Cascade").clicked() {
                            self.cascade_cards();
                        }
                        if ui.button("Spread").clicked() {
                            self.spread_cards();
                        }
                        ui.checkbox(&mut self.snap_to_grid, "snap");
                        ui.checkbox(&mut self.show_stack_numbers, "order badges");
                    });
                });
            });
    }

    fn stack_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Stack order");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("all").clicked() {
                        for effect in &mut self.effects {
                            effect.enabled = true;
                        }
                    }
                    if ui.small_button("none").clicked() {
                        for effect in &mut self.effects {
                            effect.enabled = false;
                        }
                    }
                });
            });
            ui.label(
                egui::RichText::new("Bottom → top. Order drives overlapping cards and the labeled shared-source approximation.")
                    .color(Color32::from_rgb(150, 160, 183)),
            );
            ui.add_space(8.0);

            let mut move_op: Option<(usize, usize)> = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                let len = self.effects.len();
                for idx in 0..len {
                    let selected = self.effects[idx].id == self.selected_id;
                    let mut enabled = self.effects[idx].enabled;
                    let label = self.effects[idx].kind.label();
                    let id = self.effects[idx].id;
                    let response = egui::Frame::new()
                        .fill(if selected {
                            Color32::from_rgb(33, 47, 78)
                        } else {
                            Color32::from_rgb(18, 24, 39)
                        })
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut enabled, "");
                                let text = format!("{:02}  {label}", idx + 1);
                                if ui.selectable_label(selected, text).clicked() {
                                    self.selected_id = id;
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("↓").clicked() && idx + 1 < len {
                                            move_op = Some((idx, idx + 1));
                                        }
                                        if ui.small_button("↑").clicked() && idx > 0 {
                                            move_op = Some((idx, idx - 1));
                                        }
                                    },
                                );
                            });
                        })
                        .response;
                    if response.clicked() {
                        self.selected_id = id;
                    }
                    self.effects[idx].enabled = enabled;
                    ui.add_space(4.0);
                }
            });

            if let Some((from, to)) = move_op {
                self.effects.swap(from, to);
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let size = Vec2::new(
            ui.available_width().max(420.0),
            ui.available_height().max(520.0),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        if response.clicked() {
            self.drag_origin = None;
        }
        let painter = ui.painter_at(rect);
        paint_canvas_background(&painter, rect);
        paint_sample_shapes(&painter, rect);

        let time = ui.ctx().input(|input| input.time);
        paint_combined_stack_preview(ui, &painter, rect, &self.effects, time);
        for idx in 0..self.effects.len() {
            let card_rect = Rect::from_min_size(rect.min + self.effects[idx].pos, CARD_SIZE);
            let id = ui.id().with(("effect-card", self.effects[idx].id));
            let drag_response = ui.interact(card_rect, id, Sense::click_and_drag());
            if drag_response.clicked() {
                self.selected_id = self.effects[idx].id;
            }
            if drag_response.drag_started() {
                self.drag_origin = Some((self.effects[idx].id, self.effects[idx].pos));
            }
            if drag_response.dragged() {
                if let Some((drag_id, origin)) = self.drag_origin {
                    if drag_id == self.effects[idx].id {
                        let mut next = origin + drag_response.drag_delta();
                        if self.snap_to_grid {
                            next.x = (next.x / 12.0).round() * 12.0;
                            next.y = (next.y / 12.0).round() * 12.0;
                        }
                        next.x = next
                            .x
                            .clamp(-CARD_SIZE.x * 0.55, rect.width() - CARD_SIZE.x * 0.45);
                        next.y = next
                            .y
                            .clamp(-CARD_SIZE.y * 0.55, rect.height() - CARD_SIZE.y * 0.45);
                        self.effects[idx].pos = next;
                    }
                }
            }
            if drag_response.drag_stopped() {
                self.drag_origin = None;
            }
            let selected = self.effects[idx].id == self.selected_id;
            paint_effect_card(
                ui,
                &painter,
                card_rect,
                &self.effects[idx],
                selected,
                idx + 1,
                self.show_stack_numbers,
                time,
            );
        }
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        panel_frame(ui, |ui| {
            let Some(idx) = self.effects.iter().position(|e| e.id == self.selected_id) else {
                ui.label("Select an effect card.");
                return;
            };
            let card = &mut self.effects[idx];
            ui.heading(card.kind.label());
            ui.label(
                egui::RichText::new(card.kind.short_note()).color(Color32::from_rgb(150, 160, 183)),
            );
            ui.add_space(4.0);
            fidelity_badge_ui(ui, card.kind.fidelity());
            ui.label(
                egui::RichText::new(card.kind.fidelity_note())
                    .color(Color32::from_rgb(180, 190, 212))
                    .size(12.0),
            );
            ui.separator();

            ui.checkbox(&mut card.enabled, "Enabled");
            ui.add(egui::Slider::new(&mut card.opacity, 0.0..=1.0).text("opacity"));
            ui.horizontal(|ui| {
                ui.label("color");
                ui.color_edit_button_srgba(&mut card.color);
                ui.label("secondary");
                ui.color_edit_button_srgba(&mut card.secondary);
            });
            ui.add_space(6.0);

            if uses_radius(card.kind) {
                ui.add(
                    egui::Slider::new(&mut card.radius, 0.0..=80.0).text(radius_label(card.kind)),
                );
            }
            if uses_spread(card.kind) {
                ui.add(egui::Slider::new(&mut card.spread, -24.0..=32.0).text("spread"));
            }
            if uses_offset(card.kind) {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut card.offset.x)
                            .speed(0.5)
                            .prefix("x "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut card.offset.y)
                            .speed(0.5)
                            .prefix("y "),
                    );
                });
            }
            if uses_amount(card.kind) {
                ui.add(egui::Slider::new(&mut card.amount, 0.0..=1.0).text("amount"));
            }
            if uses_scale(card.kind) {
                ui.add(egui::Slider::new(&mut card.scale, 1.0..=40.0).text(scale_label(card.kind)));
            }
            if uses_angle(card.kind) {
                ui.add(egui::Slider::new(&mut card.angle, -180.0..=180.0).text("angle"));
            }
            if matches!(card.kind, EffectKind::Noise | EffectKind::PatternFill) {
                ui.add(
                    egui::DragValue::new(&mut card.seed)
                        .speed(1.0)
                        .prefix("seed "),
                );
            }
            if matches!(
                card.kind,
                EffectKind::BlendComposite
                    | EffectKind::CompositeReport
                    | EffectKind::ClippedLayerReport
                    | EffectKind::ClipMaskReport
                    | EffectKind::DrawHelpers
            ) {
                blend_mode_combo(ui, &mut card.blend_mode);
            }

            ui.separator();
            ui.label("Canvas position");
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut card.pos.x)
                        .speed(1.0)
                        .prefix("x "),
                );
                ui.add(
                    egui::DragValue::new(&mut card.pos.y)
                        .speed(1.0)
                        .prefix("y "),
                );
            });

            ui.separator();
            ui.label(
                egui::RichText::new(
                    "Quality check tip: overlap this card with neighboring cards, then move it above/below in Stack order to see whether the effect still reads well.",
                )
                .color(Color32::from_rgb(168, 178, 204)),
            );

            ui.separator();
            feature_coverage_panel(ui);
        });
    }

    fn cascade_cards(&mut self) {
        for (idx, effect) in self.effects.iter_mut().enumerate() {
            effect.pos = Vec2::new(54.0 + idx as f32 * 18.0, 50.0 + idx as f32 * 14.0);
        }
    }

    fn spread_cards(&mut self) {
        for (idx, effect) in self.effects.iter_mut().enumerate() {
            let col = (idx % 5) as f32;
            let row = (idx / 5) as f32;
            effect.pos = Vec2::new(20.0 + col * 142.0, 34.0 + row * 112.0);
        }
    }
}

fn panel_frame<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(Color32::from_rgb(13, 18, 31))
        .stroke(Stroke::new(1.0, Color32::from_rgb(42, 54, 82)))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(egui::Margin::symmetric(14, 14))
        .show(ui, add_contents)
        .inner
}

fn feature_coverage_panel(ui: &mut egui::Ui) {
    ui.heading("All crate feature families");
    ui.label(
        egui::RichText::new(
            "Scope selected: all public crate features. Rows distinguish interactive demos, report-backed contracts, compatibility-only paths, and feature-gated surfaces.",
        )
        .color(Color32::from_rgb(165, 177, 205))
        .size(12.0),
    );
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for status in [
            FeatureStatus::Interactive,
            FeatureStatus::ReportBacked,
            FeatureStatus::ContractOnly,
            FeatureStatus::FeatureGated,
        ] {
            let count = FEATURE_ENTRIES
                .iter()
                .filter(|entry| entry.status == status)
                .count();
            status_chip_ui(ui, status, count);
        }
    });
    let _ = egui_expressive::zstack(
        ui,
        |ui| ui.small_button("zstack base"),
        |ui| {
            ui.label(egui::RichText::new("●").color(Color32::from_rgb(255, 205, 96)));
        },
    );
    live_api_smoke_rack(ui);
    ui.add_space(6.0);
    egui::ScrollArea::vertical()
        .max_height(250.0)
        .show(ui, |ui| {
            for entry in FEATURE_ENTRIES {
                egui::Frame::new()
                    .fill(Color32::from_rgb(17, 23, 38))
                    .stroke(Stroke::new(1.0, with_opacity(entry.status.color(), 0.35)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(entry.module)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(entry.status.label())
                                            .color(entry.status.color())
                                            .size(11.0),
                                    );
                                },
                            );
                        });
                        ui.label(
                            egui::RichText::new(entry.surface)
                                .color(Color32::from_rgb(184, 194, 216))
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(entry.evaluator)
                                .color(Color32::from_rgb(135, 214, 184))
                                .size(10.5),
                        );
                    });
                ui.add_space(4.0);
            }
        });
}

fn live_api_smoke_rack(ui: &mut egui::Ui) {
    ui.collapsing("Live API smoke rack", |ui| {
        Tw::new().p(6.0).show(ui, |ui| {
            let mut m3_text = "feature probe".to_owned();
            ui.add(M3TextField::new(
                "effects_evaluator.m3_text",
                "M3 text field",
                &mut m3_text,
            ));

            ui.horizontal_wrapped(|ui| {
                let _ = ui.add(M3Button::new("M3 button").icon('✓').width(112.0));
                let _ = ui.add(M3Chip::new("M3 chip").filter().selected(true).icon('◆'));
            });

            let mut m3_value = 0.42_f32;
            ui.add(M3Slider::new(&mut m3_value, 0.0..=1.0).steps(4));
            let mut m3_switch = true;
            ui.add(M3Switch::new("effects_evaluator.m3_switch", &mut m3_switch));
        });

        Tw::new()
            .p(6.0)
            .rounded_xl()
            .shadow(Elevation::Level2)
            .bg_gradient_to_r(
                Color32::from_rgb(24, 34, 63),
                Color32::from_rgb(69, 42, 112),
            )
            .drop_shadow(
                Vec2::new(0.0, 8.0),
                18,
                Color32::from_rgba_unmultiplied(0, 0, 0, 110),
            )
            .ring(1.0, Color32::from_rgba_unmultiplied(130, 190, 255, 120))
            .backdrop_blur_app_provided(12.0)
            .transition(0.16)
            .show(ui, |ui| {
                ui.label("Tailwind effects: shadow/gradient/drop-shadow/ring/backdrop/transition");
            });
        egui_expressive::with_opacity(ui, 0.86, |ui| {
            ui.label("draw::with_opacity scoped helper");
        });

        let mut form_text = "forms API".to_owned();
        TextField::new("TextField", &mut form_text)
            .hint("form text")
            .show(ui);
        let mut form_notes = "validation/report notes".to_owned();
        TextAreaField::new("TextArea", &mut form_notes)
            .rows(2)
            .show(ui);
        let mut form_check = true;
        CheckboxField::new("CheckboxField", &mut form_check).show(ui);
        SwitchField::new("SwitchField", &mut form_check).show(ui);
        let mut status = FeatureStatus::Interactive;
        SelectField::new("Feature status", &mut status)
            .options([
                SelectOption::new(FeatureStatus::Interactive, "Interactive"),
                SelectOption::new(FeatureStatus::ReportBacked, "Report-backed"),
                SelectOption::new(FeatureStatus::ContractOnly, "Contract"),
                SelectOption::new(FeatureStatus::FeatureGated, "Feature-gated"),
            ])
            .show(ui);

        let scene = ArtboardScene {
            name: "feature-smoke-scene".to_owned(),
            width: 118.0,
            height: 48.0,
            nodes: vec![
                SceneNode::rect(
                    "scene-node",
                    Rect::from_min_size(Pos2::new(8.0, 8.0), Vec2::new(72.0, 28.0)),
                    8.0,
                )
                .with_fill(PaintSource::Solid(Color32::from_rgb(54, 110, 220)))
                .with_stroke(PaintSource::Solid(Color32::WHITE), 1.0)
                .with_effect(egui_expressive::EffectDef {
                    effect_type: egui_expressive::EffectType::DropShadow,
                    blur: 6.0,
                    color: Color32::from_rgba_unmultiplied(0, 0, 0, 90),
                    ..Default::default()
                }),
                SceneNode::ellipse(
                    "scene-ellipse",
                    Rect::from_min_size(Pos2::new(78.0, 10.0), Vec2::new(24.0, 22.0)),
                )
                .with_fill(PaintSource::ProceduralNoise(
                    egui_expressive::NoiseDef {
                        seed: 7,
                        cell_size: 3.0,
                        opacity: 0.22,
                    },
                )),
                SceneNode::path(
                    "scene-path",
                    vec![
                        Pos2::new(6.0, 40.0),
                        Pos2::new(42.0, 36.0),
                        Pos2::new(88.0, 42.0),
                    ],
                    false,
                )
                .with_stroke(PaintSource::Solid(Color32::from_rgb(255, 220, 120)), 1.4),
            ],
        };
        egui_expressive::render_scene(ui, &scene);

        let registry = PropRegistry::get(ui.ctx());
        if let Ok(mut reg) = registry.lock() {
            reg.register_color("Evaluator", "accent", Color32::from_rgb(110, 190, 255));
            reg.register_float("Evaluator", "radius", 12.0, 0.0, 48.0);
            reg.register_bool("Evaluator", "enabled", true);
            reg.register_vec2("Evaluator", "offset", Vec2::new(8.0, 10.0));
        }
        let mut devtools_open = false;
        DevToolsPanel::show(ui.ctx(), &mut devtools_open);

        ui.horizontal_wrapped(|ui| {
            let mut knob = 0.64_f64;
            let mut fader = -5.0_f64;
            ui.add(Knob::new(&mut knob, 0.0..=1.0).size(42.0).label("Knob"));
            ui.add(Fader::new(&mut fader, -18.0..=6.0).size(egui::vec2(28.0, 78.0)));
            ui.add(Meter::new(0.68).size(egui::vec2(10.0, 78.0)).segments(8));
        });

        let response = ui.small_button("DebugOverlay target");
        let debug_overlay = egui_expressive::debug::DebugOverlay {
            show_interaction_state: true,
            ..Default::default()
        };
        debug_overlay.response(ui.ctx(), &response, "effects-evaluator-debug");
        #[cfg(feature = "debug")]
        egui_expressive::debug::debug_label(ui.ctx(), response.rect.right_top(), "debug feature");
        #[cfg(feature = "debug")]
        egui_expressive::debug::debug_interaction(ui.ctx(), &response, "debug_interaction");
        debug_overlay.show_all(ui.ctx());

        #[cfg(any(feature = "daw", feature = "creative-editors"))]
        ui.horizontal_wrapped(|ui| {
            let mut active = true;
            ui.add(egui_expressive::daw::TransportButton::new(
                egui_expressive::daw::TransportKind::Play,
                &mut active,
            ));
            egui_expressive::daw::icon_play(
                ui.painter(),
                ui.cursor().center(),
                18.0,
                Color32::from_rgb(122, 236, 190),
            );
            ui.label("daw/creative-editors namespace");
        });

        #[cfg(feature = "native-backdrop")]
        {
            let native_symbols = [
                std::any::type_name::<egui_expressive::NativeBackdropPlatform>(),
                std::any::type_name::<egui_expressive::NativeBackdropSupportState>(),
                std::any::type_name::<egui_expressive::NativeBackdropPermissionState>(),
                std::any::type_name::<egui_expressive::NativeBackdropSourceScope>(),
                std::any::type_name::<egui_expressive::NativeBackdropContractDiagnostic>(),
                std::any::type_name::<egui_expressive::NativeBackdropSmokeArtifact>(),
                std::any::type_name::<egui_expressive::NativeBackdropInitError>(),
            ];
            ui.label(format!(
                "native backdrop flags: {} / {} / {} / {} / {} · family {} · types {}",
                egui_expressive::NATIVE_BACKDROP_FEATURE,
                egui_expressive::NATIVE_BACKDROP_X11_FEATURE,
                egui_expressive::NATIVE_BACKDROP_MACOS_FEATURE,
                egui_expressive::NATIVE_BACKDROP_WINDOWS_FEATURE,
                egui_expressive::NATIVE_BACKDROP_WAYLAND_FEATURE,
                egui_expressive::NativeBackdropSupportFamily::LinuxX11.label(),
                native_symbols.len()
            ));
        }

        broad_public_family_symbol_touch(ui);
        export_pipeline_smoke(ui);
    });
}

fn export_pipeline_smoke(ui: &mut egui::Ui) {
    let mut effect_def = egui_expressive::EffectDef {
        effect_type: egui_expressive::EffectType::Unknown("mystery".to_owned()),
        ..Default::default()
    };
    effect_def.blend_mode = BlendMode::Overlay;
    let gradient = egui_expressive::GradientDef {
        gradient_type: egui_expressive::GradientType::Radial,
        angle_deg: 35.0,
        center: Some([0.5, 0.5]),
        focal_point: None,
        radius: Some(0.6),
        transform: None,
        stops: vec![egui_expressive::GradientStop {
            position: 0.0,
            color: Color32::WHITE,
        }],
    };
    let third_party = egui_expressive::ThirdPartyEffect {
        effect_type: "plugin-shadow".to_owned(),
        opaque: false,
        note: "listed for evaluator coverage".to_owned(),
    };
    let svg_color = egui_expressive::parse_svg_color("#66ccff")
        .map(|color| {
            format!(
                "svg color #{:02x}{:02x}{:02x}",
                color.r(),
                color.g(),
                color.b()
            )
        })
        .unwrap_or_else(|| "svg color parse failed".to_owned());
    let svg_points = egui_expressive::svg_path_to_points("M0 0 L12 0 L12 12 Z").len();
    let tiny = image::RgbaImage::from_pixel(1, 1, image::Rgba([64, 128, 255, 255]));
    let diff = egui_expressive::diff_rgba_images(
        &tiny,
        &tiny,
        egui_expressive::VisualDiffConfig::default(),
    );
    let vectorized = egui_expressive::vectorize_rgba_to_scene_nodes(
        "evaluator-vector",
        &tiny,
        &egui_expressive::RasterVectorizeConfig::default(),
    )
    .map(|nodes| nodes.len())
    .unwrap_or_default();
    ui.small(format!(
        "export/vector/diff/codegen smoke: {svg_color}; points={svg_points}; diff_passed={}; vector_nodes={vectorized}; effect={:?}; grad={:?}; third_party={}",
        diff.passed,
        effect_def.effect_type,
        gradient.gradient_type,
        third_party.effect_type
    ));
}

fn broad_public_family_symbol_touch(ui: &mut egui::Ui) {
    let symbols = [
        std::any::type_name::<egui_expressive::AccessibilityMeta>(),
        std::any::type_name::<egui_expressive::AccessibilityRole>(),
        std::any::type_name::<egui_expressive::FocusRing>(),
        std::any::type_name::<egui_expressive::MotionPolicy>(),
        std::any::type_name::<egui_expressive::FeedbackQueue>(),
        std::any::type_name::<egui_expressive::InteractionState>(),
        std::any::type_name::<egui_expressive::StateSlot<bool>>(),
        std::any::type_name::<egui_expressive::StateMachine<bool>>(),
        std::any::type_name::<egui_expressive::Responsive<i32>>(),
        std::any::type_name::<egui_expressive::Breakpoints>(),
        std::any::type_name::<egui_expressive::Theme>(),
        std::any::type_name::<egui_expressive::DesignTokens>(),
        std::any::type_name::<egui_expressive::TypeSpec>(),
        std::any::type_name::<egui_expressive::TypeScale>(),
        std::any::type_name::<egui_expressive::Icon>(),
        std::any::type_name::<egui_expressive::IconButton>(),
        std::any::type_name::<egui_expressive::EditorCanvas>(),
        std::any::type_name::<egui_expressive::EditorCanvasContext<'static>>(),
        std::any::type_name::<egui_expressive::LargeCanvas>(),
        std::any::type_name::<egui_expressive::ViewportCuller>(),
        std::any::type_name::<egui_expressive::VisualDiffConfig>(),
        std::any::type_name::<egui_expressive::VisualDiffReport>(),
        std::any::type_name::<egui_expressive::PlatformSupportArtifact>(),
        std::any::type_name::<egui_expressive::PlatformSmokeResult>(),
        std::any::type_name::<egui_expressive::DashPattern>(),
        std::any::type_name::<egui_expressive::GradientPathGeometry>(),
        std::any::type_name::<egui_expressive::RadialGradientDir>(),
        std::any::type_name::<egui_expressive::StackAlign>(),
        std::any::type_name::<egui_expressive::StrokeCap>(),
        std::any::type_name::<egui_expressive::StrokeJoin>(),
    ];
    ui.small(format!("public API symbols touched: {}", symbols.len()));
}

fn status_chip_ui(ui: &mut egui::Ui, status: FeatureStatus, count: usize) {
    egui::Frame::new()
        .fill(with_opacity(status.color(), 0.16))
        .stroke(Stroke::new(1.0, with_opacity(status.color(), 0.6)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("{} {count}", status.label()))
                    .color(Color32::WHITE)
                    .size(10.5),
            );
        });
}

impl FeatureStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::ReportBacked => "report",
            Self::ContractOnly => "contract",
            Self::FeatureGated => "gated",
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::Interactive => Color32::from_rgb(104, 232, 177),
            Self::ReportBacked => Color32::from_rgb(117, 184, 255),
            Self::ContractOnly => Color32::from_rgb(255, 204, 112),
            Self::FeatureGated => Color32::from_rgb(206, 150, 255),
        }
    }
}

fn paint_canvas_background(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(rect, CornerRadius::same(20), Color32::from_rgb(7, 10, 18));
    painter.add(gradient_rect(
        rect,
        Color32::from_rgb(12, 18, 32),
        Color32::from_rgb(4, 8, 16),
    ));
    let grid = Color32::from_rgba_unmultiplied(116, 144, 190, 22);
    let mut x = rect.left();
    while x < rect.right() {
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(1.0, grid),
        );
        x += 24.0;
    }
    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(1.0, grid),
        );
        y += 24.0;
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(20),
        Stroke::new(1.0, Color32::from_rgb(37, 51, 82)),
        StrokeKind::Inside,
    );
}

fn paint_sample_shapes(painter: &egui::Painter, rect: Rect) {
    let a = rect.left_top() + Vec2::new(rect.width() * 0.58, rect.height() * 0.16);
    let b = rect.left_top() + Vec2::new(rect.width() * 0.78, rect.height() * 0.78);
    painter.circle_filled(a, 88.0, Color32::from_rgba_unmultiplied(44, 111, 255, 26));
    painter.circle_stroke(
        a,
        88.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(99, 158, 255, 74)),
    );
    painter.rect_filled(
        Rect::from_center_size(b, Vec2::new(210.0, 138.0)),
        CornerRadius::same(32),
        Color32::from_rgba_unmultiplied(255, 114, 180, 22),
    );
}

fn paint_combined_stack_preview(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    canvas_rect: Rect,
    effects: &[EffectCard],
    time: f64,
) {
    let preview_width = (canvas_rect.width() - 56.0).clamp(300.0, 430.0);
    let panel_rect = Rect::from_min_size(
        canvas_rect.left_bottom() + Vec2::new(28.0, -170.0),
        Vec2::new(preview_width, 136.0),
    );
    painter.rect_filled(
        panel_rect,
        CornerRadius::same(18),
        Color32::from_rgba_unmultiplied(5, 8, 16, 218),
    );
    painter.rect_stroke(
        panel_rect,
        CornerRadius::same(18),
        Stroke::new(1.0, Color32::from_rgb(64, 80, 120)),
        StrokeKind::Inside,
    );
    painter.text(
        panel_rect.left_top() + Vec2::new(14.0, 10.0),
        egui::Align2::LEFT_TOP,
        "Shared stack preview",
        egui::FontId::proportional(14.0),
        Color32::WHITE,
    );
    painter.text(
        panel_rect.left_top() + Vec2::new(14.0, 30.0),
        egui::Align2::LEFT_TOP,
        "Enabled effects apply bottom → top through documented egui approximations.",
        egui::FontId::proportional(11.0),
        Color32::from_rgb(164, 176, 205),
    );

    let source_rect = Rect::from_min_size(
        panel_rect.left_top() + Vec2::new(20.0, 58.0),
        Vec2::new((panel_rect.width() - 40.0).max(160.0), 58.0),
    );
    for card in effects.iter().filter(|card| card.enabled) {
        let opacity = card.opacity;
        paint_effect_behind(
            painter,
            source_rect,
            card,
            with_opacity(card.color, opacity),
            with_opacity(card.secondary, opacity),
            time,
        );
    }

    painter.rect_filled(
        source_rect,
        CornerRadius::same(18),
        Color32::from_rgb(28, 38, 65),
    );
    painter.text(
        source_rect.center(),
        egui::Align2::CENTER_CENTER,
        "source object",
        egui::FontId::proportional(14.0),
        Color32::from_rgb(220, 229, 245),
    );

    for card in effects.iter().filter(|card| card.enabled) {
        paint_stack_layer(ui, painter, source_rect, card, time);
    }

    painter.rect_stroke(
        source_rect,
        CornerRadius::same(18),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(235, 242, 255, 145)),
        StrokeKind::Inside,
    );
}

fn paint_stack_layer(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
    time: f64,
) {
    let accent = with_opacity(card.color, card.opacity * 0.58);
    let secondary = with_opacity(card.secondary, card.opacity * 0.58);
    match card.kind {
        EffectKind::LinearGradient => {
            painter.add(linear_gradient_rect(
                rect.shrink(3.0),
                &[(0.0, accent), (1.0, secondary)],
                GradientDir::Angle(card.angle),
            ));
        }
        EffectKind::RadialGradient => {
            painter.add(radial_gradient_rect(
                rect.shrink(3.0),
                accent,
                secondary,
                32,
            ));
        }
        EffectKind::MeshGradient => {
            painter.add(mesh_gradient_patch(
                [
                    rect.left_top(),
                    rect.right_top(),
                    rect.right_bottom(),
                    rect.left_bottom(),
                ],
                [
                    accent,
                    secondary,
                    with_opacity(Color32::WHITE, 0.24),
                    accent,
                ],
                10,
            ));
        }
        EffectKind::PathGradient => paint_path_gradient(painter, rect.shrink(5.0), card),
        EffectKind::PatternFill => paint_pattern(painter, rect.shrink(4.0), card, accent),
        EffectKind::ImageBlur => paint_image_blur(ui, painter, rect.shrink(8.0), card),
        EffectKind::CompositeReport => paint_composite_report(ui, painter, rect.shrink(6.0), card),
        EffectKind::ClippedLayerReport => {
            paint_clipped_layer_report(ui, painter, rect.shrink(6.0), card)
        }
        EffectKind::ClipMaskReport => paint_clip_mask_report(ui, painter, rect.shrink(6.0), card),
        EffectKind::BackdropReport => paint_backdrop_report(ui, painter, rect.shrink(6.0), card),
        EffectKind::RenderCapability => paint_render_capabilities(painter, rect.shrink(6.0)),
        EffectKind::DrawHelpers => paint_draw_helpers(ui, painter, rect.shrink(6.0), card),
        EffectKind::ImageSlot => {
            paint_image_slot(
                ui,
                painter,
                rect.shrink(9.0),
                None,
                "effects-evaluator-stack-image-slot",
                with_opacity(accent, 0.95),
                "slot",
            );
        }
        _ => paint_effect_overlay(painter, rect, card, accent, secondary, time),
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_effect_card(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
    selected: bool,
    order: usize,
    show_order: bool,
    time: f64,
) {
    let opacity = if card.enabled { card.opacity } else { 0.22 };
    let base = Color32::from_rgba_unmultiplied(22, 30, 52, (232.0 * opacity) as u8);
    let accent = with_opacity(card.color, opacity);
    let secondary = with_opacity(card.secondary, opacity);

    if card.enabled {
        paint_effect_behind(painter, rect, card, accent, secondary, time);
    }

    paint_card_fill(ui, painter, rect, card, base, accent, secondary, opacity);

    if card.enabled {
        paint_effect_overlay(painter, rect, card, accent, secondary, time);
    }

    let stroke = if selected {
        Stroke::new(2.0, Color32::from_rgb(255, 226, 125))
    } else {
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(151, 174, 220, (95.0 * opacity) as u8),
        )
    };
    painter.rect_stroke(rect, CornerRadius::same(16), stroke, StrokeKind::Inside);

    let title_color = if card.enabled {
        Color32::WHITE
    } else {
        Color32::from_gray(140)
    };
    painter.text(
        rect.left_top() + Vec2::new(12.0, 12.0),
        egui::Align2::LEFT_TOP,
        card.kind.label(),
        egui::FontId::proportional(16.0),
        title_color,
    );
    painter.text(
        rect.left_top() + Vec2::new(12.0, 36.0),
        egui::Align2::LEFT_TOP,
        card.kind.short_note(),
        egui::FontId::proportional(11.5),
        Color32::from_rgba_unmultiplied(196, 207, 229, (205.0 * opacity) as u8),
    );

    let chip = Rect::from_min_size(
        rect.left_bottom() + Vec2::new(12.0, -30.0),
        Vec2::new(76.0, 20.0),
    );
    painter.rect_filled(
        chip,
        CornerRadius::same(10),
        Color32::from_rgba_unmultiplied(0, 0, 0, (72.0 * opacity) as u8),
    );
    painter.text(
        chip.center(),
        egui::Align2::CENTER_CENTER,
        if card.enabled { "enabled" } else { "disabled" },
        egui::FontId::proportional(11.0),
        Color32::from_rgba_unmultiplied(220, 230, 245, (230.0 * opacity) as u8),
    );

    let fidelity = card.kind.fidelity();
    let status_chip = Rect::from_min_size(
        chip.right_top() + Vec2::new(6.0, 0.0),
        Vec2::new(82.0, 20.0),
    );
    painter.rect_filled(
        status_chip,
        CornerRadius::same(10),
        with_opacity(fidelity.color(), 0.22 * opacity),
    );
    painter.rect_stroke(
        status_chip,
        CornerRadius::same(10),
        Stroke::new(1.0, with_opacity(fidelity.color(), 0.78 * opacity)),
        StrokeKind::Inside,
    );
    painter.text(
        status_chip.center(),
        egui::Align2::CENTER_CENTER,
        fidelity.label(),
        egui::FontId::proportional(10.5),
        with_opacity(Color32::WHITE, opacity),
    );

    if show_order {
        let badge_center = rect.right_top() + Vec2::new(-18.0, 18.0);
        painter.circle_filled(badge_center, 13.0, Color32::from_rgb(255, 205, 96));
        painter.text(
            badge_center,
            egui::Align2::CENTER_CENTER,
            order.to_string(),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(15, 18, 28),
        );
    }
}

fn paint_effect_behind(
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
    accent: Color32,
    secondary: Color32,
    time: f64,
) {
    match card.kind {
        EffectKind::DropShadow => {
            for shape in soft_shadow(
                rect.shrink(4.0),
                accent,
                card.radius,
                card.spread,
                ShadowOffset::new(card.offset.x, card.offset.y),
                BlurQuality::High,
            ) {
                painter.add(shape);
            }
        }
        EffectKind::OuterGlow => {
            for shape in soft_glow(rect.shrink(3.0), accent, card.radius, BlurQuality::High) {
                painter.add(shape);
            }
        }
        EffectKind::RectGlow => {
            for shape in glow(rect.shrink(4.0), accent, card.radius) {
                painter.add(shape);
            }
        }
        EffectKind::GaussianBlur => {
            for shape in soft_shadow(
                rect.shrink(8.0),
                accent,
                card.radius.max(4.0),
                card.spread,
                ShadowOffset::zero(),
                BlurQuality::Medium,
            ) {
                painter.add(shape);
            }
        }
        EffectKind::Feather => {
            let steps = 8;
            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let alpha = ((1.0 - t) * 42.0 * card.opacity) as u8;
                painter.rect_filled(
                    rect.expand(card.radius * t * 0.6),
                    CornerRadius::same(18),
                    Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha),
                );
            }
        }
        EffectKind::LiveEffect => {
            let pulse = ((time as f32 * 2.2).sin() * 0.5 + 0.5) * card.amount;
            for i in 0..3 {
                let grow = (i as f32 * 10.0) + pulse * 18.0;
                painter.rect_stroke(
                    rect.expand(grow),
                    CornerRadius::same(18),
                    Stroke::new(1.0 + pulse, with_opacity(accent, 0.22 - i as f32 * 0.05)),
                    StrokeKind::Outside,
                );
            }
        }
        EffectKind::Transform => {
            let shape = egui::Shape::rect_filled(
                rect.shrink(16.0),
                CornerRadius::same(18),
                with_opacity(accent, 0.35),
            );
            let t = Transform2D::rotate_around(card.angle, rect.center());
            painter.add(transform_shape(shape, &t));
        }
        EffectKind::BlendComposite => {
            let blended = blend_color(accent, secondary, card.blend_mode.clone());
            painter.circle_filled(
                rect.center() + Vec2::new(-28.0, 6.0),
                44.0,
                with_opacity(accent, 0.62),
            );
            painter.circle_filled(
                rect.center() + Vec2::new(28.0, -4.0),
                44.0,
                with_opacity(secondary, 0.62),
            );
            painter.circle_filled(rect.center(), 30.0, with_opacity(blended, 0.88));
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_card_fill(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
    base: Color32,
    accent: Color32,
    secondary: Color32,
    opacity: f32,
) {
    match card.kind {
        EffectKind::LinearGradient => {
            painter.add(linear_gradient_rect(
                rect,
                &[
                    (0.0, accent),
                    (0.52, Color32::from_rgb(38, 46, 78)),
                    (1.0, secondary),
                ],
                GradientDir::Angle(card.angle),
            ));
        }
        EffectKind::RadialGradient => {
            painter.add(radial_gradient_rect_stops(
                rect,
                &[(0.0, Color32::WHITE), (0.35, accent), (1.0, secondary)],
                64,
            ));
        }
        EffectKind::MeshGradient => {
            painter.add(mesh_gradient_patch(
                [
                    rect.left_top(),
                    rect.right_top(),
                    rect.right_bottom(),
                    rect.left_bottom(),
                ],
                [
                    accent,
                    Color32::from_rgb(255, 176, 104),
                    secondary,
                    Color32::from_rgb(96, 255, 196),
                ],
                18,
            ));
        }
        EffectKind::PathGradient => paint_path_gradient(painter, rect.shrink(5.0), card),
        EffectKind::PatternFill => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_pattern(painter, rect, card, accent);
        }
        EffectKind::ImageBlur => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_image_blur(ui, painter, rect.shrink(12.0), card);
        }
        EffectKind::CompositeReport => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_composite_report(ui, painter, rect.shrink(10.0), card);
        }
        EffectKind::ClippedLayerReport => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_clipped_layer_report(ui, painter, rect.shrink(10.0), card);
        }
        EffectKind::ClipMaskReport => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_clip_mask_report(ui, painter, rect.shrink(10.0), card);
        }
        EffectKind::BackdropReport => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_backdrop_report(ui, painter, rect.shrink(10.0), card);
        }
        EffectKind::RenderCapability => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_render_capabilities(painter, rect.shrink(10.0));
        }
        EffectKind::DrawHelpers => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_draw_helpers(ui, painter, rect.shrink(10.0), card);
        }
        EffectKind::ImageSlot => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
            paint_image_slot(
                ui,
                painter,
                rect.shrink(18.0),
                None,
                "effects-evaluator-image-slot",
                with_opacity(accent, 0.95),
                "image slot fallback",
            );
        }
        EffectKind::ClipPath => {
            let points = hex_points(rect.shrink(8.0));
            painter.add(radial_gradient_rect(
                rect,
                with_opacity(accent, opacity),
                with_opacity(secondary, opacity),
                48,
            ));
            painter.add(egui::Shape::convex_polygon(
                points.clone(),
                with_opacity(Color32::from_rgb(9, 13, 24), 0.52),
                Stroke::new(1.0, accent),
            ));
            for shape in dot_matrix(rect.shrink(10.0), 12.0, 1.6, with_opacity(accent, 0.32)) {
                painter.add(shape);
            }
            painter.add(egui::Shape::closed_line(
                points,
                Stroke::new(2.0, Color32::WHITE),
            ));
        }
        _ => {
            painter.rect_filled(rect, CornerRadius::same(16), base);
        }
    };

    if !matches!(
        card.kind,
        EffectKind::LinearGradient
            | EffectKind::RadialGradient
            | EffectKind::MeshGradient
            | EffectKind::PathGradient
            | EffectKind::ImageBlur
            | EffectKind::CompositeReport
            | EffectKind::ClippedLayerReport
            | EffectKind::ClipMaskReport
            | EffectKind::BackdropReport
            | EffectKind::RenderCapability
            | EffectKind::DrawHelpers
            | EffectKind::ImageSlot
            | EffectKind::ClipPath
    ) {
        painter.rect_filled(
            rect.shrink(7.0),
            CornerRadius::same(12),
            with_opacity(Color32::from_rgb(31, 39, 66), 0.72 * opacity),
        );
    }

    if matches!(card.kind, EffectKind::GaussianBlur) {
        let blur_rect = rect.shrink(26.0).translate(Vec2::new(0.0, 14.0));
        let image = egui::ColorImage::example();
        let (shape, texture) =
            egui_expressive::blurred_image_shape(ui.ctx(), image, card.radius as u32, blur_rect);
        painter.add(shape);
        ui.ctx()
            .data_mut(|data| data.insert_temp(ui.id().with(("blur-texture", card.id)), texture));
    }
}

fn paint_path_gradient(painter: &egui::Painter, rect: Rect, card: &EffectCard) {
    let points = hex_points(rect);
    if let Some(shape) = gradient_path_mesh(
        &points,
        &[
            (0.0, with_opacity(card.color, card.opacity)),
            (0.48, with_opacity(Color32::WHITE, 0.72 * card.opacity)),
            (1.0, with_opacity(card.secondary, card.opacity)),
        ],
        card.angle,
        card.amount > 0.5,
    ) {
        painter.add(shape);
    }
    painter.add(egui::Shape::closed_line(
        points,
        Stroke::new(1.0, with_opacity(Color32::WHITE, 0.8 * card.opacity)),
    ));
}

fn paint_pattern(painter: &egui::Painter, rect: Rect, card: &EffectCard, color: Color32) {
    let points = vec![
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ];
    for shape in pattern_fill_path(
        &points,
        card.seed,
        color,
        Color32::TRANSPARENT,
        card.scale,
        card.amount * 8.0 + 1.0,
    ) {
        painter.add(shape);
    }
}

fn paint_image_blur(ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, card: &EffectCard) {
    let source = egui::ColorImage::example();
    let blurred = blur_image(&source, card.radius as u32);
    let texture = ui.ctx().load_texture(
        format!("__effects_evaluator_blur_image_{}", card.id),
        blurred,
        egui::TextureOptions::LINEAR,
    );
    painter.image(
        texture.id(),
        rect,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
    ui.ctx()
        .data_mut(|data| data.insert_temp(ui.id().with(("blur-image", card.id)), texture));
}

fn paint_composite_report(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
) {
    let cpu_report = composite_layers_report(ui, blend_layers_for(rect, card));
    let gpu_report = composite_layers_gpu_report(ui, blend_layers_for(rect.shrink(4.0), card));
    paint_report_chip(
        painter,
        rect,
        cpu_report.is_exact() && gpu_report.is_exact(),
        cpu_report.issues.len() + gpu_report.issues.len(),
    );
    paint_report_details(
        painter,
        rect.shrink2(Vec2::new(4.0, 28.0)),
        "cpu",
        &cpu_report,
    );
    paint_report_details(
        painter,
        rect.shrink2(Vec2::new(4.0, 44.0)),
        "gpu",
        &gpu_report,
    );
}

fn paint_clipped_layer_report(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
) {
    let clip = hex_points(rect.shrink(3.0));
    let report = clipped_layers_gpu_report(ui, &clip, blend_layers_for(rect, card));
    painter.add(egui::Shape::closed_line(
        clip,
        Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 255, 255, 170)),
    ));
    paint_report_chip(painter, rect, report.is_exact(), report.issues.len());
    paint_report_details(painter, rect.shrink2(Vec2::new(4.0, 30.0)), "clip", &report);
}

fn paint_clip_mask_report(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
) {
    let mask = ClipMask::compound_even_odd(vec![
        vec![
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ],
        hex_points(rect.shrink(rect.width().min(rect.height()) * 0.18)),
    ]);
    let report = clipped_layers_mask_report(ui, &mask, blend_layers_for(rect, card));
    #[cfg(feature = "clip-mask")]
    egui_expressive::clipped_shape_cpu(ui, &hex_points(rect.shrink(8.0)), |child| {
        child.painter().text(
            rect.center_top() + Vec2::new(0.0, 18.0),
            egui::Align2::CENTER_CENTER,
            "clipped_shape_cpu",
            egui::FontId::proportional(10.5),
            Color32::from_rgb(225, 245, 255),
        );
    });
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "even-odd mask",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    paint_report_chip(painter, rect, report.is_exact(), report.issues.len());
    paint_report_details(painter, rect.shrink2(Vec2::new(4.0, 30.0)), "mask", &report);
}

fn paint_backdrop_report(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
) {
    let contract = evaluator_backdrop_contract(ui.ctx(), rect);
    egui_expressive::install_backdrop_snapshot_provider_with_source_contract(
        ui.ctx(),
        Arc::new(EvaluatorBackdropProvider),
        contract,
    );
    let _loaded_provider = egui_expressive::load_backdrop_snapshot_provider(ui.ctx());
    let _loaded_contract = egui_expressive::load_backdrop_capture_source_contract(ui.ctx());
    let report = app_provided_backdrop_blur_report(ui.ctx(), rect, card.radius);
    let (_provided_shape, provided_shape_report) =
        egui_expressive::app_provided_backdrop_blur_shape(ui, rect.shrink(3.0), card.radius);
    #[cfg(feature = "wgpu")]
    let gated_reports = wgpu_backdrop_reports(ui, rect, card);
    painter.add(radial_gradient_rect(
        rect,
        with_opacity(card.color, 0.52),
        with_opacity(card.secondary, 0.62),
        28,
    ));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "backdrop preflight",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    paint_report_chip(painter, rect, report.is_exact(), report.issues.len());
    paint_report_details(
        painter,
        rect.shrink2(Vec2::new(4.0, 30.0)),
        "backdrop",
        &report,
    );
    paint_report_details(
        painter,
        rect.shrink2(Vec2::new(4.0, 46.0)),
        "shape",
        &provided_shape_report,
    );
    #[cfg(feature = "wgpu")]
    for (idx, (label, report)) in gated_reports.iter().enumerate() {
        paint_report_details(
            painter,
            rect.shrink2(Vec2::new(4.0, 62.0 + idx as f32 * 18.0)),
            label,
            report,
        );
    }
}

fn evaluator_backdrop_contract(
    ctx: &egui::Context,
    rect: Rect,
) -> egui_expressive::BackdropCaptureSourceContract {
    let ppp = ctx.pixels_per_point().max(1.0);
    egui_expressive::BackdropCaptureSourceContract {
        source_id: egui_expressive::BackdropCaptureSourceId(1),
        provider_id: egui_expressive::BackdropCaptureProviderId(1),
        surface_token: egui_expressive::BackdropCaptureSurfaceToken(1),
        frame_token: egui_expressive::BackdropCaptureFrameToken(1),
        consent: egui_expressive::BackdropCaptureConsent::AppOwnedSurface,
        frame_freshness: egui_expressive::BackdropFrameFreshness::CurrentFrame,
        occlusion: egui_expressive::BackdropOcclusionState::Unoccluded,
        pixels_per_point: ppp,
        physical_size: [
            (rect.width() * ppp).ceil().max(1.0) as u32,
            (rect.height() * ppp).ceil().max(1.0) as u32,
        ],
        pixel_format: egui_expressive::BackdropCapturePixelFormat::Rgba8SrgbStraightAlpha,
    }
}

#[cfg(feature = "wgpu")]
fn wgpu_backdrop_reports(
    ui: &mut egui::Ui,
    rect: Rect,
    card: &EffectCard,
) -> Vec<(&'static str, RenderReport)> {
    let surface_id = egui_expressive::AppOwnedBackdropSurfaceId(1);
    let frame_id = egui_expressive::AppOwnedBackdropFrameId(1);
    let request = OffscreenRequest {
        feature: RenderFeature::BackdropBlur,
        width: rect.width().round().max(1.0) as u32,
        height: rect.height().round().max(1.0) as u32,
        requested_quality: RenderQuality::Exact,
    };
    let caps = RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
    let contract = evaluator_backdrop_contract(ui.ctx(), rect);
    let app_owned = egui_expressive::app_owned_offscreen_backdrop_blur_report(
        ui.ctx(),
        rect,
        card.radius,
        surface_id,
        frame_id,
    );
    let (_shape, app_owned_shape) = egui_expressive::app_owned_offscreen_backdrop_blur_shape(
        ui,
        rect.shrink(4.0),
        card.radius,
        surface_id,
        frame_id,
    );
    let blur_request = OffscreenRequest {
        feature: RenderFeature::Blur,
        ..request
    };
    let source_layer = egui_expressive::wgpu_source_layer_effect_report(
        &caps,
        blur_request,
        egui_expressive::GpuEffectSource::LibraryOwnedSourceLayer,
    );
    let app_snapshot =
        egui_expressive::wgpu_app_provided_backdrop_snapshot_report(&caps, request, contract);
    let host_framebuffer = egui_expressive::host_framebuffer_backdrop_report(&caps, request);
    let lifecycle = egui_expressive::wgpu_lifecycle_report(
        RenderFeature::Blur,
        egui_expressive::RenderBackendKind::EguiWgpuCallback,
        egui_expressive::WgpuLifecycleFailure::MissingRuntime,
        egui_expressive::WgpuLifecycleFailure::MissingRuntime.default_message(),
    );
    let _wgpu_types = [
        std::any::type_name::<egui_expressive::GpuCompositeCallback>(),
        std::any::type_name::<egui_expressive::GpuEffectsResources>(),
        std::any::type_name::<egui_expressive::GpuSourceLayerEffectCallback>(),
        std::any::type_name::<egui_expressive::AppOwnedOffscreenBackdropSource>(),
        std::any::type_name::<egui_expressive::SharedAppOwnedOffscreenBackdropSource>(),
    ];
    let _install_source_fn = egui_expressive::install_app_owned_offscreen_backdrop_source;
    let _loaded_source = egui_expressive::load_app_owned_offscreen_backdrop_source(ui.ctx());
    let _shader_id = egui_expressive::blend_mode_to_shader_id(&card.blend_mode);
    vec![
        ("owned", app_owned),
        ("shape", app_owned_shape),
        ("source", source_layer),
        ("snapshot", app_snapshot),
        ("host", host_framebuffer),
        ("life", lifecycle),
    ]
}

fn paint_render_capabilities(painter: &egui::Painter, rect: Rect) {
    let caps = RenderCapabilities::egui_native();
    let cpu = RenderCapabilities::cpu_offscreen(64 * 64);
    let callback = RenderCapabilities::egui_wgpu_callback(128 * 128);
    let offscreen = RenderCapabilities::wgpu_offscreen(256 * 256, true);
    let request = OffscreenRequest {
        feature: RenderFeature::BackdropBlur,
        width: rect.width().round().max(1.0) as u32,
        height: rect.height().round().max(1.0) as u32,
        requested_quality: RenderQuality::Exact,
    };
    let all_features = [
        RenderFeature::BlendGroup,
        RenderFeature::PolygonClip,
        RenderFeature::CompoundClip,
        RenderFeature::Blur,
        RenderFeature::BackdropBlur,
        RenderFeature::Shadow,
        RenderFeature::SceneEffect,
        RenderFeature::GradientMesh,
        RenderFeature::Mask,
        RenderFeature::TextureComposite,
        RenderFeature::TextShaping,
        RenderFeature::CssLayout,
    ];
    let feature_requests = all_features.map(|feature| OffscreenRequest { feature, ..request });
    let fitting_features = feature_requests
        .iter()
        .filter(|request| request.fits(&offscreen))
        .count();
    let rows = [
        ("backend", format!("{:?}", caps.backend)),
        (
            "cpu blend/clip",
            format!("{}/{}", cpu.exact_blend_groups, cpu.exact_polygon_clips),
        ),
        ("callback blur", callback.exact_large_blur.to_string()),
        ("wgpu backdrop", offscreen.exact_backdrop_blur.to_string()),
        (
            "features",
            format!("{} RenderFeature variants", all_features.len()),
        ),
        (
            "budget",
            format!("{} fit / {}", fitting_features, feature_requests.len()),
        ),
    ];
    for (idx, (label, value)) in rows.into_iter().enumerate() {
        let y = rect.top() + 10.0 + idx as f32 * 15.0;
        painter.text(
            Pos2::new(rect.left() + 8.0, y),
            egui::Align2::LEFT_TOP,
            format!("{label}: {value}"),
            egui::FontId::proportional(10.5),
            Color32::WHITE,
        );
    }
}

fn paint_draw_helpers(ui: &mut egui::Ui, painter: &egui::Painter, rect: Rect, card: &EffectCard) {
    let path = hex_points(rect.shrink(2.0));
    egui_expressive::clipped_shape(ui, &path, true, |child| {
        child.painter().rect_filled(
            rect.shrink(20.0),
            CornerRadius::same(8),
            with_opacity(card.secondary, 0.18),
        );
    });
    let _ = egui_expressive::clip_to(
        ui,
        ClipShape::RoundedRect(rect.shrink(14.0), CornerRadius::same(12)),
        |child| {
            child.label("clip_to");
        },
    );
    let _ = egui_expressive::clip_to_bounding_rect(
        ui,
        ClipShape::Circle(rect.center(), rect.height().min(rect.width()) * 0.28),
        |child| {
            child.label("clip_to_bounding_rect");
        },
    );
    egui_expressive::clipped_to_bounding_rect(ui, rect.shrink(18.0), 8.0, |child| {
        child.label("clipped_to_bounding_rect");
    });
    egui_expressive::clipped_rounded_rect(ui, rect.shrink(22.0), 10.0, |child| {
        child.label("clipped_rounded_rect");
    });
    egui_expressive::composite_layers(ui, blend_layers_for(rect.shrink(36.0), card));
    egui_expressive::composite_layers_gpu(ui, blend_layers_for(rect.shrink(42.0), card));
    let _ = egui_expressive::paint_image_from_path(
        ui,
        painter,
        rect.shrink(50.0),
        "__missing_effects_evaluator_asset__.png",
        "effects-evaluator-missing-image",
        Color32::WHITE,
    );
    egui_expressive::paint_placeholder_slot(
        painter,
        rect.shrink(56.0),
        Color32::from_rgba_unmultiplied(255, 0, 0, 32),
        Stroke::new(1.0, Color32::from_rgb(255, 110, 110)),
        "placeholder",
    );
    for (idx, icon_fn) in [
        egui_expressive::icon_play,
        egui_expressive::icon_stop,
        egui_expressive::icon_record,
        egui_expressive::icon_loop,
    ]
    .into_iter()
    .enumerate()
    {
        icon_fn(
            painter,
            rect.left_top() + Vec2::new(18.0 + idx as f32 * 18.0, 18.0),
            12.0,
            Color32::WHITE,
        );
    }
    let clipped = with_clip_path(
        &with_blend_mode(painter, card.blend_mode.clone()),
        path.clone(),
    );
    clipped.add(
        ShapeBuilder::rect(rect.shrink(8.0))
            .fill(with_opacity(card.color, 0.35))
            .stroke(Stroke::new(1.0, with_opacity(card.secondary, 0.8)))
            .rounding(CornerRadius::same(12))
            .build(),
    );
    clipped.add(ShapeBuilder::diamond(
        rect.center(),
        rect.height().min(rect.width()) * 0.46,
        with_opacity(card.secondary, 0.42),
        Stroke::new(1.0, Color32::WHITE),
    ));
    dashed_path(
        &clipped,
        &path,
        &RichStroke::dashed(1.4, Color32::WHITE, 7.0, 4.0),
    );
    clipped.add(egui_expressive::radial_gradient(
        rect.center(),
        rect.height().min(rect.width()) * 0.25,
        with_opacity(card.color, 0.42),
        Color32::TRANSPARENT,
        24,
    ));
    if let Some(shape) = egui_expressive::gradient_path_mesh_with_geometry(
        &path,
        &[(0.0, card.color), (1.0, card.secondary)],
        card.angle,
        true,
        Some(rect.center()),
        None,
        Some(rect.width().min(rect.height()) * 0.42),
    ) {
        clipped.add(shape);
    }
    if let Some(shape) = egui_expressive::gradient_path_mesh_with_transform(
        &path,
        &[(0.0, card.secondary), (1.0, card.color)],
        card.angle,
        false,
        egui_expressive::GradientPathGeometry {
            center: Some(rect.center()),
            focal_point: None,
            radius: None,
            transform: Some(Transform2D::translate(2.0, -2.0)),
        },
    ) {
        clipped.add(shape);
    }
    let rounded = egui_expressive::rounded_rect_path(rect.shrink(10.0), 12.0);
    clipped.add(egui::Shape::closed_line(
        rounded,
        Stroke::new(1.0, with_opacity(Color32::WHITE, 0.65)),
    ));
    let _ = egui_expressive::zstack_layers(
        ui,
        vec![
            Box::new(|ui: &mut egui::Ui| {
                ui.label("zstack layer A");
            }),
            Box::new(|ui: &mut egui::Ui| {
                ui.label("layer B");
            }),
        ],
    );
    let layered = LayeredPainter::from_ui(ui);
    layered.clipped(rect).text(
        rect.center_bottom() + Vec2::new(0.0, -14.0),
        egui::Align2::CENTER_CENTER,
        "clip + blend + builder + dash",
        egui::FontId::proportional(10.0),
        Color32::from_rgb(236, 240, 255),
    );
}

fn blend_layers_for(rect: Rect, card: &EffectCard) -> Vec<BlendLayer> {
    let left = Rect::from_center_size(
        rect.center() + Vec2::new(-rect.width() * 0.16, 0.0),
        Vec2::new(rect.width() * 0.55, rect.height() * 0.78),
    );
    let right = Rect::from_center_size(
        rect.center() + Vec2::new(rect.width() * 0.16, 0.0),
        Vec2::new(rect.width() * 0.55, rect.height() * 0.78),
    );
    vec![
        BlendLayer::new(vec![egui::Shape::rect_filled(
            left,
            CornerRadius::same(14),
            with_opacity(card.color, 0.72),
        )]),
        BlendLayer::new(vec![egui::Shape::rect_filled(
            right,
            CornerRadius::same(14),
            with_opacity(card.secondary, 0.78),
        )])
        .blend_mode(card.blend_mode.clone())
        .opacity(card.amount),
    ]
}

fn paint_report_chip(painter: &egui::Painter, rect: Rect, exact: bool, issue_count: usize) {
    let chip = Rect::from_min_size(
        rect.left_top() + Vec2::new(8.0, 8.0),
        Vec2::new(112.0, 20.0),
    );
    let color = if exact {
        Color32::from_rgb(95, 232, 166)
    } else {
        Color32::from_rgb(255, 194, 92)
    };
    painter.rect_filled(chip, CornerRadius::same(10), with_opacity(color, 0.24));
    painter.rect_stroke(
        chip,
        CornerRadius::same(10),
        Stroke::new(1.0, with_opacity(color, 0.85)),
        StrokeKind::Inside,
    );
    painter.text(
        chip.center(),
        egui::Align2::CENTER_CENTER,
        if exact {
            "report: exact".to_owned()
        } else {
            format!("report: {issue_count} issue")
        },
        egui::FontId::proportional(10.5),
        Color32::WHITE,
    );
}

fn paint_report_details(painter: &egui::Painter, rect: Rect, label: &str, report: &RenderReport) {
    let issue = report.issues.first();
    let lines = [
        format!(
            "{label}: {:?} {:?}->{:?}",
            report.backend, report.requested_quality, report.actual_quality
        ),
        issue
            .map(|issue| format!("{:?}/{:?}", issue.feature, issue.kind))
            .unwrap_or_else(|| "no issues".to_owned()),
        issue
            .map(|issue| issue.message.to_owned())
            .unwrap_or_else(|| "exact path".to_owned()),
    ];
    for (idx, line) in lines.into_iter().enumerate() {
        painter.text(
            rect.left_top() + Vec2::new(0.0, idx as f32 * 12.0),
            egui::Align2::LEFT_TOP,
            truncate_for_card(&line, 42),
            egui::FontId::monospace(8.5),
            Color32::from_rgb(226, 235, 248),
        );
    }
}

fn truncate_for_card(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_chars {
            out.push('…');
            return out;
        }
        out.push(ch);
    }
    out
}

fn paint_effect_overlay(
    painter: &egui::Painter,
    rect: Rect,
    card: &EffectCard,
    accent: Color32,
    secondary: Color32,
    time: f64,
) {
    match card.kind {
        EffectKind::InnerShadow => {
            for shape in soft_inner_shadow(rect.shrink(1.0), accent, card.radius, BlurQuality::High)
            {
                painter.add(shape);
            }
            for shape in inner_shadow(
                rect.shrink(1.0),
                with_opacity(accent, 0.55),
                (card.radius * 0.5).max(1.0),
            ) {
                painter.add(shape);
            }
        }
        EffectKind::InnerGlow => {
            for shape in soft_inner_shadow(rect.shrink(2.0), accent, card.radius, BlurQuality::High)
            {
                painter.add(shape);
            }
        }
        EffectKind::Noise => {
            for shape in noise_rect(
                rect.shrink(1.0),
                card.seed,
                card.scale,
                card.amount * card.opacity,
            ) {
                painter.add(shape);
            }
        }
        EffectKind::UnknownEffect => {
            painter.rect_stroke(
                rect.shrink(8.0),
                CornerRadius::same(12),
                Stroke::new(2.0, accent),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Unknown(String)\nunsupported",
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
        }
        EffectKind::Bevel => paint_bevel(painter, rect, accent, secondary, card),
        EffectKind::ScanLines => {
            for shape in scan_lines(rect.shrink(1.0), card.scale, card.amount) {
                painter.add(shape);
            }
        }
        EffectKind::DotMatrix => {
            for shape in dot_matrix(
                rect.shrink(2.0),
                card.scale,
                card.amount * 3.6 + 0.8,
                accent,
            ) {
                painter.add(shape);
            }
        }
        EffectKind::Vignette => {
            painter.add(vignette(
                rect.shrink(1.0),
                accent,
                card.radius.clamp(0.0, 1.0),
            ));
        }
        EffectKind::LiveEffect => {
            let sweep = ((time as f32 * 38.0) % rect.width()) + rect.left();
            painter.line_segment(
                [
                    Pos2::new(sweep, rect.top() + 8.0),
                    Pos2::new(sweep - 44.0, rect.bottom() - 8.0),
                ],
                Stroke::new(2.0, with_opacity(accent, 0.62)),
            );
        }
        EffectKind::DropShadow => {
            for shape in box_shadow(
                rect.shrink(11.0),
                with_opacity(accent, 0.25),
                card.radius * 0.3,
                0.0,
                ShadowOffset::new(card.offset.x * 0.15, card.offset.y * 0.15),
            ) {
                painter.add(shape);
            }
        }
        _ => {}
    }
}

fn paint_bevel(
    painter: &egui::Painter,
    rect: Rect,
    highlight: Color32,
    shadow: Color32,
    card: &EffectCard,
) {
    let width = 1.0 + card.amount * 6.0;
    let rect = rect.shrink(3.0);
    let light = Stroke::new(width, with_opacity(highlight, 0.84));
    let dark = Stroke::new(width, with_opacity(shadow, 0.86));
    let flip = card.angle.to_radians().sin() < 0.0;
    let (top_left, bottom_right) = if flip { (dark, light) } else { (light, dark) };
    painter.line_segment([rect.left_top(), rect.right_top()], top_left);
    painter.line_segment([rect.left_top(), rect.left_bottom()], top_left);
    painter.line_segment([rect.right_bottom(), rect.right_top()], bottom_right);
    painter.line_segment([rect.right_bottom(), rect.left_bottom()], bottom_right);
}

fn hex_points(rect: Rect) -> Vec<Pos2> {
    let c = rect.center();
    let rx = rect.width() * 0.48;
    let ry = rect.height() * 0.46;
    (0..6)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / 6.0 + std::f32::consts::FRAC_PI_6;
            Pos2::new(c.x + angle.cos() * rx, c.y + angle.sin() * ry)
        })
        .collect()
}

fn with_opacity(color: Color32, opacity: f32) -> Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * opacity.clamp(0.0, 1.0)).round() as u8)
}

fn uses_radius(kind: EffectKind) -> bool {
    matches!(
        kind,
        EffectKind::DropShadow
            | EffectKind::InnerShadow
            | EffectKind::OuterGlow
            | EffectKind::RectGlow
            | EffectKind::InnerGlow
            | EffectKind::GaussianBlur
            | EffectKind::ImageBlur
            | EffectKind::BackdropReport
            | EffectKind::Feather
            | EffectKind::Vignette
    )
}

fn uses_spread(kind: EffectKind) -> bool {
    matches!(
        kind,
        EffectKind::DropShadow
            | EffectKind::OuterGlow
            | EffectKind::RectGlow
            | EffectKind::GaussianBlur
    )
}

fn uses_offset(kind: EffectKind) -> bool {
    matches!(kind, EffectKind::DropShadow)
}

fn uses_amount(kind: EffectKind) -> bool {
    matches!(
        kind,
        EffectKind::Noise
            | EffectKind::Bevel
            | EffectKind::LiveEffect
            | EffectKind::ScanLines
            | EffectKind::DotMatrix
            | EffectKind::PatternFill
            | EffectKind::CompositeReport
            | EffectKind::ClippedLayerReport
            | EffectKind::ClipMaskReport
            | EffectKind::BackdropReport
            | EffectKind::RenderCapability
    )
}

fn uses_scale(kind: EffectKind) -> bool {
    matches!(
        kind,
        EffectKind::Noise | EffectKind::PatternFill | EffectKind::ScanLines | EffectKind::DotMatrix
    )
}

fn uses_angle(kind: EffectKind) -> bool {
    matches!(
        kind,
        EffectKind::LinearGradient
            | EffectKind::PathGradient
            | EffectKind::Bevel
            | EffectKind::Transform
    )
}

fn radius_label(kind: EffectKind) -> &'static str {
    match kind {
        EffectKind::Vignette => "strength",
        _ => "radius",
    }
}

fn scale_label(kind: EffectKind) -> &'static str {
    match kind {
        EffectKind::Noise => "cell size",
        EffectKind::PatternFill => "cell size",
        EffectKind::ScanLines => "line height",
        EffectKind::DotMatrix => "spacing",
        _ => "scale",
    }
}

fn fidelity_badge_ui(ui: &mut egui::Ui, fidelity: EffectFidelity) {
    egui::Frame::new()
        .fill(with_opacity(fidelity.color(), 0.18))
        .stroke(Stroke::new(1.0, with_opacity(fidelity.color(), 0.72)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("fidelity: {}", fidelity.label()))
                    .color(Color32::WHITE)
                    .size(11.0),
            );
        });
}

fn blend_mode_combo(ui: &mut egui::Ui, mode: &mut BlendMode) {
    const MODES: &[BlendMode] = &[
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::HardLight,
        BlendMode::SoftLight,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ];
    egui::ComboBox::from_label("blend mode")
        .selected_text(blend_mode_name(mode))
        .show_ui(ui, |ui| {
            for candidate in MODES {
                ui.selectable_value(mode, candidate.clone(), blend_mode_name(candidate));
            }
        });
}

fn blend_mode_name(mode: &BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "Normal",
        BlendMode::Multiply => "Multiply",
        BlendMode::Screen => "Screen",
        BlendMode::Overlay => "Overlay",
        BlendMode::Darken => "Darken",
        BlendMode::Lighten => "Lighten",
        BlendMode::ColorDodge => "Color dodge",
        BlendMode::ColorBurn => "Color burn",
        BlendMode::HardLight => "Hard light",
        BlendMode::SoftLight => "Soft light",
        BlendMode::Difference => "Difference",
        BlendMode::Exclusion => "Exclusion",
        BlendMode::Hue => "Hue",
        BlendMode::Saturation => "Saturation",
        BlendMode::Color => "Color",
        BlendMode::Luminosity => "Luminosity",
    }
}

fn install_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(6, 9, 16);
    visuals.window_fill = Color32::from_rgb(10, 14, 24);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 34, 54);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(39, 53, 84);
    visuals.selection.bg_fill = Color32::from_rgb(80, 128, 220);
    ctx.set_visuals(visuals);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1360.0, 860.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui_expressive Effect Evaluator",
        options,
        Box::new(|_cc| {
            #[cfg(feature = "wgpu")]
            if let Some(render_state) = _cc.wgpu_render_state.as_ref() {
                egui_expressive::init_gpu_effects(render_state);
                egui_expressive::init_gpu_effects_for_context(render_state, &_cc.egui_ctx);
                let _binding_report =
                    egui_expressive::bind_app_owned_offscreen_backdrop_source_for_context(
                        render_state,
                        &_cc.egui_ctx,
                        egui_expressive::AppOwnedBackdropSurfaceId(1),
                        egui_expressive::AppOwnedBackdropFrameId(1),
                    );
            }
            Ok(Box::new(EffectsEvaluatorApp::default()))
        }),
    )
}
