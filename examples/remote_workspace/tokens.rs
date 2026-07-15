use eframe::egui;

#[derive(Clone, Copy)]
pub struct WorkspaceTokens {
    pub bg_deep: egui::Color32,
    pub bg: egui::Color32,
    pub sidebar: egui::Color32,
    pub panel: egui::Color32,
    pub panel_raised: egui::Color32,
    pub field: egui::Color32,
    pub border: egui::Color32,
    pub border_strong: egui::Color32,
    pub text: egui::Color32,
    pub muted: egui::Color32,
    pub quiet: egui::Color32,
    pub mint: egui::Color32,
    pub blue: egui::Color32,
    pub blue_fill: egui::Color32,
    pub lavender: egui::Color32,
    pub sand: egui::Color32,
    pub rose: egui::Color32,
    pub info: egui::Color32,
}

impl Default for WorkspaceTokens {
    fn default() -> Self {
        Self {
            bg_deep: hex(0x04060a),
            bg: hex(0x080b12),
            sidebar: hex(0x0b0e16),
            panel: hex(0x0f131d),
            panel_raised: hex(0x151a26),
            field: hex(0x080b12),
            border: hex(0x242b39),
            border_strong: hex(0x384255),
            text: hex(0xeef1f7),
            muted: hex(0x8d96aa),
            quiet: hex(0x626c80),
            mint: hex(0x86d9ae),
            blue: hex(0x8eb9e8),
            blue_fill: hex(0x286596),
            lavender: hex(0xb7a7d9),
            sand: hex(0xd8bd88),
            rose: hex(0xdc8d99),
            info: hex(0x8eb9cf),
        }
    }
}

impl WorkspaceTokens {
    pub const RADIUS_SMALL: u8 = 3;
    pub const RADIUS_MEDIUM: u8 = 5;
    pub const RADIUS_LARGE: u8 = 7;

    pub fn mix(base: egui::Color32, accent: egui::Color32, amount: f32) -> egui::Color32 {
        let amount = amount.clamp(0.0, 1.0);
        let channel =
            |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * amount).round() as u8;
        egui::Color32::from_rgb(
            channel(base.r(), accent.r()),
            channel(base.g(), accent.g()),
            channel(base.b(), accent.b()),
        )
    }

    pub fn apply(self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = self.bg;
        visuals.window_fill = self.panel;
        visuals.extreme_bg_color = self.field;
        visuals.faint_bg_color = self.panel;
        visuals.code_bg_color = self.field;
        visuals.override_text_color = Some(self.text);
        visuals.selection.bg_fill = self.blue_fill;
        visuals.selection.stroke = egui::Stroke::new(1.0, self.blue);
        visuals.window_stroke = egui::Stroke::new(1.0, self.border);
        visuals.widgets.noninteractive.bg_fill = self.panel;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, self.border);
        visuals.widgets.inactive.bg_fill = self.panel_raised;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, self.border);
        visuals.widgets.hovered.bg_fill = self.border;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, self.border_strong);
        visuals.widgets.active.bg_fill = self.blue_fill;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, self.blue);
        visuals.widgets.open.bg_fill = self.panel_raised;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, self.blue);
        visuals.window_corner_radius = egui::CornerRadius::same(Self::RADIUS_LARGE);
        visuals.menu_corner_radius = egui::CornerRadius::same(Self::RADIUS_MEDIUM);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.global_style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.spacing.interact_size = egui::vec2(40.0, 40.0);
        style.spacing.indent = 16.0;
        style.visuals = ctx.global_style().visuals.clone();
        ctx.set_global_style(style);
    }
}

fn hex(value: u32) -> egui::Color32 {
    egui::Color32::from_rgb(
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
    )
}
