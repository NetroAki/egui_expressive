//! Primitive-first gallery showing reusable controls/displays and blend/report probes.
//!
//! Run with:
//! `cargo run --example primitive_gallery`

use eframe::egui;
use egui::{Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Vec2};
use egui_expressive::draw::{
    clipped_layers_mask_report, composite_layers_gpu_report, composite_layers_report, BlendLayer,
    ClipMask,
};
use egui_expressive::widgets::*;
use egui_expressive::{
    box_shadow, glow, linear_gradient_rect, soft_glow, soft_shadow, BlendMode, BlurQuality,
    CheckboxField, GradientDir, Icon, IconButton, M3Badge, M3Button, M3Card, M3Checkbox, M3Chip,
    M3CircularProgress, M3Dialog, M3Divider, M3DropdownMenu, M3Fab, M3LinearProgress, M3ListItem,
    M3NavItem, M3NavigationBar, M3NavigationRail, M3RadioButton, M3Slider, M3Snackbar,
    M3SnackbarState, M3Switch, M3TextField, M3Theme, M3Tooltip, M3TopAppBar, RenderReport,
    SelectField, SelectOption, ShadowOffset, SwitchField, TextAreaField, TextField, Theme,
};
const DEMO_FX_QUALITY: BlurQuality = BlurQuality::Fast;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GallerySectionKind {
    Effects,
    Controls,
    Data,
    Public,
    Overlay,
    Canvas,
    Blend,
    Coverage,
}

impl GallerySectionKind {
    const ALL: [Self; 8] = [
        Self::Effects,
        Self::Controls,
        Self::Data,
        Self::Public,
        Self::Overlay,
        Self::Canvas,
        Self::Blend,
        Self::Coverage,
    ];

    fn number(self) -> &'static str {
        match self {
            Self::Effects => "01",
            Self::Controls => "02",
            Self::Data => "03",
            Self::Public => "04",
            Self::Overlay => "05",
            Self::Canvas => "06",
            Self::Blend => "07",
            Self::Coverage => "08",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Effects => "Effects lab",
            Self::Controls => "Controls",
            Self::Data => "Data/shell",
            Self::Public => "M3 API",
            Self::Overlay => "Overlay",
            Self::Canvas => "Canvas",
            Self::Blend => "Blend",
            Self::Coverage => "Coverage",
        }
    }
}

struct PrimitiveGalleryApp {
    active_section: GallerySectionKind,
    gain: f64,
    pan: f64,
    fader: f64,
    drag_value: f64,
    range_start: f64,
    range_end: f64,
    xy_x: f64,
    xy_y: f64,
    steps: Vec<Vec<StepCell>>,
    search: String,
    color: Color32,
    dot: DotState,
    tool_active: bool,
    mute: DotState,
    solo: DotState,
    record: DotState,
    send_a: f64,
    send_b: f64,
    transport_playing: bool,
    toolbar_items: Vec<ToolbarItem>,
    menus: Vec<MenuDef>,
    menu_activated: Option<String>,
    tab: usize,
    tree_nodes: Vec<TreeNode>,
    tree_selected: Option<String>,
    tree_dragged: Option<String>,
    data_model: DataGridModel,
    data_state: DataGridState,
    property_model: PropertyGridModel,
    tree_table_model: TreeTableModel,
    tree_table_state: TreeTableState,
    command_query: String,
    command_selected: usize,
    command_activated: Option<String>,
    command_items: Vec<CommandPaletteItem>,
    split_fraction: f32,
    dock_zones: Vec<DockDropZone>,
    floating_state: FloatingPanelState,
    toasts: Vec<Toast>,
    show_modal: bool,
    progress: f32,
    bool_steps: Vec<Vec<bool>>,
    reorder_items: Vec<String>,
    mixer_sections: Vec<MixerStripSection>,
    m3_value: f32,
    m3_switch: bool,
    m3_dialog_open: bool,
    m3_snackbar: M3SnackbarState,
    form_text: String,
    form_notes: String,
    form_choice: String,
    parts: Vec<DesignerPart>,
    notes: Vec<PianoRollNote>,
    color_wheel: ColorWheelState,
    plugin_items: Vec<PluginManagerItem>,
    plugin_query: String,
    generator_slots: Vec<GeneratorSlot>,
    system_metrics: Vec<SystemMetric>,
    controller_link: ControllerLinkState,
    timeline_clip_start: f32,
    timeline_clip_length: f32,
    ruler_beats: f32,
    spectrum: Vec<f32>,
    waveform: Vec<f32>,
    spectrogram: Vec<Vec<f32>>,
    bars: Vec<f32>,
    blend_mode: BlendMode,
    glow_radius: f32,
    blur_radius: f32,
    glass_opacity: f32,
    motion_amount: f32,
}

fn default_floating_panel_state() -> FloatingPanelState {
    FloatingPanelState {
        pos: Pos2::new(950.0, 112.0),
        size: Vec2::new(320.0, 176.0),
        docked: false,
    }
}

impl Default for PrimitiveGalleryApp {
    fn default() -> Self {
        let columns = vec![
            DataColumn::new("primitive", "Primitive"),
            DataColumn::new("state", "State"),
        ];
        let rows = vec![
            DataRow::new(
                "knob",
                vec![DataCell::new("Knob"), DataCell::new("interactive")],
            ),
            DataRow::new(
                "blend",
                vec![DataCell::new("BlendLayer"), DataCell::new("report")],
            ),
        ];
        Self {
            active_section: GallerySectionKind::Effects,
            gain: 0.65,
            pan: 0.0,
            fader: -6.0,
            drag_value: 12.0,
            range_start: 1.0,
            range_end: 4.0,
            xy_x: 0.5,
            xy_y: 0.5,
            steps: vec![vec![StepCell::default(); 16]; 3],
            search: String::new(),
            color: Color32::from_rgb(90, 170, 255),
            dot: DotState::On,
            tool_active: true,
            mute: DotState::Off,
            solo: DotState::Off,
            record: DotState::On,
            send_a: 0.35,
            send_b: 0.18,
            transport_playing: true,
            toolbar_items: vec![
                ToolbarItem::button("select", "Select")
                    .icon('⌖')
                    .active(true),
                ToolbarItem::button("paint", "Paint").icon('✎'),
                ToolbarItem {
                    id: "space".to_owned(),
                    label: String::new(),
                    kind: ToolbarItemKind::Spacer,
                    icon: None,
                    active: false,
                    enabled: true,
                    width: 28.0,
                },
                ToolbarItem {
                    id: "overflow".to_owned(),
                    label: "More".to_owned(),
                    kind: ToolbarItemKind::Overflow,
                    icon: Some('⋯'),
                    active: false,
                    enabled: true,
                    width: 64.0,
                },
            ],
            menus: vec![MenuDef {
                label: "Primitives".to_owned(),
                items: vec![
                    MenuItemDef::action("reset", "Reset values")
                        .icon('↺')
                        .shortcut("R"),
                    MenuItemDef::separator(),
                    MenuItemDef::action("blend", "Blend preview").checked(true),
                    MenuItemDef::action("disabled", "Disabled item").disabled(true),
                ],
            }],
            menu_activated: None,
            tab: 0,
            tree_nodes: vec![TreeNode::new("root", "Widget families")
                .icon('▣')
                .children(vec![
                    TreeNode::new("controls", "Controls").icon('●'),
                    TreeNode::new("overlays", "Overlays").icon('▤'),
                ])],
            tree_selected: None,
            tree_dragged: None,
            data_model: DataGridModel::new(columns.clone(), rows),
            data_state: DataGridState::default(),
            property_model: PropertyGridModel::new(vec![
                PropertyGridEntry::new("Blend mode", "interactive", "Render")
                    .description("Composite report uses BlendLayer."),
                PropertyGridEntry::new("Primitive count", "all widget families", "Coverage")
                    .group("Gallery"),
            ]),
            tree_table_model: TreeTableModel::new(
                columns,
                vec![TreeTableNode::new("visual", "Visual primitives")
                    .with_cells(vec![DataCell::new("Controls"), DataCell::new("Ready")])
                    .with_children(vec![TreeTableNode::new("blend", "Blend reports")
                        .with_cells(vec![
                            DataCell::new("Composite"),
                            DataCell::new("Exact/approx"),
                        ])])],
            ),
            tree_table_state: TreeTableState::default(),
            command_query: String::new(),
            command_selected: 0,
            command_activated: None,
            command_items: vec![
                CommandPaletteItem {
                    id: "show_blend".to_owned(),
                    label: "Show blend report".to_owned(),
                    hint: "Render/composite".to_owned(),
                },
                CommandPaletteItem {
                    id: "show_all".to_owned(),
                    label: "Show all primitives".to_owned(),
                    hint: "Coverage".to_owned(),
                },
            ],
            split_fraction: 0.42,
            dock_zones: vec![DockDropZone::new(
                DockZone::Left,
                Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(120.0, 80.0)),
            )],
            floating_state: default_floating_panel_state(),
            toasts: vec![Toast::new(
                "Primitive gallery · drag the floating panel header to move it.",
                4.0,
            )],
            show_modal: false,
            progress: 0.58,
            bool_steps: vec![
                vec![true, false, true, false, true, false, false, true],
                vec![false, true, false, true, false, true, false, false],
                vec![true, true, false, false, true, false, true, false],
            ],
            reorder_items: vec![
                "DragReorder one".to_owned(),
                "DragReorder two".to_owned(),
                "DragReorder three".to_owned(),
            ],
            mixer_sections: vec![
                MixerStripSection {
                    id: "input".to_owned(),
                    label: "Input".to_owned(),
                    visible: true,
                },
                MixerStripSection {
                    id: "sends".to_owned(),
                    label: "Sends".to_owned(),
                    visible: true,
                },
                MixerStripSection {
                    id: "meter".to_owned(),
                    label: "Meter".to_owned(),
                    visible: true,
                },
            ],
            m3_value: 0.42,
            m3_switch: true,
            m3_dialog_open: false,
            m3_snackbar: M3SnackbarState {
                message: "M3Snackbar".to_owned(),
                action_label: Some("Action".to_owned()),
                visible: true,
                show_until: None,
            },
            form_text: "primitive value".to_owned(),
            form_notes: "Type here to exercise TextAreaField.".to_owned(),
            form_choice: "blend".to_owned(),
            parts: vec![
                DesignerPart {
                    id: "osc".to_owned(),
                    pos: Pos2::new(70.0, 52.0),
                },
                DesignerPart {
                    id: "filter".to_owned(),
                    pos: Pos2::new(150.0, 88.0),
                },
            ],
            notes: vec![
                PianoRollNote {
                    pitch: 60,
                    beat: 0.0,
                    length: 1.0,
                    velocity: 0.8,
                    selected: true,
                },
                PianoRollNote {
                    pitch: 64,
                    beat: 1.0,
                    length: 1.5,
                    velocity: 0.6,
                    selected: false,
                },
            ],
            color_wheel: ColorWheelState {
                hue: 0.58,
                saturation: 0.72,
                value: 0.95,
            },
            plugin_items: vec![PluginManagerItem {
                id: "primitive-fx".to_owned(),
                name: "Primitive FX".to_owned(),
                vendor: "egui_expressive".to_owned(),
                category: "Utility".to_owned(),
                enabled: true,
                favorite: true,
            }],
            plugin_query: String::new(),
            generator_slots: vec![
                GeneratorSlot {
                    name: "Noise".to_owned(),
                    enabled: true,
                    macro_value: 0.4,
                },
                GeneratorSlot {
                    name: "Ramp".to_owned(),
                    enabled: false,
                    macro_value: 0.72,
                },
            ],
            system_metrics: vec![
                SystemMetric {
                    label: "Render".to_owned(),
                    value: 0.72,
                    warning: 0.88,
                },
                SystemMetric {
                    label: "Blend".to_owned(),
                    value: 0.91,
                    warning: 0.95,
                },
            ],
            controller_link: ControllerLinkState {
                target: "Cutoff".to_owned(),
                source: "MIDI CC74".to_owned(),
                automation_enabled: true,
                learn_mode: false,
            },
            timeline_clip_start: 0.25,
            timeline_clip_length: 2.2,
            ruler_beats: 8.0,
            spectrum: (0..32)
                .map(|i| ((i as f32 * 0.37).sin() * 0.5 + 0.5).clamp(0.0, 1.0))
                .collect(),
            waveform: (0..96).map(|i| (i as f32 * 0.16).sin()).collect(),
            spectrogram: (0..12)
                .map(|row| {
                    (0..24)
                        .map(|col| ((row + col) as f32 * 0.23).sin() * 0.5 + 0.5)
                        .collect()
                })
                .collect(),
            bars: vec![0.2, 0.8, 0.45, 0.7, 0.35, 0.9],
            blend_mode: BlendMode::Overlay,
            glow_radius: 20.0,
            blur_radius: 14.0,
            glass_opacity: 0.58,
            motion_amount: 0.62,
        }
    }
}

impl PrimitiveGalleryApp {
    fn new(ctx: &egui::Context) -> Self {
        let theme = Theme::dark();
        theme.store(ctx);

        let m3_theme = M3Theme::from_seed(theme.colors.primary, theme.is_dark);
        m3_theme.store(ctx);
        m3_theme.apply_to_egui(ctx);

        let mut style = (*ctx.global_style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        ctx.set_global_style(style);

        Self::default()
    }
}

impl eframe::App for PrimitiveGalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Ok(section) = std::env::var("PRIMITIVE_GALLERY_SECTION") {
            capture_banner(ui, section.as_str());
            ui.set_width(1360.0);
            ui.set_max_width(1360.0);
            self.render_named_section(ui, section.as_str());
            ToastLayer::new(&mut self.toasts).show(ui.ctx());
            return;
        }

        egui::ScrollArea::vertical()
            .max_width(1360.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(1360.0);
                ui.set_max_width(1360.0);
                hero_banner(ui);
                ui.add_space(14.0);
                section_nav(ui, &mut self.active_section);
                ui.add_space(14.0);
                self.render_active_section(ui);
            });
        ToastLayer::new(&mut self.toasts).show(ui.ctx());
    }
}

impl PrimitiveGalleryApp {
    fn render_active_section(&mut self, ui: &mut egui::Ui) {
        match self.active_section {
            GallerySectionKind::Effects => self.effects_interface_section(ui),
            GallerySectionKind::Controls => self.controls_section(ui),
            GallerySectionKind::Data => self.data_and_shell_section(ui),
            GallerySectionKind::Public => self.public_api_primitives_section(ui),
            GallerySectionKind::Overlay => self.overlay_layout_section(ui),
            GallerySectionKind::Canvas => self.canvas_display_section(ui),
            GallerySectionKind::Blend => self.blend_report_section(ui),
            GallerySectionKind::Coverage => self.coverage_inventory(ui),
        }
    }

    fn render_named_section(&mut self, ui: &mut egui::Ui, section: &str) {
        match section {
            "effects" => self.effects_interface_section(ui),
            "controls" => self.controls_section(ui),
            "data" => self.data_and_shell_section(ui),
            "public" => self.public_api_primitives_section(ui),
            "overlay" => self.overlay_layout_section(ui),
            "canvas" => self.canvas_display_section(ui),
            "blend" => self.blend_report_section(ui),
            "coverage" => self.coverage_inventory(ui),
            _ => {
                ui.label("Unknown section; rendering all sections.");
                self.effects_interface_section(ui);
                self.controls_section(ui);
                self.data_and_shell_section(ui);
                self.public_api_primitives_section(ui);
                self.overlay_layout_section(ui);
                self.canvas_display_section(ui);
                self.blend_report_section(ui);
                self.coverage_inventory(ui);
            }
        }
    }

    fn effects_interface_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(168, 126, 255);
        gallery_section(
            ui,
            "01",
            "Modern effects & interface lab",
            "This is the first-stop demo for what blur, glow, glass, reactive hover states, and blend modes actually feel like in a composed interface.",
            accent,
            |ui| {
                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "Glassmorphism dashboard mockup",
                        "Move the pointer over the cards and chips: glow, elevation, and highlight intensity react so the affordances are visible instead of hidden.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                gallery_pill(ui, "hover reactive", Color32::from_rgb(94, 234, 212));
                                gallery_pill(ui, "soft glow", accent);
                                gallery_pill(ui, "glass panels", Color32::from_rgb(88, 166, 255));
                                gallery_pill(ui, "fast path", Color32::from_rgb(255, 170, 96));
                            });
                            ui.add_space(8.0);
                            ui.add(
                                egui::Slider::new(&mut self.glow_radius, 4.0..=32.0)
                                    .text("glow radius"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.glass_opacity, 0.24..=0.86)
                                    .text("glass opacity"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.motion_amount, 0.0..=1.0)
                                    .text("reaction amount"),
                            );
                            ui.add_space(8.0);
                            paint_modern_interface_lab(
                                ui,
                                self.glow_radius,
                                self.glass_opacity,
                                self.motion_amount,
                            );
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "Blur, glow & shadow primitives",
                        "Small isolated tiles show the raw visual vocabulary before it is composed into the interface mockup.",
                        accent,
                        |ui| {
                            ui.add(
                                egui::Slider::new(&mut self.blur_radius, 3.0..=24.0)
                                    .text("blur/shadow radius"),
                            );
                            ui.add_space(8.0);
                            ui.horizontal_wrapped(|ui| {
                                effect_tile(
                                    ui,
                                    "Soft shadow",
                                    "depth/elevation",
                                    Color32::from_rgb(88, 166, 255),
                                    |painter, rect, hover| {
                                        for shape in soft_shadow(
                                            rect.shrink(18.0),
                                            Color32::from_rgba_unmultiplied(40, 95, 255, 150),
                                            self.blur_radius + if hover { 8.0 } else { 0.0 },
                                            2.0,
                                            ShadowOffset::new(0.0, 10.0),
                                            DEMO_FX_QUALITY,
                                        ) {
                                            painter.add(shape);
                                        }
                                    },
                                );
                                effect_tile(
                                    ui,
                                    "Soft glow",
                                    "neon focus",
                                    accent,
                                    |painter, rect, hover| {
                                        for shape in soft_glow(
                                            rect.shrink(24.0),
                                            Color32::from_rgba_unmultiplied(170, 110, 255, 190),
                                            self.blur_radius + if hover { 10.0 } else { 0.0 },
                                            DEMO_FX_QUALITY,
                                        ) {
                                            painter.add(shape);
                                        }
                                    },
                                );
                                effect_tile(
                                    ui,
                                    "Rect glow",
                                    "fast helper",
                                    Color32::from_rgb(94, 234, 212),
                                    |painter, rect, hover| {
                                        for shape in glow(
                                            rect.shrink(22.0),
                                            Color32::from_rgba_unmultiplied(94, 234, 212, 170),
                                            self.blur_radius * if hover { 1.25 } else { 0.85 },
                                        ) {
                                            painter.add(shape);
                                        }
                                    },
                                );
                                effect_tile(
                                    ui,
                                    "Glass blur",
                                    "backdrop proxy",
                                    Color32::from_rgb(255, 170, 96),
                                    |painter, rect, hover| {
                                        let glass = rect.shrink(18.0);
                                        for shape in box_shadow(
                                            glass,
                                            Color32::from_rgba_unmultiplied(0, 0, 0, 130),
                                            self.blur_radius,
                                            0.0,
                                            ShadowOffset::new(0.0, 8.0),
                                        ) {
                                            painter.add(shape);
                                        }
                                        let alpha = if hover { 150 } else { 112 };
                                        painter.rect_filled(
                                            glass,
                                            CornerRadius::same(18),
                                            Color32::from_rgba_unmultiplied(230, 245, 255, alpha),
                                        );
                                    },
                                );
                            });
                        },
                    );
                });

                ui.add_space(10.0);
                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "Reactive state row",
                        "Hover, press, and compare the controls: every state has a visible color, elevation, and label change.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                reactive_demo_chip(ui, "Hover me", Color32::from_rgb(88, 166, 255));
                                reactive_demo_chip(ui, "Press me", Color32::from_rgb(94, 234, 212));
                                reactive_demo_chip(ui, "Danger", Color32::from_rgb(255, 92, 120));
                                reactive_demo_chip(ui, "Blend", Color32::from_rgb(255, 170, 96));
                            });
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("The point: an interface should tell you where to click before you click it.")
                                    .color(muted_text_color()),
                            );
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "Blend modes in plain English",
                        "Four common modes are shown as UI decisions: darken content, brighten light, boost contrast, or show difference/error.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for (mode, label) in [
                                    (BlendMode::Multiply, "Multiply = darken"),
                                    (BlendMode::Screen, "Screen = lighten"),
                                    (BlendMode::Overlay, "Overlay = contrast"),
                                    (BlendMode::Difference, "Difference = compare"),
                                ] {
                                    blend_meaning_tile(ui, mode, label);
                                }
                            });
                        },
                    );
                });
            },
        );
    }

    fn controls_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(88, 166, 255);
        gallery_section(
            ui,
            "03",
            "Controls & tactile input",
            "Knobs, drags, transport, and sequence widgets are grouped by the gesture they teach.",
            accent,
            |ui| {
                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "Continuous controls",
                        "Drag or scroll these first: they prove range, precision, reset, and two-axis input behavior.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.add(
                                    Knob::new(&mut self.gain, 0.0..=1.0)
                                        .label("GAIN")
                                        .bipolar(false)
                                        .wheel_step(0.01)
                                        .value_popup(true),
                                );
                                ui.add(
                                    Slider::new(&mut self.pan, -1.0..=1.0)
                                        .label("PAN")
                                        .marks(vec![-1.0, 0.0, 1.0])
                                        .value_popup(true),
                                );
                                ui.add(
                                    Fader::new(&mut self.fader, -18.0..=6.0)
                                        .size(Vec2::new(32.0, 110.0)),
                                );
                                ui.add(
                                    DragNumber::new(&mut self.drag_value, -48.0..=48.0)
                                        .label("DragNumber")
                                        .reset_value(0.0),
                                );
                                ui.add(
                                    VerticalDrag::new(&mut self.drag_value, -48.0..=48.0)
                                        .step(0.5)
                                        .reset_value(0.0),
                                );
                                ui.add(
                                    RangeSlider::new(
                                        &mut self.range_start,
                                        &mut self.range_end,
                                        0.0..=8.0,
                                    )
                                    .size(Vec2::new(180.0, 28.0)),
                                );
                                ui.add(
                                    XYPad::new(
                                        &mut self.xy_x,
                                        &mut self.xy_y,
                                        0.0..=1.0,
                                        0.0..=1.0,
                                    )
                                    .label("XY"),
                                );
                                ui.add(ColorSwatch::new(&mut self.color).size(Vec2::splat(28.0)));
                            });
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "Buttons & immediate feedback",
                        "These controls show state right away: toggle, meter, toolbar selection, and transport intent all remain visible.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.add(ToggleDot::new(&mut self.dot).size(18.0));
                                ui.add(
                                    ToolButton::new("tool.brush", "Brush")
                                        .active(&mut self.tool_active),
                                );
                                ui.add(
                                    Meter::new(self.gain as f32)
                                        .size(Vec2::new(12.0, 100.0))
                                        .segments(8),
                                );
                                for kind in [
                                    TransportKind::Play,
                                    TransportKind::Stop,
                                    TransportKind::Record,
                                    TransportKind::Metronome,
                                    TransportKind::Loop,
                                ] {
                                    ui.add(
                                        TransportButton::new(kind, &mut self.transport_playing)
                                            .size(28.0),
                                    );
                                }
                            });
                            ui.add_space(8.0);
                            ui.label(RichText::new("Toolbar strip").strong());
                            ui.add(ToolbarStrip::new(&mut self.toolbar_items).dragged(Some(1)));
                        },
                    );
                });

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Sequencing, reorder & channel strip",
                    "Move rows, flip steps, search the sample state, then inspect a fuller channel-strip cluster.",
                    accent,
                    |ui| {
                        ui.label(RichText::new("Drag reorder").strong());
                        DragReorder::new(&mut self.reorder_items, "primitive-drag-reorder").show(
                            ui,
                            |ui, index, item| {
                                ui.label(format!("{index}:"));
                                ui.text_edit_singleline(item);
                            },
                        );
                        ui.add_space(8.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.add(StepCellGrid::new(&mut self.steps, 3, 16).active_col(4));
                            ui.add(
                                StepGrid::new(&mut self.bool_steps, 3, 8)
                                    .cell_size(Vec2::new(22.0, 22.0))
                                    .size(Vec2::new(190.0, 76.0))
                                    .active_col(2)
                                    .row_colors(vec![
                                        Color32::from_rgb(42, 56, 78),
                                        Color32::from_rgb(48, 68, 54),
                                        Color32::from_rgb(72, 52, 58),
                                    ]),
                            );
                        });
                        ui.add_space(8.0);
                        ui.add(SearchField::new(&mut self.search).hint("Search primitives…"));
                        let channel_level = self.gain as f32;
                        ui.add(
                            ChannelStrip::new(
                                &mut self.gain,
                                &mut self.pan,
                                &mut self.mute,
                                channel_level,
                            )
                            .solo(&mut self.solo)
                            .record(&mut self.record)
                            .send("A", &mut self.send_a)
                            .send("B", &mut self.send_b)
                            .name("ChannelStrip"),
                        );
                    },
                );
            },
        );
    }

    fn data_and_shell_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(179, 136, 255);
        let menu_state = self.menu_activated.as_deref().unwrap_or("idle").to_owned();
        let tree_state = self.tree_selected.as_deref().unwrap_or("none").to_owned();
        gallery_section(
            ui,
            "02",
            "Navigation, hierarchy & data",
            "App-shell primitives now show where you are, what is selected, and how data surfaces relate to that state.",
            accent,
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    gallery_pill(ui, &format!("menu {menu_state}"), accent);
                    gallery_pill(ui, &format!("tree {tree_state}"), Color32::from_rgb(112, 197, 255));
                    gallery_pill(ui, "data surfaces stay deterministic", Color32::from_rgb(255, 170, 96));
                });
                ui.add_space(10.0);

                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "Shell navigation",
                        "Menus, tabs, breadcrumbs, sidebar, and status bar work together so the example reads like an app instead of a pile of widgets.",
                        accent,
                        |ui| {
                            ui.add(TopMenuBar::new(&self.menus).activated(&mut self.menu_activated));
                            ui.add(TabBar::new(
                                &mut self.tab,
                                vec![
                                    "Controls".to_owned(),
                                    "Data".to_owned(),
                                    "Blend".to_owned(),
                                ],
                            ));
                            let crumbs = [
                                BreadcrumbItem::new("root", "Primitives"),
                                BreadcrumbItem::new("blend", "Blend"),
                            ];
                            ui.add(Breadcrumbs::new(&crumbs));
                            let sidebar = [
                                SidebarItem::new("controls", "Controls").icon('●'),
                                SidebarItem::new("render", "Render").icon('◈'),
                            ];
                            let mut selected_sidebar = "controls".to_owned();
                            ui.add(SidebarNav::new(&mut selected_sidebar, &sidebar));
                            ui.add(StatusBar::new(&[
                                StatusBarItem::new("interactive"),
                                StatusBarItem::new("blend").value("reports"),
                            ]));
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "Hierarchy selection",
                        "TreeView exposes structure and drag state without hiding what is currently selected.",
                        accent,
                        |ui| {
                            ui.add(
                                TreeView::new(&mut self.tree_nodes)
                                    .selected_id(&mut self.tree_selected)
                                    .dragged_id(&mut self.tree_dragged),
                            );
                            ui.add_space(8.0);
                            ui.small(
                                RichText::new(
                                    "Look for the live tree selection chip above while clicking nodes here.",
                                )
                                .color(muted_text_color()),
                            );
                        },
                    );
                });

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Structured data surfaces",
                    "The fixed-width demo rectangles keep the tables readable and repeatable in capture mode.",
                    accent,
                    |ui| {
                        ui.columns(2, |columns| {
                            let (table_rect, _) = columns[0]
                                .allocate_exact_size(Vec2::new(520.0, 180.0), egui::Sense::hover());
                            let mut table_ui =
                                columns[0].new_child(egui::UiBuilder::new().max_rect(table_rect));
                            table_ui.set_clip_rect(table_rect);
                            table_ui.add(DataTable::new(&self.data_model, &mut self.data_state));
                            columns[0].add_space(8.0);
                            columns[0].add(PropertyGrid::new(&self.property_model));

                            let (tree_table_rect, _) = columns[1]
                                .allocate_exact_size(Vec2::new(520.0, 180.0), egui::Sense::hover());
                            let mut tree_table_ui =
                                columns[1].new_child(egui::UiBuilder::new().max_rect(tree_table_rect));
                            tree_table_ui.set_clip_rect(tree_table_rect);
                            tree_table_ui.add(TreeTable::new(
                                &self.tree_table_model,
                                &mut self.tree_table_state,
                            ));
                        });
                    },
                );
            },
        );
    }

    fn public_api_primitives_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(255, 183, 77);
        gallery_section(
            ui,
            "04",
            "Broader public APIs",
            "Material 3, forms, icons, and transient feedback now read as a guided catalog instead of a flat dump.",
            accent,
            |ui| {
                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "Material actions",
                        "Buttons, chips, badges, switches, icons, and FABs show the action vocabulary at a glance.",
                        accent,
                        |ui| {
                            M3TopAppBar::new("M3TopAppBar")
                                .center_aligned()
                                .navigation_icon('☰')
                                .action('★', "favorite")
                                .action('⋯', "more")
                                .show(ui);
                            ui.add_space(8.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.add(M3Button::new("M3 filled").icon('✓'));
                                ui.add(M3Button::new("M3 tonal").tonal());
                                ui.add(M3Button::new("M3 outline").outlined());
                                ui.add(M3Button::new("M3 text").text_only());
                                ui.add(M3Button::new("M3 elevated").elevated());
                                ui.add(M3Chip::new("M3Chip").filter().selected(true).icon('◆'));
                                ui.add(M3Chip::new("input").input().trailing_icon('×'));
                                ui.add(M3Chip::new("suggest").suggestion());
                                ui.add(M3Switch::new("m3-switch", &mut self.m3_switch));
                                ui.add(M3Checkbox::new(&mut self.m3_switch));
                                ui.add(M3RadioButton::new(self.m3_switch));
                                ui.add(Icon::new('★').size(24.0).color(Color32::YELLOW));
                                let icon_response =
                                    ui.add(IconButton::new('★').active(self.m3_switch));
                                M3Tooltip::new("M3Tooltip on IconButton")
                                    .show_on_hover(ui, &icon_response);
                                ui.add(M3Badge::dot());
                                ui.add(M3Badge::count(7));
                                ui.add(M3Fab::new('+').small());
                                ui.add(M3Fab::new('✦').extended("M3Fab").secondary());
                                ui.add(M3Fab::new('◎').large().tertiary());
                            });
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "Progress, sliders & card surfaces",
                        "This cluster keeps feedback and input density readable while proving the broader M3 surface set.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.add(M3CircularProgress::new(self.m3_value).size(32.0));
                                ui.add(
                                    M3CircularProgress::indeterminate("m3-circular-indeterminate")
                                        .size(32.0),
                                );
                            });
                            ui.add(M3LinearProgress::new(self.m3_value).height(8.0));
                            ui.add(
                                M3LinearProgress::indeterminate("m3-linear-indeterminate")
                                    .height(6.0),
                            );
                            ui.add(
                                M3Slider::new(&mut self.m3_value, 0.0..=1.0)
                                    .steps(16)
                                    .show_value(true),
                            );
                            ui.add(
                                M3TextField::new(
                                    "m3-text-field",
                                    "M3TextField",
                                    &mut self.form_text,
                                )
                                .outlined()
                                .hint("material input")
                                .leading_icon('⌕')
                                .trailing_icon('✓'),
                            );
                            M3Card::new().outlined().width(360.0).show(ui, |ui| {
                                ui.label("M3Card with M3ListItem, divider, and dropdown");
                                ui.add(
                                    M3ListItem::new("M3ListItem")
                                        .supporting("supporting text")
                                        .trailing_supporting("trail")
                                        .leading_icon('●')
                                        .trailing_icon('›')
                                        .selected(true),
                                );
                                ui.add(M3Divider::horizontal().inset(8.0).thickness(1.0));
                                ui.add(
                                    M3DropdownMenu::new(
                                        "m3-dropdown",
                                        "M3DropdownMenu",
                                        &mut self.tab,
                                    )
                                    .items(vec!["Controls", "Blend", "Forms"]),
                                );
                            });
                        },
                    );
                });

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Navigation, dialogs & plain form fields",
                    "Use the trigger buttons below for transient feedback, then compare them with the plain public field primitives underneath.",
                    accent,
                    |ui| {
                        M3NavigationBar::new(&mut self.tab)
                            .item(M3NavItem::new("Controls", '●').badge(1))
                            .item(M3NavItem::new("Blend", '◈'))
                            .item(M3NavItem::new("Forms", '□'))
                            .height(72.0)
                            .show(ui);

                        let (rail_rect, _) =
                            ui.allocate_exact_size(Vec2::new(140.0, 220.0), egui::Sense::hover());
                        let mut rail_ui = ui.new_child(egui::UiBuilder::new().max_rect(rail_rect));
                        M3NavigationRail::new(&mut self.tab)
                            .header(|ui| {
                                ui.add(M3Fab::new('+').surface());
                            })
                            .item(M3NavItem::new("Rail", '▌'))
                            .item(M3NavItem::new("Test", '✓').badge(3))
                            .width(112.0)
                            .show(&mut rail_ui);

                        ui.add_space(8.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Open M3Dialog").clicked() {
                                self.m3_dialog_open = true;
                            }
                            if ui.button("Show M3Snackbar").clicked() {
                                self.m3_snackbar.show_with_action(
                                    "M3Snackbar from primitive_gallery",
                                    "Action",
                                );
                            }
                        });

                        let _confirmed = M3Dialog::new(&mut self.m3_dialog_open)
                            .title("M3Dialog")
                            .body(
                                "Dialog is opt-in and closeable so it does not block gallery testing.",
                            )
                            .confirm("Close")
                            .cancel("Cancel")
                            .icon('ⓘ')
                            .show(ui.ctx());
                        M3Snackbar::new(&mut self.m3_snackbar)
                            .duration(8.0)
                            .show(ui.ctx());

                        TextField::new("TextField", &mut self.form_text)
                            .hint("form text")
                            .show(ui);
                        TextAreaField::new("TextAreaField", &mut self.form_notes)
                            .rows(2)
                            .show(ui);
                        CheckboxField::new("CheckboxField", &mut self.m3_switch).show(ui);
                        SwitchField::new("SwitchField", &mut self.m3_switch).show(ui);
                        SelectField::new("SelectField", &mut self.form_choice)
                            .options([
                                SelectOption::new("controls".to_owned(), "Controls"),
                                SelectOption::new("blend".to_owned(), "Blend"),
                                SelectOption::new("forms".to_owned(), "Forms"),
                            ])
                            .show(ui);
                    },
                );
            },
        );
    }

    fn overlay_layout_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(94, 234, 212);
        let command_state = self
            .command_activated
            .as_deref()
            .unwrap_or("idle")
            .to_owned();
        gallery_section(
            ui,
            "05",
            "Overlay, docking & layout affordances",
            "This section now names the important hit areas: drag the floating title bar, resize from the corner, and inspect dock/split feedback in place.",
            accent,
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    gallery_pill(ui, "grab floating title bar", accent);
                    gallery_pill(ui, "resize from ↘ corner", Color32::from_rgb(124, 195, 255));
                    gallery_pill(ui, &format!("command {command_state}"), Color32::from_rgb(255, 170, 96));
                });
                ui.add_space(10.0);

                let panel_mode = if self.floating_state.docked {
                    "Docked demo state"
                } else {
                    "Floating demo state"
                };
                let panel_pos = format!(
                    "{:.0}, {:.0}",
                    self.floating_state.pos.x, self.floating_state.pos.y
                );
                let panel_size = format!(
                    "{:.0} × {:.0}",
                    self.floating_state.size.x, self.floating_state.size.y
                );
                ui.horizontal_wrapped(|ui| {
                    gallery_stat(ui, "panel mode", panel_mode, accent);
                    gallery_stat(
                        ui,
                        "position",
                        &panel_pos,
                        Color32::from_rgb(124, 195, 255),
                    );
                    gallery_stat(
                        ui,
                        "size",
                        &panel_size,
                        Color32::from_rgb(255, 170, 96),
                    );
                    if ui
                        .add(M3Button::new("Reset floating panel").outlined().icon('↺'))
                        .clicked()
                    {
                        self.floating_state = default_floating_panel_state();
                        self.toasts
                            .push(Toast::new("Floating panel reset to demo position.", 2.5));
                    }
                });

                ui.add_space(10.0);
                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "Commands, context & modal surfaces",
                        "These surfaces are grouped together so the example reads like a stack of transient UI patterns instead of unrelated widgets.",
                        accent,
                        |ui| {
                            CollapsePanel::new("collapse-primitive", "CollapsePanel").show(
                                ui,
                                |ui| {
                                    ControlGroup::new().title("ControlGroup").show(ui, |ui| {
                                        ui.label("Grouped primitive content")
                                    });
                                },
                            );
                            ui.add(
                                CommandPalette::new(&mut self.command_query, &self.command_items)
                                    .selected(&mut self.command_selected)
                                    .activated(&mut self.command_activated),
                            );
                            ContextMenuBuilder::new()
                                .item("Context item", || {})
                                .separator()
                                .submenu(
                                    "Nested",
                                    vec![ContextMenuEntry::Item {
                                        label: "Child".to_owned(),
                                        shortcut: Some("C".to_owned()),
                                        icon: Some('›'),
                                        disabled: false,
                                        checked: false,
                                        callback: Box::new(|| {}),
                                    }],
                                )
                                .show(ui);
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(
                                    "ModalOverlay is opt-in here so it never blocks the gallery by surprise.",
                                )
                                .color(muted_text_color()),
                            );
                            if ui.button("Open ModalOverlay").clicked() {
                                self.show_modal = true;
                            }
                            ui.add(ProgressOverlay {
                                progress: &mut self.progress,
                            });
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "Docking proof surfaces",
                        "The split handle, drop zone, and recovery model stay visible together so the layout story is understandable.",
                        accent,
                        |ui| {
                            ui.allocate_ui(Vec2::new(ui.available_width(), 100.0), |ui| {
                                ResizableSplit::new(
                                    "primitive-split",
                                    &mut self.split_fraction,
                                    SplitAxis::Horizontal,
                                )
                                .show(
                                    ui,
                                    |ui| {
                                        ui.label("ResizableSplit A");
                                    },
                                    |ui| {
                                        ui.label("ResizableSplit B");
                                    },
                                );
                            });
                            ui.allocate_ui(Vec2::new(ui.available_width(), 96.0), |ui| {
                                ui.add(
                                    DockOverlay::new(&self.dock_zones)
                                        .pointer(Pos2::new(20.0, 20.0)),
                                )
                            });
                            let mut panel = DockPanel::new(
                                "dock-panel",
                                "DockPanel",
                                DockPlacement::floating(
                                    Pos2::new(10.0, 10.0),
                                    Vec2::new(120.0, 70.0),
                                ),
                            );
                            panel.recover_placement(DockZone::Left);
                            let dock_response = ui.add_sized(
                                [260.0, 38.0],
                                egui::Button::new(format!(
                                    "DockPanel {} at {:?}",
                                    panel.title(),
                                    panel.placement()
                                ))
                                .sense(egui::Sense::click_and_drag()),
                            );
                            ui.small(format!(
                                "DockPanel model/recovery hit area: hovered={} dragged={} closable={}",
                                dock_response.hovered(),
                                dock_response.dragged(),
                                panel.closable()
                            ));
                        },
                    );
                });

                if self.show_modal {
                    let mut modal_close_requested = false;
                    ModalOverlay::new()
                        .title("ModalOverlay")
                        .click_outside_to_close(true)
                        .close_requested(&mut modal_close_requested)
                        .show(ui, |ui| {
                            ui.label(
                                "ModalOverlay; click outside or Close to return to primitives.",
                            );
                        });
                    if modal_close_requested {
                        self.show_modal = false;
                    }
                }

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Floating panel live demo",
                    "The widget itself floats above the gallery. Grab the highlighted header to move it, use Dock/Undock to flip state, and drag the bottom-right corner to resize.",
                    accent,
                    |ui| {
                        ui.label(
                            RichText::new(
                                "The demo panel stays detached on purpose so you can validate its affordance without leaving this section.",
                            )
                            .color(muted_text_color()),
                        );
                    },
                );

                let floating_pos = format!(
                    "x {:.0} · y {:.0}",
                    self.floating_state.pos.x, self.floating_state.pos.y
                );
                let floating_size = format!(
                    "{:.0} × {:.0}",
                    self.floating_state.size.x, self.floating_state.size.y
                );
                let floating_mode = if self.floating_state.docked {
                    "Docked demo state"
                } else {
                    "Floating demo state"
                };
                FloatingPanel::new("Floating inspector")
                    .state(&mut self.floating_state)
                    .show(ui, |ui| {
                        ui.label("Grab the highlighted title bar to drag this panel.");
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            gallery_pill(ui, floating_mode, accent);
                            gallery_pill(ui, &floating_pos, Color32::from_rgb(124, 195, 255));
                            gallery_pill(ui, &floating_size, Color32::from_rgb(255, 170, 96));
                        });
                        ui.add_space(8.0);
                        ui.small(
                            RichText::new(
                                "Use this to validate drag, resize, and dock affordances without guessing where the interaction lives.",
                            )
                            .color(muted_text_color()),
                        );
                    });
            },
        );
    }

    fn canvas_display_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(120, 220, 120);
        gallery_section(
            ui,
            "06",
            "Canvas, displays & creative editor primitives",
            "Display widgets, editor surfaces, and hit-area proofs are separated so the visual story is easier to scan.",
            accent,
            |ui| {
                gallery_card(
                    ui,
                    "Signal displays",
                    "These fixed sample datasets make waveform, spectrum, spectrogram, and bar-graph rendering easy to compare at a glance.",
                    accent,
                    |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add(Waveform {
                                size: Vec2::new(180.0, 72.0),
                                ..Waveform::new(&self.waveform)
                            });
                            ui.add(WaveformDisplay {
                                size: Vec2::new(180.0, 72.0),
                                filled: false,
                                ..WaveformDisplay::new(&self.waveform)
                            });
                            ui.add(SpectrumDisplay {
                                size: Vec2::new(160.0, 72.0),
                                ..SpectrumDisplay::new(&self.spectrum)
                            });
                            ui.add(SpectrogramDisplay {
                                size: Vec2::new(160.0, 72.0),
                                ..SpectrogramDisplay::new(&self.spectrogram)
                            });
                            ui.add(MiniBarGraph {
                                size: Vec2::new(120.0, 72.0),
                                ..MiniBarGraph::new(&self.bars)
                            });
                        });
                    },
                );

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Editor surfaces",
                    "Color, piano, plugin, generator, controller, and mixer primitives all stay visible without collapsing into one endless block.",
                    accent,
                    |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add(ColorWheel::new(&mut self.color_wheel));
                            ui.add(PianoRoll::new(&mut self.notes));
                            ui.add(PianoRollView::new_view(&self.notes));
                        });
                        ui.add_space(8.0);
                        ui.add(PluginManager {
                            query: &mut self.plugin_query,
                            plugins: &mut self.plugin_items,
                        });
                        ui.add(GeneratorOverlay {
                            title: "GeneratorOverlay",
                            slots: &mut self.generator_slots,
                        });
                        ui.add(SystemMonitor {
                            metrics: &self.system_metrics,
                        });
                        ui.add(ControllerLinkOverlay {
                            state: &mut self.controller_link,
                        });
                        ui.add(MixerStripDesigner {
                            sections: &mut self.mixer_sections,
                        });
                    },
                );

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Canvas hit areas",
                    "DesignerCanvas, routing cable, and timeline helpers include live hover/drag readouts so the interaction model is explicit.",
                    accent,
                    |ui| {
                        ui.allocate_ui(Vec2::new(ui.available_width(), 120.0), |ui| {
                            ui.add(DesignerCanvas {
                                parts: &mut self.parts,
                            })
                        });

                        let (cable_rect, cable_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width().min(260.0), 78.0),
                            egui::Sense::click_and_drag(),
                        );
                        let cable = RoutingCable::new(
                            cable_rect.left_top() + Vec2::new(24.0, 24.0),
                            cable_rect.left_top() + Vec2::new(160.0, 62.0),
                        );
                        ui.painter().add(egui::Shape::line(
                            cable.points.to_vec(),
                            egui::Stroke::new(
                                if cable_response.dragged() { 4.0 } else { 2.0 },
                                Color32::LIGHT_BLUE,
                            ),
                        ));
                        ui.small(format!(
                            "RoutingCable interactive hit area: hovered={} dragged={}",
                            cable_response.hovered(),
                            cable_response.dragged()
                        ));

                        let (rect, timeline_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), 140.0),
                            egui::Sense::click_and_drag(),
                        );
                        let rect = rect.shrink(8.0);
                        let grid = GridCanvas::new(4, 32.0, 18.0).subdivisions(4);
                        grid.paint_grid(
                            ui.painter(),
                            rect,
                            0,
                            8,
                            4,
                            [Color32::from_gray(45), Color32::from_gray(80)],
                        );
                        NoteRect::new("note", 1.0, 2.0, 1).paint(ui.painter(), &grid, rect.min);
                        let mut loop_region = LoopRegion {
                            start: 0.5,
                            end: 3.5,
                        };
                        loop_region.snap();
                        ui.painter().rect_stroke(
                            Rect::from_min_max(
                                rect.min + Vec2::new(loop_region.start * 32.0, 4.0),
                                rect.min + Vec2::new(
                                    loop_region.end * 32.0,
                                    rect.height() - 4.0,
                                ),
                            ),
                            4.0,
                            egui::Stroke::new(
                                1.0,
                                Color32::from_rgba_unmultiplied(80, 160, 255, 150),
                            ),
                            egui::StrokeKind::Outside,
                        );
                        ui.add(TimelineClip {
                            start: &mut self.timeline_clip_start,
                            length: &mut self.timeline_clip_length,
                            kind: ClipKind::Audio,
                        });
                        let fade = FadeHandle {
                            side: FadeSide::In,
                            amount: 0.25,
                        };
                        ui.painter().rect_stroke(
                            fade.handle_rect(rect),
                            0.0,
                            egui::Stroke::new(1.0, Color32::WHITE),
                            egui::StrokeKind::Outside,
                        );
                        ui.add(Ruler {
                            beats: &mut self.ruler_beats,
                        });
                        AutomationCurve::new(vec![
                            AutomationPoint {
                                beat: 0.0,
                                value: 0.2,
                                segment: AutomationSegment::Linear,
                            },
                            AutomationPoint {
                                beat: 2.0,
                                value: 0.8,
                                segment: AutomationSegment::Smooth,
                            },
                            AutomationPoint {
                                beat: 4.0,
                                value: 0.4,
                                segment: AutomationSegment::Linear,
                            },
                        ])
                        .paint(ui.painter(), &grid, rect, Color32::LIGHT_BLUE);
                        ui.small(format!(
                            "GridCanvas/NoteRect/LoopRegion/FadeHandle/AutomationCurve hit area: hovered={} dragged={}",
                            timeline_response.hovered(),
                            timeline_response.dragged()
                        ));
                    },
                );
            },
        );
    }

    fn blend_report_section(&mut self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(255, 120, 96);
        gallery_section(
            ui,
            "07",
            "Blend & render correctness",
            "Exact CPU composition, GPU callback paths, and clip-mask fallbacks are split into readable proof blocks.",
            accent,
            |ui| {
                gallery_card(
                    ui,
                    "Primary blend preview",
                    "Pick a mode, then inspect the exact offscreen CPU report directly beneath the sample composition.",
                    accent,
                    |ui| {
                        blend_mode_combo(ui, &mut self.blend_mode);
                        let rect = Rect::from_min_size(
                            ui.cursor().min + Vec2::new(10.0, 8.0),
                            Vec2::new(220.0, 116.0),
                        );
                        let report = composite_layers_report(
                            ui,
                            sample_blend_layers(rect, self.blend_mode.clone()),
                        );
                        report_ui(ui, &report);
                        ui.allocate_space(Vec2::new(240.0, 128.0));
                    },
                );

                ui.add_space(10.0);
                ui.columns(2, |columns| {
                    gallery_card(
                        &mut columns[0],
                        "All blend modes",
                        "Every BlendMode is rendered through the CPU exact offscreen path so coverage remains comprehensive.",
                        accent,
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for mode in all_blend_modes() {
                                    ui.vertical(|ui| {
                                        ui.small(format!("{:?}", mode));
                                        let mini_rect = Rect::from_min_size(
                                            ui.cursor().min + Vec2::new(4.0, 2.0),
                                            Vec2::new(56.0, 36.0),
                                        );
                                        let report = composite_layers_report(
                                            ui,
                                            sample_blend_layers(mini_rect, mode.clone()),
                                        );
                                        ui.allocate_space(Vec2::new(64.0, 44.0));
                                        ui.small(format!("issues={}", report.issues.len()));
                                    });
                                }
                            });
                        },
                    );

                    gallery_card(
                        &mut columns[1],
                        "GPU callback & clip-mask paths",
                        "These reports explain when the render path stays exact versus falling back for GPU or compound mask handling.",
                        accent,
                        |ui| {
                            let gpu_rect = Rect::from_min_size(
                                ui.cursor().min + Vec2::new(10.0, 8.0),
                                Vec2::new(96.0, 64.0),
                            );
                            let gpu_report = composite_layers_gpu_report(
                                ui,
                                sample_blend_layers(gpu_rect, self.blend_mode.clone()),
                            );
                            report_ui(ui, &gpu_report);
                            ui.allocate_space(Vec2::new(112.0, 78.0));

                            let mask_rect = Rect::from_min_size(
                                ui.cursor().min + Vec2::new(10.0, 8.0),
                                Vec2::new(120.0, 80.0),
                            );
                            let outer = vec![
                                mask_rect.left_top(),
                                mask_rect.right_top(),
                                mask_rect.right_bottom(),
                                mask_rect.left_bottom(),
                            ];
                            let inner = vec![
                                mask_rect.center() + Vec2::new(-24.0, -16.0),
                                mask_rect.center() + Vec2::new(24.0, -16.0),
                                mask_rect.center() + Vec2::new(24.0, 16.0),
                                mask_rect.center() + Vec2::new(-24.0, 16.0),
                            ];
                            let mask = ClipMask::compound_even_odd(vec![outer, inner]);
                            let mask_report = clipped_layers_mask_report(
                                ui,
                                &mask,
                                sample_blend_layers(mask_rect, self.blend_mode.clone()),
                            );
                            report_ui(ui, &mask_report);
                            ui.allocate_space(Vec2::new(140.0, 94.0));
                        },
                    );
                });
            },
        );
    }

    fn coverage_inventory(&self, ui: &mut egui::Ui) {
        let accent = Color32::from_rgb(158, 168, 190);
        gallery_section(
            ui,
            "08",
            "Coverage inventory & capture map",
            "The final matrix doubles as the deterministic checklist for what this example proves and how to capture it section-by-section.",
            accent,
            |ui| {
                gallery_card(
                    ui,
                    "Capture-friendly reference",
                    "Keep the existing PRIMITIVE_GALLERY_SECTION contract; use these labels to render one deterministic section at 1360 pt width.",
                    accent,
                    |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for section in [
                                "controls",
                                "data",
                                "public",
                                "overlay",
                                "canvas",
                                "blend",
                                "coverage",
                            ] {
                                gallery_pill(ui, section, accent);
                            }
                        });
                        ui.add_space(8.0);
                        ui.monospace(
                            "PRIMITIVE_GALLERY_SECTION=overlay cargo run --example primitive_gallery",
                        );
                    },
                );

                ui.add_space(10.0);
                gallery_card(
                    ui,
                    "Public family coverage",
                    "Each row ties a public family or helper surface to the live interaction, render, or blend evidence above.",
                    accent,
                    |ui| {
                        let names = [
                            "app_shell",
                            "channel_strip",
                            "controls",
                            "data",
                            "daw_editors/editor_tools",
                            "designer",
                            "displays",
                            "dock",
                            "drag",
                            "faders",
                            "grid",
                            "knobs",
                            "menus",
                            "meters",
                            "overlays",
                            "tabs",
                            "timeline",
                            "toolbar",
                            "transport",
                            "tree",
                            "BlendLayer/RenderReport",
                            "forms fields",
                            "Material 3 controls",
                            "icons",
                            "GPU callback/fallback blend report",
                            "compound clip/mask blend report",
                        ];
                        ui.label(names.join(" · "));
                        egui::Grid::new("primitive_coverage_matrix")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("Public family/API");
                                ui.strong("Live surface in this example");
                                ui.strong("Interaction / render / blend evidence");
                                ui.end_row();
                                for (family, surface, evidence) in primitive_coverage_rows() {
                                    ui.label(*family);
                                    ui.label(*surface);
                                    ui.label(*evidence);
                                    ui.end_row();
                                }
                            });
                    },
                );
            },
        );
    }
}

fn hero_banner(ui: &mut egui::Ui) {
    let accent = Color32::from_rgb(88, 166, 255);
    egui::Frame::new()
        .fill(Color32::from_rgb(10, 14, 24))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.65)))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(egui::Margin::symmetric(20, 18))
        .show(ui, |ui| {
            ui.label(RichText::new("PRIMITIVE GALLERY").strong().color(accent));
            ui.heading("Modern egui_expressive effects, interactions & primitives");
            ui.label(
                RichText::new(
                    "Start with the effects lab: it shows glass, glow, blur-like depth, reactive affordances, and blend modes as a composed interface before the raw primitive inventory.",
                )
                .color(muted_text_color()),
            );
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                gallery_pill(ui, "Drag the floating panel header", accent);
                gallery_pill(ui, "Try effects first", Color32::from_rgb(168, 126, 255));
                gallery_pill(ui, "Resize from the ↘ corner", Color32::from_rgb(94, 234, 212));
                gallery_pill(ui, "Capture mode stays deterministic", Color32::from_rgb(255, 170, 96));
                gallery_pill(ui, "8 labeled sections", Color32::from_rgb(179, 136, 255));
            });
            ui.add_space(10.0);
            ui.small(
                RichText::new(
                    "Tip: PRIMITIVE_GALLERY_SECTION=effects|controls|data|public|overlay|canvas|blend|coverage renders one section at 1360 pt width.",
                )
                .color(muted_text_color()),
            );
        });
}

fn section_nav(ui: &mut egui::Ui, active: &mut GallerySectionKind) {
    egui::Frame::new()
        .fill(Color32::from_rgb(9, 13, 22))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(120, 175, 255, 62),
        ))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(egui::Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Render one section at a time for base-egui responsiveness")
                        .strong()
                        .color(Color32::from_rgb(214, 225, 245)),
                );
                ui.separator();
                for section in GallerySectionKind::ALL {
                    let selected = *active == section;
                    let label = format!("{}  {}", section.number(), section.label());
                    if ui
                        .selectable_label(selected, RichText::new(label).strong())
                        .on_hover_text("Switch section without rendering the whole gallery at once")
                        .clicked()
                    {
                        *active = section;
                    }
                }
            });
        });
}

fn capture_banner(ui: &mut egui::Ui, section: &str) {
    let accent = Color32::from_rgb(88, 166, 255);
    egui::Frame::new()
        .fill(Color32::from_rgb(12, 16, 27))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.55)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                gallery_pill(ui, "Section capture mode", accent);
                ui.label(RichText::new(section).strong().size(16.0));
            });
            ui.small(
                RichText::new(
                    "Only the requested named section is rendered below so screenshots remain deterministic.",
                )
                .color(muted_text_color()),
            );
        });
    ui.add_space(12.0);
}

fn gallery_section(
    ui: &mut egui::Ui,
    number: &str,
    title: &str,
    summary: &str,
    accent: Color32,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(Color32::from_rgb(14, 19, 31))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.55)))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(egui::Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                gallery_pill(ui, number, accent);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).strong().size(20.0));
                    ui.label(RichText::new(summary).color(muted_text_color()));
                });
            });
            ui.add_space(12.0);
            add(ui);
        });
    ui.add_space(12.0);
}

fn gallery_card(
    ui: &mut egui::Ui,
    title: &str,
    hint: &str,
    accent: Color32,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::new()
        .fill(Color32::from_rgb(19, 25, 39))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.35)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).strong().size(16.0));
            ui.add_space(4.0);
            ui.label(RichText::new(hint).color(muted_text_color()));
            ui.add_space(10.0);
            add(ui);
        });
}

fn gallery_pill(ui: &mut egui::Ui, label: &str, accent: Color32) {
    egui::Frame::new()
        .fill(accent.linear_multiply(0.18))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.65)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(RichText::new(label).strong().color(accent));
        });
}

fn gallery_stat(ui: &mut egui::Ui, label: &str, value: &str, accent: Color32) {
    egui::Frame::new()
        .fill(accent.linear_multiply(0.16))
        .stroke(Stroke::new(1.0, accent.linear_multiply(0.5)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.small(RichText::new(label.to_uppercase()).strong().color(accent));
            ui.label(RichText::new(value).strong());
        });
}

fn paint_modern_interface_lab(
    ui: &mut egui::Ui,
    glow_radius: f32,
    glass_opacity: f32,
    reaction_amount: f32,
) {
    let width = ui.available_width().clamp(360.0, 620.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 342.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let pointer = ui.input(|input| input.pointer.hover_pos());
    let panel_hovered = response.hovered();
    let reaction = if panel_hovered { reaction_amount } else { 0.0 };

    painter.add(linear_gradient_rect(
        rect,
        &[
            (0.0, Color32::from_rgb(12, 18, 34)),
            (0.45, Color32::from_rgb(32, 22, 74)),
            (1.0, Color32::from_rgb(5, 12, 22)),
        ],
        GradientDir::Angle(135.0),
    ));
    painter.circle_filled(
        rect.left_top() + Vec2::new(88.0, 72.0),
        62.0 + reaction * 10.0,
        Color32::from_rgba_unmultiplied(88, 166, 255, 84),
    );
    painter.circle_filled(
        rect.right_top() + Vec2::new(-112.0, 92.0),
        78.0 + reaction * 12.0,
        Color32::from_rgba_unmultiplied(168, 126, 255, 72),
    );
    painter.circle_filled(
        rect.left_bottom() + Vec2::new(230.0, -64.0),
        58.0,
        Color32::from_rgba_unmultiplied(94, 234, 212, 50),
    );

    let glass = rect.shrink2(Vec2::new(30.0, 28.0));
    for shape in soft_glow(
        glass,
        Color32::from_rgba_unmultiplied(168, 126, 255, 128),
        glow_radius + reaction * 10.0,
        DEMO_FX_QUALITY,
    ) {
        painter.add(shape);
    }
    for shape in soft_shadow(
        glass,
        Color32::from_rgba_unmultiplied(0, 0, 0, 180),
        28.0,
        2.0,
        ShadowOffset::new(0.0, 18.0),
        DEMO_FX_QUALITY,
    ) {
        painter.add(shape);
    }
    let glass_alpha = (glass_opacity.clamp(0.2, 0.9) * 210.0) as u8;
    painter.rect_filled(
        glass,
        CornerRadius::same(26),
        Color32::from_rgba_unmultiplied(20, 28, 48, glass_alpha),
    );
    painter.rect_stroke(
        glass,
        CornerRadius::same(26),
        Stroke::new(
            1.0 + reaction,
            Color32::from_rgba_unmultiplied(210, 225, 255, 92),
        ),
        egui::StrokeKind::Outside,
    );

    painter.text(
        glass.left_top() + Vec2::new(22.0, 20.0),
        egui::Align2::LEFT_TOP,
        "NEUTRA FX CONSOLE",
        egui::FontId::proportional(12.0),
        Color32::from_rgb(160, 178, 212),
    );
    painter.text(
        glass.left_top() + Vec2::new(22.0, 44.0),
        egui::Align2::LEFT_TOP,
        "Modern surfaces need visible states",
        egui::FontId::proportional(22.0),
        Color32::from_rgb(248, 250, 255),
    );
    painter.text(
        glass.left_top() + Vec2::new(22.0, 78.0),
        egui::Align2::LEFT_TOP,
        "Blur/glow/depth are useful only when they explain focus, layering, and what can be touched.",
        egui::FontId::proportional(13.0),
        Color32::from_rgb(178, 190, 214),
    );

    let metric_top = glass.left_top() + Vec2::new(22.0, 122.0);
    paint_metric_card(
        &painter,
        Rect::from_min_size(metric_top, Vec2::new(132.0, 76.0)),
        "Glow",
        format!("{glow_radius:.0}px"),
        Color32::from_rgb(168, 126, 255),
        pointer,
    );
    paint_metric_card(
        &painter,
        Rect::from_min_size(metric_top + Vec2::new(146.0, 0.0), Vec2::new(132.0, 76.0)),
        "Glass",
        format!("{:.0}%", glass_opacity * 100.0),
        Color32::from_rgb(88, 166, 255),
        pointer,
    );
    paint_metric_card(
        &painter,
        Rect::from_min_size(metric_top + Vec2::new(292.0, 0.0), Vec2::new(132.0, 76.0)),
        "React",
        format!("{:.0}%", reaction_amount * 100.0),
        Color32::from_rgb(94, 234, 212),
        pointer,
    );

    for (index, (label, color)) in [
        ("Preview", Color32::from_rgb(88, 166, 255)),
        ("Compare", Color32::from_rgb(255, 170, 96)),
        ("Export", Color32::from_rgb(94, 234, 212)),
    ]
    .iter()
    .enumerate()
    {
        let button = Rect::from_min_size(
            glass.left_bottom() + Vec2::new(22.0 + index as f32 * 118.0, -56.0),
            Vec2::new(102.0, 34.0),
        );
        let hovered = pointer.is_some_and(|pos| button.contains(pos));
        let fill_alpha = if hovered { 178 } else { 92 };
        for shape in glow(
            button.shrink(4.0),
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90),
            if hovered { glow_radius * 0.55 } else { 8.0 },
        ) {
            painter.add(shape);
        }
        painter.rect_filled(
            button,
            CornerRadius::same(15),
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), fill_alpha),
        );
        painter.text(
            button.center(),
            egui::Align2::CENTER_CENTER,
            *label,
            egui::FontId::proportional(13.0),
            Color32::from_rgb(248, 250, 255),
        );
    }
}

fn paint_metric_card(
    painter: &egui::Painter,
    rect: Rect,
    label: &str,
    value: String,
    accent: Color32,
    pointer: Option<Pos2>,
) {
    let hovered = pointer.is_some_and(|pos| rect.contains(pos));
    for shape in box_shadow(
        rect,
        Color32::from_rgba_unmultiplied(0, 0, 0, 120),
        if hovered { 18.0 } else { 10.0 },
        0.0,
        ShadowOffset::new(0.0, if hovered { 8.0 } else { 4.0 }),
    ) {
        painter.add(shape);
    }
    painter.rect_filled(
        rect,
        CornerRadius::same(18),
        Color32::from_rgba_unmultiplied(255, 255, 255, if hovered { 46 } else { 28 }),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(18),
        Stroke::new(
            1.0,
            accent.linear_multiply(if hovered { 0.95 } else { 0.55 }),
        ),
        egui::StrokeKind::Outside,
    );
    painter.text(
        rect.left_top() + Vec2::new(14.0, 12.0),
        egui::Align2::LEFT_TOP,
        label.to_uppercase(),
        egui::FontId::proportional(10.0),
        accent,
    );
    painter.text(
        rect.left_bottom() + Vec2::new(14.0, -18.0),
        egui::Align2::LEFT_BOTTOM,
        value,
        egui::FontId::proportional(22.0),
        Color32::from_rgb(246, 248, 255),
    );
}

fn effect_tile(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    accent: Color32,
    paint: impl FnOnce(&egui::Painter, Rect, bool),
) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(154.0, 132.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let hovered = response.hovered();
    painter.rect_filled(rect, CornerRadius::same(18), Color32::from_rgb(10, 14, 24));
    paint(&painter, rect, hovered);
    let chip = rect.shrink2(Vec2::new(14.0, 16.0));
    painter.rect_filled(
        chip,
        CornerRadius::same(16),
        Color32::from_rgba_unmultiplied(18, 24, 38, if hovered { 220 } else { 184 }),
    );
    painter.rect_stroke(
        chip,
        CornerRadius::same(16),
        Stroke::new(
            1.0,
            accent.linear_multiply(if hovered { 1.0 } else { 0.55 }),
        ),
        egui::StrokeKind::Outside,
    );
    painter.text(
        chip.center_top() + Vec2::new(0.0, 28.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(14.0),
        Color32::from_rgb(248, 250, 255),
    );
    painter.text(
        chip.center_bottom() + Vec2::new(0.0, -26.0),
        egui::Align2::CENTER_CENTER,
        subtitle,
        egui::FontId::proportional(11.0),
        Color32::from_rgb(170, 182, 204),
    );
}

fn reactive_demo_chip(ui: &mut egui::Ui, label: &str, accent: Color32) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(138.0, 50.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    let state = if pressed {
        "pressed"
    } else if hovered {
        "hover"
    } else {
        "idle"
    };
    let lift = if pressed {
        0.0
    } else if hovered {
        8.0
    } else {
        3.0
    };
    for shape in soft_shadow(
        rect.shrink(4.0),
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 92),
        if hovered { 22.0 } else { 9.0 },
        0.0,
        ShadowOffset::new(0.0, lift),
        DEMO_FX_QUALITY,
    ) {
        painter.add(shape);
    }
    painter.rect_filled(
        rect,
        CornerRadius::same(18),
        Color32::from_rgba_unmultiplied(
            accent.r(),
            accent.g(),
            accent.b(),
            if pressed {
                205
            } else if hovered {
                155
            } else {
                88
            },
        ),
    );
    painter.text(
        rect.center_top() + Vec2::new(0.0, 16.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        Color32::from_rgb(250, 252, 255),
    );
    painter.text(
        rect.center_bottom() + Vec2::new(0.0, -10.0),
        egui::Align2::CENTER_CENTER,
        state,
        egui::FontId::proportional(11.0),
        Color32::from_rgb(235, 241, 255),
    );
}

fn blend_meaning_tile(ui: &mut egui::Ui, mode: BlendMode, label: &str) {
    let (rect, _response) = ui.allocate_exact_size(Vec2::new(166.0, 112.0), egui::Sense::hover());
    let sample = Rect::from_min_size(
        rect.left_top() + Vec2::new(12.0, 12.0),
        Vec2::new(78.0, 54.0),
    );
    let report = composite_layers_report(ui, sample_blend_layers(sample, mode));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(16),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 42)),
        egui::StrokeKind::Outside,
    );
    ui.painter().text(
        rect.left_bottom() + Vec2::new(12.0, -26.0),
        egui::Align2::LEFT_BOTTOM,
        label,
        egui::FontId::proportional(12.0),
        Color32::from_rgb(238, 242, 255),
    );
    ui.painter().text(
        rect.left_bottom() + Vec2::new(12.0, -9.0),
        egui::Align2::LEFT_BOTTOM,
        format!("issues={}", report.issues.len()),
        egui::FontId::proportional(10.0),
        Color32::from_rgb(170, 182, 204),
    );
}

fn muted_text_color() -> Color32 {
    Color32::from_rgb(170, 182, 204)
}

fn primitive_coverage_rows() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("widgets/app_shell", "Breadcrumbs, SidebarNav, StatusBar", "click/selection state"),
        ("widgets/channel_strip", "ChannelStrip", "gain/pan/mute/solo/record/sends"),
        ("widgets/controls", "CollapsePanel, ControlGroup, ColorSwatch, SearchField, ToggleDot, ToolButton", "click/text/color state"),
        ("widgets/data", "DataTable, PropertyGrid, TreeTable", "selection/sort/model render"),
        ("widgets/daw_editors", "ColorWheel, PianoRoll, PluginManager, GeneratorOverlay, SystemMonitor, ControllerLinkOverlay, MixerStripDesigner", "live value/edit toggles"),
        ("widgets/designer", "DesignerCanvas, RoutingCable", "drag/hit-area render probe"),
        ("widgets/displays", "Waveform, WaveformDisplay, SpectrumDisplay, SpectrogramDisplay, MiniBarGraph", "sample data render"),
        ("widgets/dock", "DockOverlay, DockPanel model/recovery, FloatingPanel, ResizableSplit", "drop/drag/split hit areas"),
        ("widgets/drag/grid", "DragNumber, VerticalDrag, DragReorder, StepCellGrid, StepGrid", "drag/click grid toggles"),
        ("widgets/faders/knobs/meters", "Fader, Slider, RangeSlider, XYPad, Knob, Meter", "drag/wheel/display"),
        ("widgets/menus/overlays", "TopMenuBar, ContextMenuBuilder, CommandPalette, ModalOverlay, ToastLayer, ProgressOverlay", "menu/cmd/modal/progress"),
        ("widgets/tabs/toolbar/transport/tree", "TabBar, ToolbarStrip, TransportButton, TreeView", "click/drag/selection"),
        ("widgets/timeline", "TimelineClip, Ruler, GridCanvas, NoteRect, LoopRegion, FadeHandle, AutomationCurve", "clip/ruler widgets + paint hit area"),
        ("forms", "TextField, TextAreaField, CheckboxField, SwitchField, SelectField", "text/bool/select live state"),
        ("Material 3", "M3Button/Card/Chip/Switch/Checkbox/Radio/Slider/TextField/List/Nav/AppBar/Dialog/Snackbar/Fab/Progress/Badge/Divider/Tooltip/Dropdown", "click/toggle/select/modal/snackbar"),
        ("draw/render", "BlendLayer, RenderReport, CPU/GPU/fallback, compound ClipMask", "all BlendMode variants + report issues"),
        ("icons", "Icon and IconButton", "glyph render + hover/click response"),
        ("non-widget helpers", "model/helper APIs are exercised by effects_evaluator/tests", "not inherently interactive"),
    ]
}

fn report_ui(ui: &mut egui::Ui, report: &RenderReport) {
    ui.monospace(format!(
        "backend={:?} quality={:?}->{:?} issues={}",
        report.backend,
        report.requested_quality,
        report.actual_quality,
        report.issues.len()
    ));
    for issue in &report.issues {
        ui.monospace(format!(
            "{:?}/{:?}: {}",
            issue.feature, issue.kind, issue.message
        ));
    }
}

fn sample_blend_layers(rect: Rect, mode: BlendMode) -> Vec<BlendLayer> {
    vec![
        BlendLayer::new(vec![egui::Shape::rect_filled(
            rect.translate(Vec2::new(0.0, 0.0)),
            12.0,
            Color32::from_rgb(90, 170, 255),
        )]),
        BlendLayer::new(vec![egui::Shape::circle_filled(
            rect.center() + Vec2::new(rect.width() * 0.16, 0.0),
            rect.height().min(rect.width()) * 0.38,
            Color32::from_rgb(255, 120, 96),
        )])
        .blend_mode(mode)
        .opacity(0.82),
    ]
}

fn all_blend_modes() -> &'static [BlendMode] {
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
    MODES
}

fn blend_mode_combo(ui: &mut egui::Ui, mode: &mut BlendMode) {
    egui::ComboBox::from_label("blend mode")
        .selected_text(format!("{:?}", mode))
        .show_ui(ui, |ui| {
            for candidate in all_blend_modes() {
                ui.selectable_value(mode, candidate.clone(), format!("{:?}", candidate));
            }
        });
}

fn configure_gallery_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(proportional) = fonts.families.get(&egui::FontFamily::Proportional).cloned() {
        fonts
            .families
            .insert(egui::FontFamily::Name("icons".into()), proportional.clone());
        fonts
            .families
            .insert(egui::FontFamily::Name("phosphor".into()), proportional);
    }

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Primitive Gallery",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1420.0, 920.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            configure_gallery_fonts(&cc.egui_ctx);
            Ok(Box::new(PrimitiveGalleryApp::new(&cc.egui_ctx)))
        }),
    )
}
