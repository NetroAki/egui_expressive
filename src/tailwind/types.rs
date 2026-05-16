//! Small Tailwind/CSS value types used by the `Tw` builder.

/// CSS-like size token used by `Tw`.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Size {
    #[default]
    Auto,
    Px(f32),
    Full,
    Percent(f32),
    ViewportWidth(f32),
    ViewportHeight(f32),
}

/// Tailwind-like font-weight scale for utility-style typography.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl FontWeight {
    pub const fn css_value(self) -> u16 {
        match self {
            Self::Thin => 100,
            Self::ExtraLight => 200,
            Self::Light => 300,
            Self::Normal => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::Black => 900,
        }
    }

    pub const fn from_css(weight: u16) -> Self {
        let clamped = if weight < 100 {
            100
        } else if weight > 900 {
            900
        } else {
            weight
        };
        let rounded = ((clamped + 50) / 100) * 100;
        match rounded {
            100 => Self::Thin,
            200 => Self::ExtraLight,
            300 => Self::Light,
            400 => Self::Normal,
            500 => Self::Medium,
            600 => Self::SemiBold,
            700 => Self::Bold,
            800 => Self::ExtraBold,
            _ => Self::Black,
        }
    }
}

/// Tailwind-like display intent for `Tw` containers.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Display {
    #[default]
    Block,
    Flex,
    Grid,
    Hidden,
}

/// Flex main-axis direction for `Tw::flex()` containers.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

/// Tailwind-like `justify-*` intent.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    Between,
}

/// Tailwind-like `items-*` cross-axis alignment intent.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Items {
    Start,
    #[default]
    Center,
    End,
    Stretch,
}

/// Overflow policy for clipped/scrolling containers.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Clip,
    Auto,
    Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientDirection {
    ToRight,
    ToBottom,
    ToBottomRight,
    Angle(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TwGradient {
    pub direction: GradientDirection,
    pub stops: Vec<(f32, egui::Color32)>,
}

impl TwGradient {
    pub fn new(
        direction: GradientDirection,
        stops: impl IntoIterator<Item = (f32, egui::Color32)>,
    ) -> Self {
        let mut stops: Vec<_> = stops
            .into_iter()
            .map(|(position, color)| (position.clamp(0.0, 1.0), color))
            .collect();
        stops.sort_by(|a, b| a.0.total_cmp(&b.0));
        if stops.is_empty() {
            stops.extend([
                (0.0, egui::Color32::TRANSPARENT),
                (1.0, egui::Color32::TRANSPARENT),
            ]);
        }
        Self { direction, stops }
    }

    pub fn two_stop(direction: GradientDirection, from: egui::Color32, to: egui::Color32) -> Self {
        Self::new(direction, [(0.0, from), (1.0, to)])
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        for (_, color) in &mut self.stops {
            *color = apply_opacity(*color, opacity);
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwRing {
    pub width: f32,
    pub color: egui::Color32,
}

impl TwRing {
    pub fn with_opacity(self, opacity: f32) -> Self {
        Self {
            color: apply_opacity(self.color, opacity),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwDropShadow {
    pub offset: egui::Vec2,
    pub blur: u8,
    pub color: egui::Color32,
}

impl TwDropShadow {
    pub fn with_opacity(self, opacity: f32) -> Self {
        Self {
            color: apply_opacity(self.color, opacity),
            ..self
        }
    }
}

/// Source selection for Tailwind-style backdrop blur rendering.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum TwBackdropSource {
    /// Keep `Tw::backdrop_blur` on the bounded overlay fallback path.
    #[default]
    BoundedOverlay,
    /// Use the app-provided snapshot helper when it reports exact support.
    AppProvidedSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwTransition {
    pub duration_secs: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionStyle {
    pub bg: egui::Color32,
    pub fg: egui::Color32,
}

/// Per-corner radius values used by Tailwind corner shorthands.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct RadiusCorners {
    pub nw: f32,
    pub ne: f32,
    pub sw: f32,
    pub se: f32,
}

impl RadiusCorners {
    pub fn same(radius: f32) -> Self {
        Self {
            nw: radius,
            ne: radius,
            sw: radius,
            se: radius,
        }
    }

    pub fn to_corner_radius(self) -> egui::CornerRadius {
        egui::CornerRadius {
            nw: clamp_radius(self.nw),
            ne: clamp_radius(self.ne),
            sw: clamp_radius(self.sw),
            se: clamp_radius(self.se),
        }
    }
}

fn clamp_radius(radius: f32) -> u8 {
    radius.clamp(0.0, 255.0).round() as u8
}

fn apply_opacity(color: egui::Color32, opacity: f32) -> egui::Color32 {
    let alpha = (color.a() as f32 * opacity.clamp(0.0, 1.0)).round() as u8;
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}
