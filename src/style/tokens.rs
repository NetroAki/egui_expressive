use super::*;

/// Complete theming data for a widget: background, foreground, border,
/// corner rounding, and expansion.
pub struct WidgetTheme {
    pub bg: VisualState<Color32>,
    pub fg: VisualState<Color32>,
    pub border: VisualState<Stroke>,
    pub rounding: CornerRadius,
    pub expansion: VisualState<f32>,
}

impl WidgetTheme {
    /// Derive a [`WidgetTheme`] from egui's [`Visuals`].
    ///
    /// Note: egui 0.31's `Widgets` struct has `noninteractive`, `inactive`,
    /// `hovered`, `active`, and `open` fields. We map them to our six-variant
    /// system by treating `active` as pressed/focused and `open` as selected.
    #[inline]
    pub fn from_egui_visuals(visuals: &Visuals) -> Self {
        Self {
            bg: VisualState {
                inactive: visuals.widgets.inactive.bg_fill,
                hovered: visuals.widgets.hovered.bg_fill,
                pressed: visuals.widgets.active.bg_fill,
                selected: visuals.widgets.open.bg_fill,
                focused: visuals.widgets.active.bg_fill,
                disabled: visuals.widgets.noninteractive.bg_fill,
            },
            fg: VisualState {
                inactive: visuals.widgets.inactive.text_color(),
                hovered: visuals.widgets.hovered.text_color(),
                pressed: visuals.widgets.active.text_color(),
                selected: visuals.widgets.open.text_color(),
                focused: visuals.widgets.active.text_color(),
                disabled: visuals.widgets.noninteractive.text_color(),
            },
            border: VisualState {
                inactive: visuals.widgets.inactive.bg_stroke,
                hovered: visuals.widgets.hovered.bg_stroke,
                pressed: visuals.widgets.active.bg_stroke,
                selected: visuals.widgets.open.bg_stroke,
                focused: visuals.widgets.active.bg_stroke,
                disabled: visuals.widgets.noninteractive.bg_stroke,
            },
            rounding: CornerRadius::ZERO,
            expansion: VisualState::uniform(0.0),
        }
    }

    /// Resolve all theme fields for the given interaction state.
    #[inline]
    pub fn resolve(&self, r: &Response, selected: bool) -> ResolvedTheme {
        let bg = *self.bg.resolve(r, selected, false);
        let fg = *self.fg.resolve(r, selected, false);
        let border = *self.border.resolve(r, selected, false);
        let expansion = *self.expansion.resolve(r, selected, false);

        let rect = r.rect.expand(expansion);

        ResolvedTheme {
            bg,
            fg,
            border,
            rounding: self.rounding,
            rect,
        }
    }
}

impl Default for WidgetTheme {
    fn default() -> Self {
        Self {
            bg: VisualState::uniform(Color32::TRANSPARENT),
            fg: VisualState::uniform(Color32::WHITE),
            border: VisualState::uniform(Stroke::NONE),
            rounding: CornerRadius::ZERO,
            expansion: VisualState::uniform(0.0),
        }
    }
}

// ---------------------------------------------------------------------------
// ResolvedTheme
// ---------------------------------------------------------------------------

/// Fully resolved (non-variant) theme ready for painting.
#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub bg: Color32,
    pub fg: Color32,
    pub border: Stroke,
    pub rounding: CornerRadius,
    pub rect: egui::Rect,
}

// ─── Design Tokens ────────────────────────────────────────────────────────────

/// Surface scale matching the mockup's surface-50 through surface-950 palette.
#[derive(Clone, Debug)]
pub struct SurfacePalette {
    pub s50: egui::Color32,
    pub s100: egui::Color32,
    pub s150: egui::Color32,
    pub s200: egui::Color32,
    pub s250: egui::Color32,
    pub s300: egui::Color32,
    pub s400: egui::Color32,
    pub s500: egui::Color32,
    pub s600: egui::Color32,
    pub s700: egui::Color32,
    pub s800: egui::Color32,
    pub s900: egui::Color32,
    pub s950: egui::Color32,
}

impl SurfacePalette {
    /// Tailwind-style shade lookup.
    /// n must be one of: 50, 100, 150, 200, 250, 300, 400, 500, 600, 700, 800, 900, 950.
    pub fn shade(&self, n: u16) -> egui::Color32 {
        match n {
            50 => self.s50,
            100 => self.s100,
            150 => self.s150,
            200 => self.s200,
            250 => self.s250,
            300 => self.s300,
            400 => self.s400,
            500 => self.s500,
            600 => self.s600,
            700 => self.s700,
            800 => self.s800,
            900 => self.s900,
            950 => self.s950,
            _ => self.s500,
        }
    }

    /// Dark theme surface palette (near-black to near-white).
    pub fn dark() -> Self {
        Self {
            s50: egui::Color32::from_rgb(245, 245, 247),
            s100: egui::Color32::from_rgb(220, 220, 225),
            s150: egui::Color32::from_rgb(190, 190, 198),
            s200: egui::Color32::from_rgb(160, 160, 170),
            s250: egui::Color32::from_rgb(130, 130, 142),
            s300: egui::Color32::from_rgb(100, 100, 112),
            s400: egui::Color32::from_rgb(75, 75, 85),
            s500: egui::Color32::from_rgb(55, 55, 63),
            s600: egui::Color32::from_rgb(40, 40, 47),
            s700: egui::Color32::from_rgb(30, 30, 36),
            s800: egui::Color32::from_rgb(22, 22, 27),
            s900: egui::Color32::from_rgb(15, 15, 19),
            s950: egui::Color32::from_rgb(10, 10, 13),
        }
    }

    /// Light theme surface palette.
    pub fn light() -> Self {
        Self {
            s50: egui::Color32::from_rgb(10, 10, 13),
            s100: egui::Color32::from_rgb(22, 22, 27),
            s150: egui::Color32::from_rgb(40, 40, 47),
            s200: egui::Color32::from_rgb(55, 55, 63),
            s250: egui::Color32::from_rgb(75, 75, 85),
            s300: egui::Color32::from_rgb(100, 100, 112),
            s400: egui::Color32::from_rgb(130, 130, 142),
            s500: egui::Color32::from_rgb(160, 160, 170),
            s600: egui::Color32::from_rgb(190, 190, 198),
            s700: egui::Color32::from_rgb(220, 220, 225),
            s800: egui::Color32::from_rgb(235, 235, 240),
            s900: egui::Color32::from_rgb(245, 245, 247),
            s950: egui::Color32::from_rgb(255, 255, 255),
        }
    }
}

/// Named accent colors for semantic use (glow, active, midi, audio, warn, danger).
#[derive(Clone, Debug)]
pub struct AccentColors {
    pub glow: egui::Color32,
    pub active: egui::Color32,
    pub midi: egui::Color32,
    pub audio: egui::Color32,
    pub warn: egui::Color32,
    pub danger: egui::Color32,
}

impl AccentColors {
    pub fn default_dark() -> Self {
        Self {
            glow: egui::Color32::from_rgb(120, 200, 255),
            active: egui::Color32::from_rgb(80, 180, 120),
            midi: egui::Color32::from_rgb(180, 100, 220),
            audio: egui::Color32::from_rgb(80, 160, 220),
            warn: egui::Color32::from_rgb(220, 180, 60),
            danger: egui::Color32::from_rgb(220, 70, 70),
        }
    }
}

/// Spacing scale (4-point grid).
#[derive(Clone, Debug)]
pub struct SpacingScale {
    pub xs: f32,  // 2
    pub sm: f32,  // 4
    pub md: f32,  // 8
    pub lg: f32,  // 12
    pub xl: f32,  // 16
    pub xxl: f32, // 24
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 24.0,
        }
    }
}

/// Complete design token set for egui_expressive.
#[derive(Clone, Debug)]
pub struct DesignTokens {
    pub surface: SurfacePalette,
    pub accent: AccentColors,
    pub spacing: SpacingScale,
    /// Base rounding radius for controls.
    pub rounding: f32,
    /// Base rounding radius for panels/containers.
    pub panel_rounding: f32,
}

impl DesignTokens {
    pub fn dark() -> Self {
        Self {
            surface: SurfacePalette::dark(),
            accent: AccentColors::default_dark(),
            spacing: SpacingScale::default(),
            rounding: 3.0,
            panel_rounding: 6.0,
        }
    }

    pub fn light() -> Self {
        Self {
            surface: SurfacePalette::light(),
            accent: AccentColors::default_dark(),
            spacing: SpacingScale::default(),
            rounding: 3.0,
            panel_rounding: 6.0,
        }
    }

    /// Store tokens in egui context for global access.
    pub fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("__expressive_tokens"), self.clone()));
    }

    /// Retrieve tokens from egui context. Returns dark defaults if not set.
    pub fn load(ctx: &egui::Context) -> Self {
        ctx.data(|d| {
            d.get_temp(egui::Id::new("__expressive_tokens"))
                .unwrap_or_else(Self::dark)
        })
    }
}

// ─── Opacity Helpers ──────────────────────────────────────────────────────────

/// Multiply the alpha channel of a color by `alpha` (0.0–1.0).
pub fn with_alpha(color: egui::Color32, alpha: f32) -> egui::Color32 {
    let a = (color.a() as f32 * alpha.clamp(0.0, 1.0)) as u8;
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

/// Apply alpha to all `Color32` values inside a slice of shapes.
pub fn fade_shapes(shapes: &mut [egui::Shape], alpha: f32) {
    for shape in shapes.iter_mut() {
        fade_shape(shape, alpha);
    }
}

pub(crate) fn fade_shape(shape: &mut egui::Shape, alpha: f32) {
    match shape {
        egui::Shape::Rect(r) => {
            r.fill = with_alpha(r.fill, alpha);
            r.stroke.color = with_alpha(r.stroke.color, alpha);
        }
        egui::Shape::Circle(c) => {
            c.fill = with_alpha(c.fill, alpha);
            c.stroke.color = with_alpha(c.stroke.color, alpha);
        }
        egui::Shape::Path(p) => {
            p.fill = with_alpha(p.fill, alpha);
            if let egui::epaint::ColorMode::Solid(color) = p.stroke.color {
                p.stroke.color = egui::epaint::ColorMode::Solid(with_alpha(color, alpha));
            }
        }
        egui::Shape::LineSegment { stroke, .. } => {
            stroke.color = with_alpha(stroke.color, alpha);
        }
        egui::Shape::Vec(shapes) => {
            for s in shapes.iter_mut() {
                fade_shape(s, alpha);
            }
        }
        _ => {}
    }
}

// ─── Typography ───────────────────────────────────────────────────────────────
