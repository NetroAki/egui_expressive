use eframe::egui;
#[cfg(not(target_os = "android"))]
use egui_expressive::M3Theme;
use egui_expressive::{
    breakpoint_for_width, AccentKind, Axis, BreakpointName, Breakpoints, CanvasItem, CheckboxField,
    DisplayScale, EditorCanvas, Elevation, Fader, Knob, M3Button, M3Card, M3Chip, M3NavItem,
    M3NavigationBar, M3NavigationRail, M3Slider, M3Switch, M3TextField, M3TopAppBar, Meter,
    Responsive, SelectField, SelectOption, SnapGrid, StatusBar, StatusBarItem, SurfaceLevel,
    SwitchField, TextAreaField, TextField, Theme, Tw, TypeLabel, TypeSpec,
};

const BREAKPOINTS: Breakpoints = Breakpoints::tailwind();
const NAV_ITEMS: [(&str, char, u32); 4] = [
    ("Overview", '⌂', 0),
    ("Controls", '◫', 0),
    ("Scene", '▦', 3),
    ("Review", '✓', 0),
];

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Cross-Platform Showcase",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(CrossPlatformShowcase::new(&cc.egui_ctx)))),
    )
}

pub struct CrossPlatformShowcase {
    selected_nav: usize,
    selected_focus_chip: usize,
    selected_scene: u64,
    search: String,
    surface_name: String,
    notes: String,
    density: SurfaceDensity,
    render_profile: RenderProfile,
    offline_only: bool,
    keep_support_planned: bool,
    reduce_motion: bool,
    touch_safe: bool,
    ambient_mix: f64,
    focus_mix: f64,
    clarity_mix: f64,
    master_level: f64,
    motion_budget: f32,
    status_message: String,
    perf: PerfSnapshot,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SurfaceDensity {
    Compact,
    #[default]
    Comfortable,
    Touch,
}

impl SurfaceDensity {
    fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
            Self::Touch => "Touch-safe",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Compact => Self::Comfortable,
            Self::Comfortable => Self::Touch,
            Self::Touch => Self::Compact,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum RenderProfile {
    Battery,
    #[default]
    Balanced,
    Crisp,
}

impl RenderProfile {
    fn label(self) -> &'static str {
        match self {
            Self::Battery => "Battery bias",
            Self::Balanced => "Balanced",
            Self::Crisp => "Crisp",
        }
    }
}

#[derive(Clone, Copy, Default)]
struct PerfSnapshot {
    frame_ms: f32,
    cpu_ms: f32,
    fps: f32,
    viewport: egui::Vec2,
    pixels_per_point: f32,
    tap_target_px: f32,
    frame_nr: u64,
    cpu_known: bool,
}

#[derive(Clone, Copy)]
struct SceneBlock {
    id: u64,
    label: &'static str,
    subtitle: &'static str,
    rect: egui::Rect,
    accent: egui::Color32,
}

impl Default for CrossPlatformShowcase {
    fn default() -> Self {
        Self {
            selected_nav: 0,
            selected_focus_chip: 1,
            selected_scene: 2,
            search: String::new(),
            surface_name: "Shared smoke surface".to_owned(),
            notes: "Promote support labels only after host-specific smoke artifacts exist."
                .to_owned(),
            density: SurfaceDensity::Comfortable,
            render_profile: RenderProfile::Balanced,
            offline_only: true,
            keep_support_planned: true,
            reduce_motion: false,
            touch_safe: true,
            ambient_mix: 0.58,
            focus_mix: 0.74,
            clarity_mix: 0.41,
            master_level: -4.0,
            motion_budget: 0.56,
            status_message:
                "Ready: shared showcase stays app-owned, offline-safe, and artifact-aware."
                    .to_owned(),
            perf: PerfSnapshot::default(),
        }
    }
}

impl eframe::App for CrossPlatformShowcase {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        #[cfg(not(target_os = "android"))]
        ui.ctx().request_repaint();

        let shell_theme = Theme::dark();

        self.sync_perf(ui.ctx(), frame);

        let breakpoint = breakpoint_for_width(self.perf.viewport.x, BREAKPOINTS);

        Tw::new()
            .w_full()
            .min_h_screen()
            .bg_surface(SurfaceLevel::Base)
            .text_surface(SurfaceLevel::On)
            .p(12.0)
            .show(ui, |ui| {
                self.show_top_app_bar(ui, breakpoint);
                ui.add_space(10.0);

                if uses_navigation_rail(breakpoint) {
                    ui.horizontal(|ui| {
                        self.show_navigation_rail(ui, &shell_theme);
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width());
                            self.show_main_surface(ui, breakpoint, &shell_theme);
                        });
                    });
                } else {
                    self.show_main_surface(ui, breakpoint, &shell_theme);
                    ui.add_space(8.0);
                    self.show_navigation_bar(ui);
                }
            });
    }
}

impl CrossPlatformShowcase {
    pub fn new(ctx: &egui::Context) -> Self {
        #[cfg(not(target_os = "android"))]
        {
            let shell_theme = Theme::dark();
            shell_theme.store(ctx);

            let m3_theme = M3Theme::from_seed(shell_theme.colors.primary, shell_theme.is_dark);
            m3_theme.store(ctx);
            m3_theme.apply_to_egui(ctx);
        }

        #[cfg(target_os = "android")]
        let _ = ctx;

        Self::default()
    }

    fn sync_perf(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let frame_ms = ctx.input(|input| input.stable_dt.max(1.0 / 240.0) * 1000.0);
        let viewport = ctx.input(|input| input.content_rect().size());
        let scale = DisplayScale::new(ctx.pixels_per_point());

        self.perf.frame_ms = blend_metric(self.perf.frame_ms, frame_ms, 0.18);
        self.perf.fps = 1000.0 / self.perf.frame_ms.max(0.1);
        self.perf.viewport = viewport;
        self.perf.pixels_per_point = ctx.pixels_per_point();
        self.perf.tap_target_px = scale.logical_to_physical(44.0);
        self.perf.frame_nr = ctx.cumulative_frame_nr();

        if let Some(cpu_usage) = frame.info().cpu_usage {
            self.perf.cpu_ms = blend_metric(self.perf.cpu_ms, cpu_usage * 1000.0, 0.22);
            self.perf.cpu_known = true;
        }
    }

    fn show_top_app_bar(&mut self, ui: &mut egui::Ui, breakpoint: BreakpointName) {
        let mut bar = M3TopAppBar::new("Cross-Platform Showcase")
            .navigation_icon('◈')
            .action('⌕', "Search")
            .action('⋯', "More")
            .scrolled(true);

        if matches!(breakpoint, BreakpointName::Xs | BreakpointName::Sm) {
            bar = bar.center_aligned();
        } else {
            bar = bar.medium();
        }

        bar.show(ui);
    }

    fn show_navigation_rail(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let mut rail = M3NavigationRail::new(&mut self.selected_nav)
            .width(92.0)
            .header(|ui| {
                ui.add_space(4.0);
                ui.centered_and_justified(|ui| {
                    Tw::new()
                        .w(48.0)
                        .h(48.0)
                        .rounded_full()
                        .bg_accent(AccentKind::Secondary)
                        .text_accent(AccentKind::OnSecondary)
                        .show(ui, |ui| {
                            ui.strong("XP");
                        });
                });
            });

        for (label, icon, badge) in NAV_ITEMS {
            let item = if badge > 0 {
                M3NavItem::new(label, icon).badge(badge)
            } else {
                M3NavItem::new(label, icon)
            };
            rail = rail.item(item);
        }

        rail.show(ui);

        ui.add_space(8.0);
        ui.small(
            TypeSpec::new(12.0)
                .color(theme.colors.on_surface_variant)
                .to_rich_text("Shared host shell"),
        );
    }

    fn show_navigation_bar(&mut self, ui: &mut egui::Ui) {
        let mut bar = M3NavigationBar::new(&mut self.selected_nav).height(84.0);
        for (label, icon, badge) in NAV_ITEMS {
            let item = if badge > 0 {
                M3NavItem::new(label, icon).badge(badge)
            } else {
                M3NavItem::new(label, icon)
            };
            bar = bar.item(item);
        }
        bar.show(ui);
    }

    fn show_main_surface(&mut self, ui: &mut egui::Ui, breakpoint: BreakpointName, theme: &Theme) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_hero(ui, breakpoint, theme);
            ui.add_space(12.0);
            self.show_controls_and_form(ui, breakpoint, theme);
            ui.add_space(12.0);
            self.show_scene_canvas(ui, breakpoint, theme);
            ui.add_space(12.0);
            self.show_status(ui, breakpoint);
        });
    }

    fn show_hero(&mut self, ui: &mut egui::Ui, breakpoint: BreakpointName, theme: &Theme) {
        let headline_size = *Responsive::new(28.0).md(34.0).lg(42.0).resolve(breakpoint);
        let nav_label = nav_label(self.selected_nav);
        let focus_label = focus_chip_label(self.selected_focus_chip);
        let headline = format!("Responsive smoke surface for {nav_label}");
        let filter_label = if self.search.trim().is_empty() {
            "all modules".to_owned()
        } else {
            self.search.clone()
        };

        show_shell_card(ui, theme.colors.primary, |ui| {
            if matches!(
                breakpoint,
                BreakpointName::Lg | BreakpointName::Xl | BreakpointName::Xxl
            ) {
                ui.columns(2, |columns| {
                    self.show_hero_copy(
                        &mut columns[0],
                        theme,
                        headline_size,
                        &headline,
                        focus_label,
                        &filter_label,
                    );
                    self.show_hero_metrics(&mut columns[1], theme, breakpoint);
                });
            } else {
                self.show_hero_copy(
                    ui,
                    theme,
                    headline_size,
                    &headline,
                    focus_label,
                    &filter_label,
                );
                ui.add_space(12.0);
                self.show_hero_metrics(ui, theme, breakpoint);
            }
        });
    }

    fn show_hero_copy(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        headline_size: f32,
        headline: &str,
        focus_label: &str,
        filter_label: &str,
    ) {
        ui.add(TypeLabel::new(
            "SHARED SMOKE SURFACE",
            TypeSpec::micro_label().color(theme.colors.secondary),
        ));
        ui.add_space(4.0);
        ui.add(TypeLabel::new(
            headline,
            TypeSpec::new(headline_size)
                .weight(700)
                .line_height(1.0)
                .letter_spacing(-0.6)
                .color(theme.colors.on_surface),
        ));
        ui.add_space(8.0);
        ui.label(
            "One polished host-facing surface for desktop and mobile shells. Build and smoke this first, then promote platform labels only after target-specific artifacts exist.",
        );
        ui.add_space(10.0);
        ui.add(
            M3TextField::new("showcase.search", "Filter emphasis", &mut self.search)
                .outlined()
                .leading_icon('⌕')
                .trailing_icon('✦'),
        );
        ui.add_space(8.0);
        ui.small(format!(
            "Current emphasis: {focus_label}; query: {filter_label}"
        ));
        ui.add_space(10.0);

        ui.horizontal_wrapped(|ui| {
            for (index, label, icon) in [
                (0, "Touch-safe", '◎'),
                (1, "Balanced", '◌'),
                (2, "Diagnostics", '◈'),
            ] {
                if ui
                    .add(
                        M3Chip::new(label)
                            .filter()
                            .selected(self.selected_focus_chip == index)
                            .icon(icon),
                    )
                    .clicked()
                {
                    self.selected_focus_chip = index;
                    self.status_message = format!("Focus emphasis set to {label}");
                }
            }
        });

        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(M3Button::new("Preview host").icon('▶').width(152.0))
                .clicked()
            {
                self.status_message = format!(
                    "Previewed {headline} without filesystem, network, or permission side effects"
                );
            }

            if ui
                .add(
                    M3Button::new("Rotate density")
                        .tonal()
                        .icon('↔')
                        .width(152.0),
                )
                .clicked()
            {
                self.density = self.density.next();
                self.status_message = format!("Density profile set to {}", self.density.label());
            }

            if ui
                .add(
                    M3Button::new("Keep planned labels")
                        .outlined()
                        .icon('✓')
                        .width(176.0),
                )
                .clicked()
            {
                self.keep_support_planned = true;
                self.status_message =
                    "Support labels remain planned until per-platform smoke artifacts exist."
                        .to_owned();
            }
        });
    }

    fn show_hero_metrics(&mut self, ui: &mut egui::Ui, theme: &Theme, breakpoint: BreakpointName) {
        M3Card::new().outlined().padding(14.0).show(ui, |ui| {
            section_header(ui, theme, "LIVE HOST STATE", "Frame counters and neutral contract");
            ui.add_space(10.0);
            perf_readout(ui, theme, "Frame", &format!("{:.1} ms", self.perf.frame_ms));
            perf_readout(ui, theme, "CPU", &cpu_label(&self.perf));
            perf_readout(
                ui,
                theme,
                "Viewport",
                &format!("{:.0} × {:.0} pt", self.perf.viewport.x, self.perf.viewport.y),
            );
            perf_readout(
                ui,
                theme,
                "Density",
                &format!("{} · {:.2}×", breakpoint_label(breakpoint), self.perf.pixels_per_point),
            );
            perf_readout(
                ui,
                theme,
                "Tap target",
                &format!("44 pt → {:.0} px", self.perf.tap_target_px),
            );
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                accent_capsule(ui, "No filesystem", AccentKind::Primary);
                accent_capsule(ui, "No network", AccentKind::Secondary);
                accent_capsule(ui, "No native capture", AccentKind::Primary);
            });
            ui.add_space(8.0);
            ui.small(
                "This example only reads egui frame timing and viewport state from the current host. It does not imply support proof for any target by itself.",
            );
        });
    }

    fn show_controls_and_form(
        &mut self,
        ui: &mut egui::Ui,
        breakpoint: BreakpointName,
        theme: &Theme,
    ) {
        if matches!(breakpoint, BreakpointName::Xs | BreakpointName::Sm) {
            self.show_control_rack(ui, theme);
            ui.add_space(12.0);
            self.show_host_form(ui, theme);
        } else {
            ui.columns(2, |columns| {
                self.show_control_rack(&mut columns[0], theme);
                self.show_host_form(&mut columns[1], theme);
            });
        }
    }

    fn show_control_rack(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let time = ui.ctx().input(|input| input.time) as f32;
        let ambient_meter = animated_level(time, 1.1, 0.55);
        let focus_meter = animated_level(time, 1.6, 1.25);
        let clarity_meter = animated_level(time, 0.9, 2.05);
        let master_meter = ((ambient_meter + focus_meter + clarity_meter) / 3.0).clamp(0.0, 1.0);

        show_shell_card(ui, theme.colors.secondary, |ui| {
            section_header(
                ui,
                theme,
                "RESPONSIVE CONTROLS",
                "Widgets sized for touch and pointer",
            );
            ui.add_space(6.0);
            ui.small(
                "Reusable primitives stay local to the host app and scale cleanly from compact mobile shells to wider desktop workspaces.",
            );
            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                channel_strip(ui, theme, "Ambient", &mut self.ambient_mix, ambient_meter);
                channel_strip(ui, theme, "Focus", &mut self.focus_mix, focus_meter);
                channel_strip(ui, theme, "Clarity", &mut self.clarity_mix, clarity_meter);
            });

            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Motion budget").strong());
                    ui.small("Lower keeps motion calmer on constrained hosts.");
                });
                ui.add_space(8.0);
                ui.add(M3Slider::new(&mut self.motion_budget, 0.0..=1.0).steps(4));
            });

            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("Reduce motion");
                ui.add(M3Switch::new(
                    "showcase.reduce_motion",
                    &mut self.reduce_motion,
                ));
                ui.add_space(12.0);
                ui.label("Touch-safe spacing");
                ui.add(M3Switch::new("showcase.touch_safe", &mut self.touch_safe));
            });

            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Master trim").strong());
                    ui.small("One vertical fader and host-facing meter.");
                });
                ui.add_space(12.0);
                ui.add(
                    Fader::new(&mut self.master_level, -18.0..=6.0).size(egui::vec2(30.0, 92.0)),
                );
                ui.add(
                    Meter::new(master_meter)
                        .size(egui::vec2(12.0, 92.0))
                        .segments(8),
                );
                ui.label(format!("{:.1} dB", self.master_level));
            });
        });
    }

    fn show_host_form(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        show_shell_card(ui, theme.colors.primary, |ui| {
            section_header(
                ui,
                theme,
                "HOST CONTRACT",
                "Forms keep side effects app-owned",
            );
            ui.add_space(8.0);

            TextField::new("Surface label", &mut self.surface_name)
                .hint("Cross-platform showcase")
                .show(ui);

            ui.add_space(8.0);
            SelectField::new("Density", &mut self.density)
                .options([
                    SelectOption::new(SurfaceDensity::Compact, "Compact"),
                    SelectOption::new(SurfaceDensity::Comfortable, "Comfortable"),
                    SelectOption::new(SurfaceDensity::Touch, "Touch-safe"),
                ])
                .show(ui);

            ui.add_space(8.0);
            SelectField::new("Rendering", &mut self.render_profile)
                .options([
                    SelectOption::new(RenderProfile::Balanced, "Balanced"),
                    SelectOption::new(RenderProfile::Crisp, "Crisp"),
                    SelectOption::new(RenderProfile::Battery, "Battery bias"),
                ])
                .show(ui);

            ui.add_space(8.0);
            CheckboxField::new("Offline-only interactions", &mut self.offline_only).show(ui);
            SwitchField::new(
                "Keep support claims planned",
                &mut self.keep_support_planned,
            )
            .show(ui);

            ui.add_space(8.0);
            TextAreaField::new("Smoke notes", &mut self.notes)
                .rows(4)
                .show(ui);

            ui.add_space(8.0);
            ui.small(format!(
                "Current density: {}; rendering: {}; 44pt target = {:.0}px on this host.",
                self.density.label(),
                self.render_profile.label(),
                self.perf.tap_target_px,
            ));
        });
    }

    fn show_scene_canvas(&mut self, ui: &mut egui::Ui, breakpoint: BreakpointName, theme: &Theme) {
        let blocks = scene_blocks(theme);
        let canvas_height = *Responsive::new(220.0)
            .md(280.0)
            .lg(340.0)
            .resolve(breakpoint);

        show_shell_card(ui, theme.colors.secondary, |ui| {
            section_header(
                ui,
                theme,
                "EDITOR CANVAS",
                "Tap or click cards to inspect shared states",
            );
            ui.add_space(6.0);
            ui.small(
                "The canvas stays local to egui state: no files, no background services, and no platform capture. It is the same surface desktop and mobile hosts can smoke.",
            );
            ui.add_space(12.0);

            EditorCanvas::new(
                ui.id().with("cross_platform_showcase_canvas"),
                egui::vec2(1200.0, canvas_height),
            )
            .snap_grid(SnapGrid::uniform(0.5))
            .x_axis(Axis::time(0.0..=14.0, 4.0).unit("s").minor_step(1.0))
            .zoom_range(18.0, 140.0)
            .show(ui, |canvas| {
                let painter = canvas.ui.painter();
                for lane in 0..3 {
                    let lane_rect = egui::Rect::from_min_max(
                        egui::pos2(0.0, lane as f32 * 2.4),
                        egui::pos2(14.0, lane as f32 * 2.4 + 2.0),
                    );
                    painter.rect_filled(
                        canvas.rect_to_screen(lane_rect),
                        10.0,
                        theme.colors.surface_container.linear_multiply(0.9),
                    );
                }

                for block in blocks {
                    let canvas_item = CanvasItem::rect(block.id, block.rect)
                        .resizable_x(true)
                        .min_size(egui::vec2(1.4, 0.8));
                    let screen_rect = canvas.rect_to_screen(canvas_item.rect);
                    let response = canvas.ui.interact(
                        screen_rect.expand(4.0),
                        canvas.ui.id().with(("showcase_scene", block.id)),
                        egui::Sense::click(),
                    );

                    if response.clicked() {
                        self.selected_scene = block.id;
                        self.status_message = format!("Focused scene card: {}", block.label);
                    }

                    let selected = self.selected_scene == block.id;
                    let fill = if selected {
                        block.accent.linear_multiply(0.88)
                    } else {
                        block.accent.linear_multiply(0.58)
                    };
                    let stroke = if selected {
                        egui::Stroke::new(2.0, egui::Color32::WHITE)
                    } else {
                        egui::Stroke::new(1.0, block.accent)
                    };

                    painter.rect_filled(screen_rect, 12.0, fill);
                    painter.rect_stroke(screen_rect, 12.0, stroke, egui::StrokeKind::Outside);
                    painter.text(
                        screen_rect.left_top() + egui::vec2(12.0, 14.0),
                        egui::Align2::LEFT_TOP,
                        block.label,
                        egui::FontId::proportional(16.0),
                        egui::Color32::WHITE,
                    );
                    painter.text(
                        screen_rect.left_bottom() - egui::vec2(-12.0, 12.0),
                        egui::Align2::LEFT_BOTTOM,
                        block.subtitle,
                        egui::FontId::proportional(12.0),
                        egui::Color32::from_white_alpha(210),
                    );
                }
            });

            ui.add_space(8.0);
            if let Some(selected) = blocks.iter().find(|block| block.id == self.selected_scene) {
                ui.label(format!(
                    "Focused scene: {} — {}",
                    selected.label, selected.subtitle
                ));
            }
        });
    }

    fn show_status(&mut self, ui: &mut egui::Ui, breakpoint: BreakpointName) {
        let status = vec![
            StatusBarItem::new("Section").value(nav_label(self.selected_nav)),
            StatusBarItem::new("Breakpoint").value(breakpoint_label(breakpoint)),
            StatusBarItem::new("Frame").value(format!(
                "{:.1} ms / {:.0} fps",
                self.perf.frame_ms, self.perf.fps
            )),
            StatusBarItem::new("CPU").value(cpu_label(&self.perf)),
            StatusBarItem::new("Frame #").value(self.perf.frame_nr.to_string()),
        ];

        ui.label(egui::RichText::new(&self.status_message).strong());
        ui.add_space(6.0);
        ui.add(StatusBar::new(&status));
    }
}

fn show_shell_card(ui: &mut egui::Ui, accent: egui::Color32, contents: impl FnOnce(&mut egui::Ui)) {
    Tw::new()
        .w_full()
        .p(16.0)
        .rounded_xl()
        .shadow(Elevation::Level2)
        .bg_surface(SurfaceLevel::Container)
        .border(1.0)
        .border_color(accent.linear_multiply(0.28))
        .show(ui, contents);
}

fn section_header(ui: &mut egui::Ui, theme: &Theme, overline: &str, title: &str) {
    ui.add(TypeLabel::new(
        overline,
        TypeSpec::micro_label().color(theme.colors.secondary),
    ));
    ui.label(egui::RichText::new(title).strong().size(18.0));
}

fn accent_capsule(ui: &mut egui::Ui, label: &str, accent: AccentKind) {
    let on_accent = match accent {
        AccentKind::Primary => AccentKind::OnPrimary,
        AccentKind::Secondary => AccentKind::OnSecondary,
        AccentKind::Error => AccentKind::OnError,
        AccentKind::OnPrimary
        | AccentKind::OnSecondary
        | AccentKind::OnError
        | AccentKind::Scrim => AccentKind::OnPrimary,
    };

    Tw::new()
        .px(10.0)
        .py(4.0)
        .rounded_full()
        .bg_accent(accent)
        .text_accent(on_accent)
        .show(ui, |ui| {
            ui.label(label);
        });
}

fn perf_readout(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add(TypeLabel::new(
            label,
            TypeSpec::micro_label().color(theme.colors.on_surface_variant),
        ));
        ui.add_space(8.0);
        ui.add(TypeLabel::new(
            value,
            TypeSpec::mono_readout(13.0).color(theme.colors.on_surface),
        ));
    });
}

fn channel_strip(ui: &mut egui::Ui, theme: &Theme, label: &str, value: &mut f64, meter_level: f32) {
    Tw::new()
        .w(110.0)
        .p(10.0)
        .rounded_lg()
        .bg_surface(SurfaceLevel::Dim)
        .border_t(2.0)
        .border_t_color(theme.colors.secondary)
        .show(ui, |ui| {
            ui.add(TypeLabel::new(
                "SEND",
                TypeSpec::micro_label().color(theme.colors.on_surface_variant),
            ));
            ui.add_space(8.0);
            ui.add(Knob::new(value, 0.0..=1.0).size(44.0).label(label));
            ui.add_space(6.0);
            ui.add(
                Meter::new(meter_level)
                    .size(egui::vec2(12.0, 56.0))
                    .segments(8),
            );
            ui.small(format!("{:.0}%", *value * 100.0));
        });
}

fn cpu_label(perf: &PerfSnapshot) -> String {
    if perf.cpu_known {
        format!("{:.2} ms", perf.cpu_ms)
    } else {
        "n/a".to_owned()
    }
}

fn animated_level(time: f32, speed: f32, phase: f32) -> f32 {
    ((time * speed + phase).sin() * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn blend_metric(current: f32, next: f32, alpha: f32) -> f32 {
    if current <= f32::EPSILON {
        next
    } else {
        current + (next - current) * alpha
    }
}

fn breakpoint_label(breakpoint: BreakpointName) -> &'static str {
    match breakpoint {
        BreakpointName::Xs => "xs",
        BreakpointName::Sm => "sm",
        BreakpointName::Md => "md",
        BreakpointName::Lg => "lg",
        BreakpointName::Xl => "xl",
        BreakpointName::Xxl => "2xl",
    }
}

fn uses_navigation_rail(breakpoint: BreakpointName) -> bool {
    matches!(
        breakpoint,
        BreakpointName::Lg | BreakpointName::Xl | BreakpointName::Xxl
    )
}

fn nav_label(selected_nav: usize) -> &'static str {
    NAV_ITEMS
        .get(selected_nav)
        .map(|(label, _, _)| *label)
        .unwrap_or("Overview")
}

fn focus_chip_label(index: usize) -> &'static str {
    match index {
        0 => "touch-safe",
        2 => "diagnostics",
        _ => "balanced",
    }
}

fn scene_blocks(theme: &Theme) -> [SceneBlock; 4] {
    [
        SceneBlock {
            id: 1,
            label: "Host summary",
            subtitle: "Shared launch + health card",
            rect: egui::Rect::from_min_size(egui::pos2(0.8, 0.3), egui::vec2(3.0, 1.4)),
            accent: theme.colors.primary,
        },
        SceneBlock {
            id: 2,
            label: "Control rack",
            subtitle: "Touch-safe meters and knobs",
            rect: egui::Rect::from_min_size(egui::pos2(4.2, 2.7), egui::vec2(4.0, 1.4)),
            accent: theme.colors.secondary,
        },
        SceneBlock {
            id: 3,
            label: "Artifact note",
            subtitle: "Support remains planned",
            rect: egui::Rect::from_min_size(egui::pos2(8.7, 0.6), egui::vec2(3.6, 1.2)),
            accent: theme.colors.error,
        },
        SceneBlock {
            id: 4,
            label: "Form review",
            subtitle: "Host-owned state only",
            rect: egui::Rect::from_min_size(egui::pos2(8.5, 3.0), egui::vec2(4.2, 1.3)),
            accent: theme.colors.outline_variant,
        },
    ]
}
