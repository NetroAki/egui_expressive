use super::*;

pub(crate) fn artboard_re() -> &'static Regex {
    ARTBOARD_RE.get_or_init(|| {
        Regex::new(r"%AI9_Artboard\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)\s+(-?\d+\.?\d*)")
            .expect("valid artboard regex")
    })
}

pub(crate) fn artboard_name_re() -> &'static Regex {
    ARTBOARD_NAME_RE.get_or_init(|| {
        Regex::new(r"%AI9_ArtboardName\s+([^\n]+)").expect("valid artboard name regex")
    })
}

/// RGBA Color representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub opacity: Option<f64>,
    #[serde(default)]
    pub blend_mode: String,
}

/// Stroke representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stroke {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub opacity: Option<f64>,
    #[serde(default)]
    pub blend_mode: String,
    #[serde(default)]
    pub cap: Option<String>,
    #[serde(default)]
    pub join: Option<String>,
    #[serde(default)]
    pub dash: Option<Vec<f32>>,
    #[serde(default)]
    pub miter_limit: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Value>,
}

/// Live Effect parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEffectParams {
    #[serde(flatten)]
    pub params: HashMap<String, Value>,
}

/// Live Effect representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEffect {
    pub name: String,
    pub params: LiveEffectParams,
}

/// Mesh patch corner and color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshPatch {
    pub corners: Vec<Vec<f64>>,
    pub colors: Vec<Vec<u8>>,
}

/// Envelope mesh representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeMesh {
    pub rows: usize,
    pub cols: usize,
    pub points: Vec<Vec<f64>>,
}

/// 3D effect representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeD {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub depth: f64,
    #[serde(rename = "rotation_x")]
    pub rotation_x: f64,
    #[serde(rename = "rotation_y")]
    pub rotation_y: f64,
    #[serde(rename = "rotation_z")]
    pub rotation_z: f64,
}

/// Page tile representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTile {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Artboard representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artboard {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A Bezier path point with anchor and control handles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathPoint {
    pub anchor: [f64; 2],
    #[serde(alias = "leftDir")]
    pub left_ctrl: [f64; 2],
    #[serde(alias = "rightDir")]
    pub right_ctrl: [f64; 2],
}

/// Element representation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Element {
    pub id: String,
    #[serde(default)]
    pub live_effects: Vec<LiveEffect>,
    #[serde(default)]
    pub appearance_fills: Vec<Color>,
    #[serde(default)]
    pub appearance_strokes: Vec<Stroke>,
    #[serde(default)]
    pub mesh_patches: Vec<MeshPatch>,
    #[serde(default)]
    pub envelope_mesh: Option<EnvelopeMesh>,
    #[serde(default)]
    pub three_d: Option<ThreeD>,
    #[serde(default)]
    pub rotation_deg: f64,
    #[serde(default)]
    pub scale_x: f64,
    #[serde(default)]
    pub scale_y: f64,
    #[serde(default)]
    pub translate_x: f64,
    #[serde(default)]
    pub translate_y: f64,
    #[serde(default)]
    pub transform_candidates: Vec<[f64; 5]>,
    #[serde(default)]
    pub corner_radius: f64,
    #[serde(default)]
    pub path_points: Vec<PathPoint>,
    #[serde(default)]
    pub path_closed: bool,
    #[serde(default)]
    pub artboard_name: Option<String>,
    #[serde(default)]
    pub is_pseudo_element: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub element_type: Option<String>,
    #[serde(default)]
    pub bounds: Option<[f64; 4]>,
}

/// Parsed AI file result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiParseResult {
    pub version: String,
    #[serde(rename = "source_file")]
    pub source_file: String,
    #[serde(rename = "ai_version")]
    pub ai_version: String,
    #[serde(default)]
    pub artboards: Vec<Artboard>,
    #[serde(default)]
    pub page_tiles: Vec<PageTile>,
    #[serde(default)]
    pub elements: Vec<Element>,
    #[serde(default)]
    pub transform_candidates: Vec<Element>,
    #[serde(default)]
    pub errors: Vec<String>,
}
