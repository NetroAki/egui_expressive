use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct GradientStop {
    pub position: f32, // 0.0–1.0
    pub color: Color32,
}

/// Type of gradient.
#[derive(Clone, Debug, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
}

/// Gradient definition with angle and stops.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientDef {
    pub gradient_type: GradientType,
    pub angle_deg: f32,
    pub center: Option<[f32; 2]>,
    pub focal_point: Option<[f32; 2]>,
    pub radius: Option<f32>,
    pub transform: Option<[f32; 6]>,
    pub stops: Vec<GradientStop>,
}

pub(crate) fn stable_pattern_seed(name: &str) -> u32 {
    name.bytes().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

pub(crate) fn seeded_pattern_colors(seed: u32) -> (Color32, Color32) {
    let r = 64 + (seed & 0x7f) as u8;
    let g = 64 + ((seed >> 8) & 0x7f) as u8;
    let b = 64 + ((seed >> 16) & 0x7f) as u8;
    (
        Color32::from_rgba_unmultiplied(r, g, b, 220),
        Color32::from_rgba_unmultiplied(255 - r / 2, 255 - g / 2, 255 - b / 2, 48),
    )
}

/// Blend mode for compositing.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl std::str::FromStr for BlendMode {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "multiply" => Self::Multiply,
            "screen" => Self::Screen,
            "overlay" => Self::Overlay,
            "darken" => Self::Darken,
            "lighten" => Self::Lighten,
            "color_dodge" => Self::ColorDodge,
            "color_burn" => Self::ColorBurn,
            "hard_light" => Self::HardLight,
            "soft_light" => Self::SoftLight,
            "difference" => Self::Difference,
            "exclusion" => Self::Exclusion,
            "hue" => Self::Hue,
            "saturation" => Self::Saturation,
            "color" => Self::Color,
            "luminosity" => Self::Luminosity,
            _ => Self::Normal,
        })
    }
}

/// Effect type for shadows/glow.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectType {
    DropShadow,
    InnerShadow,
    OuterGlow,
    InnerGlow,
    GaussianBlur,
    Bevel,
    Feather,
    Noise,
    LiveEffect,
    Unknown(String),
}

/// Effect definition.
#[derive(Clone, Debug)]
pub struct EffectDef {
    pub effect_type: EffectType,
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color32,
    pub blend_mode: BlendMode,
    // For bevel
    pub depth: f32,
    pub angle: f32,
    pub highlight: Option<Color32>,
    pub shadow_color: Option<Color32>,
    // For blur/feather
    pub radius: f32,
    // For noise/grain
    pub amount: f32,
    pub scale: f32,
    pub seed: u32,
}

impl Default for EffectDef {
    fn default() -> Self {
        Self {
            effect_type: EffectType::DropShadow,
            x: 0.0,
            y: 0.0,
            blur: 0.0,
            spread: 0.0,
            color: Color32::BLACK,
            blend_mode: BlendMode::Normal,
            depth: 0.0,
            angle: 0.0,
            highlight: None,
            shadow_color: None,
            radius: 0.0,
            amount: 0.0,
            scale: 1.0,
            seed: 0,
        }
    }
}

/// Text alignment options.
#[derive(Clone, Debug, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justified,
}

/// Stroke cap style.
#[derive(Clone, Debug, PartialEq)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

impl std::str::FromStr for StrokeCap {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "round" => Self::Round,
            "square" => Self::Square,
            _ => Self::Butt,
        })
    }
}

/// Stroke join style.
#[derive(Clone, Debug, PartialEq)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

impl std::str::FromStr for StrokeJoin {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "round" => Self::Round,
            "bevel" => Self::Bevel,
            _ => Self::Miter,
        })
    }
}

/// Text decoration.
#[derive(Clone, Debug, PartialEq)]
pub enum TextDecoration {
    Underline,
    Strikethrough,
    Both,
}

impl std::str::FromStr for TextDecoration {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "strikethrough" => Self::Strikethrough,
            "underline_strikethrough" | "both" => Self::Both,
            _ => Self::Underline,
        })
    }
}

/// Text transform.
#[derive(Clone, Debug, PartialEq)]
pub enum TextTransform {
    AllCaps,
    SmallCaps,
}

impl std::str::FromStr for TextTransform {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "small_caps" => Self::SmallCaps,
            _ => Self::AllCaps,
        })
    }
}

/// A single text run with its own style (for mixed-style text).
#[derive(Clone, Debug)]
pub struct TextRun {
    pub text: String,
    pub size: f32,
    pub weight: u16,
    pub color: Option<Color32>,
}

/// A third-party effect detected on an element.
#[derive(Clone, Debug)]
pub struct ThirdPartyEffect {
    pub effect_type: String,
    pub opaque: bool,
    pub note: String,
}

/// A fill from the appearance stack (multiple fills per element).
#[derive(Clone, Debug)]
pub struct AppearanceFill {
    pub color: Color32,
    pub gradient: Option<GradientDef>,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

/// A stroke from the appearance stack (multiple strokes per element).
#[derive(Clone, Debug)]
pub struct AppearanceStroke {
    pub color: Color32,
    pub gradient: Option<GradientDef>,
    pub pattern: Option<crate::scene::PatternDef>,
    pub width: f32,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub cap: Option<StrokeCap>,
    pub join: Option<StrokeJoin>,
    pub dash: Option<Vec<f32>>,
    pub miter_limit: Option<f32>,
}

/// Illustrator path anchor and Bezier handles preserved for code generation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathPoint {
    pub anchor: [f32; 2],
    pub left_ctrl: [f32; 2],
    pub right_ctrl: [f32; 2],
}

/// Visual styling properties carried through the layout node tree.
#[derive(Clone, Debug, Default)]
pub struct VisualStyle {
    pub opacity: f32, // 1.0 = fully opaque
    pub rotation_deg: f32,
    pub corner_radius: f32,
    pub gradient: Option<GradientDef>,
    pub blend_mode: BlendMode,
    pub effects: Vec<EffectDef>,
    pub stroke_dash: Option<Vec<f32>>,
    pub stroke_cap: Option<StrokeCap>,
    pub stroke_join: Option<StrokeJoin>,
    pub stroke_miter_limit: Option<f32>,
    pub stroke: Option<(f32, Color32)>,
    pub image_path: Option<String>, // for Image nodes
}

impl VisualStyle {
    pub fn from_element(elem: &LayoutElement) -> Self {
        Self {
            opacity: elem.opacity,
            rotation_deg: elem.rotation_deg,
            corner_radius: elem.corner_radius,
            gradient: elem.gradient.clone(),
            blend_mode: elem.blend_mode.clone(),
            effects: elem.effects.clone(),
            stroke_dash: elem.stroke_dash.clone(),
            stroke_cap: elem.stroke_cap.clone(),
            stroke_join: elem.stroke_join.clone(),
            stroke_miter_limit: elem.stroke_miter_limit,
            stroke: elem.stroke,
            image_path: elem.image_path.clone(),
        }
    }

    pub fn is_default(&self) -> bool {
        self.opacity >= 0.999
            && self.rotation_deg.abs() < 0.001
            && self.corner_radius.abs() < 0.001
            && self.gradient.is_none()
            && self.blend_mode == BlendMode::Normal
            && self.effects.is_empty()
            && self.stroke_dash.is_none()
            && self.stroke_miter_limit.is_none()
            && self.stroke.is_none()
    }
}

/// A parsed element from SVG/Illustrator export.
#[derive(Clone, Debug)]
pub struct LayoutElement {
    pub id: String,
    pub el_type: ElementType,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub fill: Option<Color32>,
    pub stroke: Option<(f32, Color32)>, // (width, color)
    pub text: Option<String>,
    pub text_size: Option<f32>,
    pub children: Vec<LayoutElement>,
    // Extended fields
    pub opacity: f32,
    pub rotation_deg: f32,
    pub corner_radius: f32,
    pub gradient: Option<GradientDef>,
    pub blend_mode: BlendMode,
    pub effects: Vec<EffectDef>,
    pub stroke_dash: Option<Vec<f32>>,
    pub clip_children: bool,
    pub text_align: Option<TextAlign>,
    pub letter_spacing: Option<f32>,
    pub line_height: Option<f32>,
    // Stroke details (from Illustrator)
    pub stroke_cap: Option<StrokeCap>,
    pub stroke_join: Option<StrokeJoin>,
    pub stroke_miter_limit: Option<f32>,
    // Text details (from Illustrator)
    pub text_decoration: Option<TextDecoration>,
    pub text_transform: Option<TextTransform>,
    pub text_runs: Vec<TextRun>,
    // Element metadata
    pub symbol_name: Option<String>,
    pub is_compound_path: bool,
    pub is_gradient_mesh: bool,
    pub is_chart: bool,
    pub is_opaque: bool,
    pub third_party_effects: Vec<ThirdPartyEffect>,
    pub notes: Vec<String>,
    // Appearance stack (multiple fills/strokes from expand+analyze)
    pub appearance_fills: Vec<AppearanceFill>,
    pub appearance_strokes: Vec<AppearanceStroke>,
    pub appearance_stack: crate::scene::AppearanceStack,
    pub path_points: Vec<PathPoint>,
    pub path_closed: bool,
    /// Artboard this element belongs to (None = unassigned / appears in all artboards).
    pub artboard_name: Option<String>,
    pub image_path: Option<String>,
}

impl LayoutElement {
    pub fn new(id: String, el_type: ElementType, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            el_type,
            x,
            y,
            w,
            h,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            stroke_cap: None,
            stroke_join: None,
            stroke_miter_limit: None,
            text_decoration: None,
            text_transform: None,
            text_runs: vec![],
            symbol_name: None,
            is_compound_path: false,
            is_gradient_mesh: false,
            is_chart: false,
            is_opaque: false,
            third_party_effects: vec![],
            notes: vec![],
            appearance_fills: vec![],
            appearance_strokes: vec![],
            appearance_stack: crate::scene::AppearanceStack::default(),
            path_points: vec![],
            path_closed: false,
            artboard_name: None,
            image_path: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Group,
    Shape,
    Circle,
    Ellipse,
    Path,
    Text,
    Image,
    Unknown,
}

/// Inferred layout node — the intermediate representation between raw elements and code.
#[derive(Clone, Debug)]
pub enum LayoutNode {
    Row {
        gap: f32,
        children: Vec<LayoutNode>,
        bg: Option<Color32>,
        id: String,
    },
    Column {
        gap: f32,
        children: Vec<LayoutNode>,
        bg: Option<Color32>,
        id: String,
    },
    ScrollArea {
        vertical: bool,
        horizontal: bool,
        children: Vec<LayoutNode>,
        id: String,
    },
    Panel {
        side: PanelSide,
        children: Vec<LayoutNode>,
        id: String,
    },
    Card {
        children: Vec<LayoutNode>,
        bg: Color32,
        rounding: f32,
        id: String,
    },
    Button {
        label: String,
        id: String,
    },
    Label {
        text: String,
        size: f32,
        color: Option<Color32>,
        font_family: Option<String>,
        id: String,
    },
    TextEdit {
        placeholder: String,
        id: String,
    },
    Separator {
        id: String,
    },
    Spacer {
        size: f32,
        id: String,
    },
    Badge {
        text: String,
        id: String,
    },
    Icon {
        name: String,
        id: String,
    },
    Shape {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: Color32,
        id: String,
        style: VisualStyle,
    },
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        id: String,
        style: VisualStyle,
    },
    RichScene(crate::scene::SceneNode),
    Unknown {
        id: String,
        comment: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanelSide {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

// ============================================================================
// Naming Convention Parser
// ============================================================================
