#![allow(dead_code)]

//! Visual state system: hover / press / select / focus / disabled variants.

use egui::{Color32, Context, CornerRadius, Id, Response, Stroke, Visuals};

// ---------------------------------------------------------------------------
// VisualVariant
// ---------------------------------------------------------------------------

/// The six interactive visual states a widget can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisualVariant {
    #[default]
    Inactive,
    Hovered,
    Pressed,
    Selected,
    Focused,
    Disabled,
}

impl VisualVariant {
    /// Determine the appropriate variant from an interaction [`Response`].
    #[inline]
    pub fn from_response(r: &Response, selected: bool, disabled: bool) -> Self {
        if disabled {
            Self::Disabled
        } else if r.is_pointer_button_down_on() {
            Self::Pressed
        } else if r.has_focus() {
            Self::Focused
        } else if selected {
            Self::Selected
        } else if r.hovered() {
            Self::Hovered
        } else {
            Self::Inactive
        }
    }
}

// ---------------------------------------------------------------------------
// VisualState
// ---------------------------------------------------------------------------

/// A value parameterized by six visual variants.
#[derive(Debug, Clone)]
pub struct VisualState<T> {
    pub inactive: T,
    pub hovered: T,
    pub pressed: T,
    pub selected: T,
    pub focused: T,
    pub disabled: T,
}

impl<T: Clone> VisualState<T> {
    /// Initialize all variants with the same value.
    #[inline]
    pub fn uniform(value: T) -> Self {
        Self {
            inactive: value.clone(),
            hovered: value.clone(),
            pressed: value.clone(),
            selected: value.clone(),
            focused: value.clone(),
            disabled: value,
        }
    }

    /// Get the value for the given variant.
    #[inline]
    pub fn get(&self, variant: VisualVariant) -> &T {
        match variant {
            VisualVariant::Inactive => &self.inactive,
            VisualVariant::Hovered => &self.hovered,
            VisualVariant::Pressed => &self.pressed,
            VisualVariant::Selected => &self.selected,
            VisualVariant::Focused => &self.focused,
            VisualVariant::Disabled => &self.disabled,
        }
    }

    /// Resolve the correct value for a [`Response`]'s current interaction state.
    #[inline]
    pub fn resolve(&self, r: &Response, selected: bool, disabled: bool) -> &T {
        self.get(VisualVariant::from_response(r, selected, disabled))
    }
}

impl<T: Default> Default for VisualState<T> {
    fn default() -> Self {
        Self {
            inactive: T::default(),
            hovered: T::default(),
            pressed: T::default(),
            selected: T::default(),
            focused: T::default(),
            disabled: T::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Lerp
// ---------------------------------------------------------------------------

/// Trait for linear interpolation between two values.
pub trait Lerp: Sized {
    /// Linearly interpolate between `a` and `b` with parameter `t` in [0, 1].
    fn lerp(a: &Self, b: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Color32 {
    #[inline]
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let a = a.to_tuple();
        let b = b.to_tuple();
        Color32::from_rgba_unmultiplied(
            ((a.0 as f32 + (b.0 as f32 - a.0 as f32) * t).round() as u8).min(255),
            ((a.1 as f32 + (b.1 as f32 - a.1 as f32) * t).round() as u8).min(255),
            ((a.2 as f32 + (b.2 as f32 - a.2 as f32) * t).round() as u8).min(255),
            ((a.3 as f32 + (b.3 as f32 - a.3 as f32) * t).round() as u8).min(255),
        )
    }
}

impl Lerp for Stroke {
    #[inline]
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Stroke {
            width: Lerp::lerp(&a.width, &b.width, t),
            color: Lerp::lerp(&a.color, &b.color, t),
        }
    }
}

impl Lerp for egui::CornerRadius {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        egui::CornerRadius {
            nw: lerp_u8(a.nw, b.nw, t),
            ne: lerp_u8(a.ne, b.ne, t),
            sw: lerp_u8(a.sw, b.sw, t),
            se: lerp_u8(a.se, b.se, t),
        }
    }
}

impl Lerp for egui::Vec2 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        egui::Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }
}

impl Lerp for egui::Pos2 {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        egui::Pos2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

// ---------------------------------------------------------------------------
// Animated resolution extension
// ---------------------------------------------------------------------------

/// Extension trait providing animated resolution for [`VisualState`].
pub trait VisualStateExt<T: Clone + 'static> {
    /// Resolve the value for the current interaction state, animating smoothly
    /// between the previous and current variant over `duration` seconds.
    fn resolve_animated(
        &self,
        ctx: &Context,
        id: Id,
        r: &Response,
        selected: bool,
        disabled: bool,
        duration: f32,
    ) -> T;
}

impl<T: Lerp + Clone + 'static> VisualStateExt<T> for VisualState<T> {
    fn resolve_animated(
        &self,
        ctx: &Context,
        id: Id,
        r: &Response,
        selected: bool,
        disabled: bool,
        duration: f32,
    ) -> T {
        let current = VisualVariant::from_response(r, selected, disabled);

        // Retrieve or initialize the last variant from egui memory
        let last = ctx.memory(|m| m.data.get_temp::<VisualVariant>(id.with("__exp_vis")));

        // Update stored last variant
        ctx.memory_mut(|m| m.data.insert_temp(id.with("__exp_vis"), current));

        // Get the current target value
        let target = self.get(current).clone();

        // If nothing changed, return the target directly
        if Some(current) == last {
            return target;
        }

        // Compute animation t (0 → 1)
        let animating = last.is_some() && last != Some(current);
        let t = if animating {
            ctx.animate_bool_with_time(id.with("__anim"), true, duration)
        } else {
            1.0
        };

        // Lerp from previous value to current target
        if let Some(last_variant) = last {
            let prev = self.get(last_variant).clone();
            Lerp::lerp(&prev, &target, t)
        } else {
            target
        }
    }
}

// ---------------------------------------------------------------------------
// WidgetTheme
// ---------------------------------------------------------------------------

/// Complete theming data for a widget: background, foreground, border,
/// corner rounding, and expansion.
#[derive(Debug, Clone)]
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
        let bg = self.bg.resolve(r, selected, false).clone();
        let fg = self.fg.resolve(r, selected, false).clone();
        let border = self.border.resolve(r, selected, false).clone();
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
        ctx.memory_mut(|mem| {
            mem.data
                .insert_temp(egui::Id::new("__expressive_tokens"), self.clone())
        });
    }

    /// Retrieve tokens from egui context. Returns dark defaults if not set.
    pub fn load(ctx: &egui::Context) -> Self {
        ctx.memory(|mem| {
            mem.data
                .get_temp(egui::Id::new("__expressive_tokens"))
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

fn fade_shape(shape: &mut egui::Shape, alpha: f32) {
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

/// A text style combining font, color, and optional letter spacing.
#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_id: egui::FontId,
    pub color: egui::Color32,
    /// Extra pixels between characters (0.0 = normal).
    pub letter_spacing: f32,
}

impl TextStyle {
    pub fn new(font_id: egui::FontId, color: egui::Color32) -> Self {
        Self {
            font_id,
            color,
            letter_spacing: 0.0,
        }
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }
}

/// Named text styles for a consistent type system.
#[derive(Clone, Debug)]
pub struct TextStyles {
    pub label: TextStyle,
    pub heading: TextStyle,
    pub mono: TextStyle,
    pub small: TextStyle,
    pub hint: TextStyle,
}

impl TextStyles {
    /// Default dark-theme text styles matching the mockup.
    pub fn dark() -> Self {
        let fg = egui::Color32::from_rgb(220, 220, 225);
        let muted = egui::Color32::from_rgb(130, 130, 142);
        Self {
            label: TextStyle::new(egui::FontId::proportional(12.0), fg),
            heading: TextStyle::new(egui::FontId::proportional(14.0), fg),
            mono: TextStyle::new(egui::FontId::monospace(11.0), fg),
            small: TextStyle::new(egui::FontId::proportional(10.0), muted),
            hint: TextStyle::new(egui::FontId::proportional(9.0), muted).with_spacing(0.5),
        }
    }
}

/// Paint text with optional letter spacing by advancing character by character.
/// Falls back to a single `painter.text()` call when `letter_spacing == 0.0`.
pub fn styled_text(
    painter: &egui::Painter,
    pos: egui::Pos2,
    text: &str,
    style: &TextStyle,
) -> egui::Rect {
    if style.letter_spacing == 0.0 || text.is_empty() {
        return painter.text(
            pos,
            egui::Align2::LEFT_TOP,
            text,
            style.font_id.clone(),
            style.color,
        );
    }

    // Character-by-character advance for letter spacing
    let mut cursor = pos;
    let mut total_rect = egui::Rect::from_min_size(pos, egui::Vec2::ZERO);
    for ch in text.chars() {
        let s = ch.to_string();
        let r = painter.text(
            cursor,
            egui::Align2::LEFT_TOP,
            &s,
            style.font_id.clone(),
            style.color,
        );
        total_rect = total_rect.union(r);
        cursor.x += r.width() + style.letter_spacing;
    }
    total_rect
}

// ─── Scrollbar Styling ────────────────────────────────────────────────────────

/// Apply custom scrollbar styling to egui's Visuals.
/// Call this before rendering to match the mockup's thin dark scrollbars.
///
/// Note: `width` must be set separately via `ctx.style_mut(|s| s.spacing.scroll.bar_width = width)`.
/// The `width` parameter is documented here but cannot be applied to `Visuals` directly.
pub fn apply_scrollbar_style(
    visuals: &mut egui::Visuals,
    track_color: egui::Color32,
    thumb_color: egui::Color32,
    width: f32,
) {
    visuals.extreme_bg_color = track_color;
    // thumb color is applied via the inactive widget bg
    visuals.widgets.inactive.bg_fill = thumb_color;
    visuals.widgets.hovered.bg_fill = thumb_color.linear_multiply(1.2);
    // Note: scroll bar width is in Style::spacing, not Visuals.
    // Document this limitation in a comment.
    let _ = width; // width must be set via ctx.style_mut(|s| s.spacing.scroll.bar_width = width)
}

/// Apply the mockup's default thin scrollbar style to the egui context.
/// Call once at app startup or per-frame before rendering.
pub fn apply_default_scrollbar_style(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.spacing.scroll.bar_width = 4.0;
        style.spacing.scroll.bar_inner_margin = 1.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(22, 22, 27);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(55, 55, 63);
    });
}
