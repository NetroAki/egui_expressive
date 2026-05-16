use super::*;

/// A full Illustrator artboard scene in logical points.
#[derive(Clone, Debug, Default)]
pub struct ArtboardScene {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<SceneNode>,
}

/// Convert compact `(x, y)` tuples into egui path points.
///
/// This keeps generated Illustrator paths readable while still producing normal
/// `egui::Pos2` values for code-first callers and the scene renderer.
pub fn path_points(points: &[(f32, f32)]) -> Vec<egui::Pos2> {
    points.iter().map(|&(x, y)| egui::pos2(x, y)).collect()
}

/// Convert artboard-relative `(x, y)` tuples into absolute egui path points.
pub fn offset_path_points(origin: egui::Pos2, points: &[(f32, f32)]) -> Vec<egui::Pos2> {
    points
        .iter()
        .map(|&(x, y)| origin + egui::vec2(x, y))
        .collect()
}

/// A closed or open contour used by path-like scene and typography data.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathContour {
    pub points: Vec<egui::Pos2>,
    pub closed: bool,
}

/// A retained scene node. Bounds are artboard-relative.
#[derive(Clone, Debug)]
pub struct SceneNode {
    pub id: String,
    pub geometry: Geometry,
    pub appearance: AppearanceStack,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub rotation_deg: f32,
    pub clip_children: bool,
    pub children: Vec<SceneNode>,
}

impl SceneNode {
    pub fn rect(id: impl Into<String>, rect: egui::Rect, corner_radius: f32) -> Self {
        Self {
            id: id.into(),
            geometry: Geometry::Rect {
                rect,
                corner_radius,
            },
            appearance: AppearanceStack::default(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            rotation_deg: 0.0,
            clip_children: false,
            children: Vec::new(),
        }
    }

    pub fn group(id: impl Into<String>, bounds: egui::Rect) -> Self {
        Self {
            id: id.into(),
            geometry: Geometry::Group { bounds },
            appearance: AppearanceStack::default(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            rotation_deg: 0.0,
            clip_children: false,
            children: Vec::new(),
        }
    }

    pub fn clip_group(id: impl Into<String>, bounds: egui::Rect) -> Self {
        Self::group(id, bounds).with_clip_children(true)
    }

    pub fn ellipse(id: impl Into<String>, rect: egui::Rect) -> Self {
        Self {
            id: id.into(),
            geometry: Geometry::Ellipse { rect },
            appearance: AppearanceStack::default(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            rotation_deg: 0.0,
            clip_children: false,
            children: Vec::new(),
        }
    }

    pub fn path(id: impl Into<String>, points: Vec<egui::Pos2>, closed: bool) -> Self {
        Self {
            id: id.into(),
            geometry: Geometry::Path { points, closed },
            appearance: AppearanceStack::default(),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            rotation_deg: 0.0,
            clip_children: false,
            children: Vec::new(),
        }
    }

    pub fn with_fill(mut self, paint: PaintSource) -> Self {
        self.appearance.entries.push(FillLayer::paint(paint).into());
        self
    }

    pub fn with_fill_layer(mut self, fill: FillLayer) -> Self {
        self.appearance.entries.push(fill.into());
        self
    }

    pub fn with_stroke(mut self, paint: PaintSource, width: f32) -> Self {
        self.appearance
            .entries
            .push(StrokeLayer::new(width, paint).into());
        self
    }

    pub fn with_stroke_layer(mut self, stroke: StrokeLayer) -> Self {
        self.appearance.entries.push(stroke.into());
        self
    }

    pub fn with_effect(mut self, effect: EffectDef) -> Self {
        self.appearance
            .entries
            .push(EffectLayer::new(effect).into());
        self
    }

    pub fn with_effect_layer(mut self, effect: EffectLayer) -> Self {
        self.appearance.entries.push(effect.into());
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub fn with_rotation(mut self, rotation_deg: f32) -> Self {
        self.rotation_deg = rotation_deg;
        self
    }

    pub fn with_clip_children(mut self, clip_children: bool) -> Self {
        self.clip_children = clip_children;
        self
    }

    pub fn with_child(mut self, child: SceneNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn push_child(&mut self, child: SceneNode) {
        self.children.push(child);
    }

    /// Convert a LayoutElement into a SceneNode, preserving rich fidelity.
    pub fn from_layout_element(elem: &crate::codegen::LayoutElement) -> Self {
        let path_backed_geometry = !elem.path_points.is_empty();
        let geometry = if path_backed_geometry {
            Geometry::Path {
                points: sample_layout_path(&elem.path_points, elem.path_closed),
                closed: elem.path_closed,
            }
        } else if elem.el_type == crate::codegen::ElementType::Circle
            || elem.el_type == crate::codegen::ElementType::Ellipse
        {
            Geometry::Ellipse {
                rect: egui::Rect::from_min_size(
                    egui::pos2(elem.x, elem.y),
                    egui::vec2(elem.w, elem.h),
                ),
            }
        } else if elem.el_type == crate::codegen::ElementType::Group {
            Geometry::Group {
                bounds: egui::Rect::from_min_size(
                    egui::pos2(elem.x, elem.y),
                    egui::vec2(elem.w, elem.h),
                ),
            }
        } else {
            Geometry::Rect {
                rect: egui::Rect::from_min_size(
                    egui::pos2(elem.x, elem.y),
                    egui::vec2(elem.w, elem.h),
                ),
                corner_radius: elem.corner_radius,
            }
        };

        let mut appearance = elem.appearance_stack.clone();

        // Fallback to legacy properties if appearance stack is empty
        if appearance.is_empty() {
            if !elem.appearance_fills.is_empty() || !elem.appearance_strokes.is_empty() {
                for fill in &elem.appearance_fills {
                    appearance.entries.push(AppearanceEntry::Fill(FillLayer {
                        paint: if let Some(grad) = &fill.gradient {
                            if grad.gradient_type == crate::codegen::GradientType::Radial {
                                PaintSource::RadialGradient(grad.clone())
                            } else {
                                PaintSource::LinearGradient(grad.clone())
                            }
                        } else {
                            PaintSource::Solid(fill.color)
                        },
                        opacity: fill.opacity,
                        blend_mode: fill.blend_mode.clone(),
                    }));
                }

                for stroke in &elem.appearance_strokes {
                    let paint = if let Some(pattern) = &stroke.pattern {
                        PaintSource::Pattern(pattern.clone())
                    } else if let Some(grad) = &stroke.gradient {
                        if grad.gradient_type == crate::codegen::GradientType::Radial {
                            PaintSource::RadialGradient(grad.clone())
                        } else {
                            PaintSource::LinearGradient(grad.clone())
                        }
                    } else {
                        PaintSource::Solid(stroke.color)
                    };
                    appearance
                        .entries
                        .push(AppearanceEntry::Stroke(StrokeLayer {
                            paint,
                            width: stroke.width,
                            opacity: stroke.opacity,
                            blend_mode: stroke.blend_mode.clone(),
                            cap: stroke.cap.clone(),
                            join: stroke.join.clone(),
                            dash: stroke.dash.clone(),
                            miter_limit: stroke.miter_limit,
                        }));
                }
            } else {
                if let Some(fill_color) = elem.fill {
                    appearance.entries.push(AppearanceEntry::Fill(FillLayer {
                        paint: if let Some(grad) = &elem.gradient {
                            if grad.gradient_type == crate::codegen::GradientType::Radial {
                                PaintSource::RadialGradient(grad.clone())
                            } else {
                                PaintSource::LinearGradient(grad.clone())
                            }
                        } else {
                            PaintSource::Solid(fill_color)
                        },
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                    }));
                }

                if let Some((width, color)) = elem.stroke {
                    appearance
                        .entries
                        .push(AppearanceEntry::Stroke(StrokeLayer {
                            paint: PaintSource::Solid(color),
                            width,
                            opacity: 1.0,
                            blend_mode: BlendMode::Normal,
                            cap: elem.stroke_cap.clone(),
                            join: elem.stroke_join.clone(),
                            dash: elem.stroke_dash.clone(),
                            miter_limit: elem.stroke_miter_limit,
                        }));
                }
            }

            for effect in &elem.effects {
                appearance
                    .entries
                    .push(AppearanceEntry::Effect(EffectLayer {
                        effect_type: effect.effect_type.clone(),
                        params: effect.clone(),
                        opacity: 1.0,
                        blend_mode: effect.blend_mode.clone(),
                    }));
            }
        }

        Self {
            id: elem.id.clone(),
            geometry,
            appearance,
            opacity: elem.opacity,
            blend_mode: elem.blend_mode.clone(),
            rotation_deg: if path_backed_geometry {
                0.0
            } else {
                elem.rotation_deg
            },
            clip_children: elem.clip_children,
            children: elem
                .children
                .iter()
                .map(Self::from_layout_element)
                .collect(),
        }
    }
}

/// Geometry supported by the parity renderer.
#[derive(Clone, Debug)]
pub enum Geometry {
    Group {
        bounds: egui::Rect,
    },
    Rect {
        rect: egui::Rect,
        corner_radius: f32,
    },
    Ellipse {
        rect: egui::Rect,
    },
    Path {
        points: Vec<egui::Pos2>,
        closed: bool,
    },
    MeshPatch {
        corners: [egui::Pos2; 4],
        colors: [egui::Color32; 4],
        subdivisions: usize,
    },
}

impl Geometry {
    pub fn bounds(&self) -> egui::Rect {
        match self {
            Self::Group { bounds }
            | Self::Rect { rect: bounds, .. }
            | Self::Ellipse { rect: bounds } => *bounds,
            Self::MeshPatch { corners, .. } => bounds_for_points(corners),
            Self::Path { points, .. } => bounds_for_slice(points),
        }
    }
}

/// Ordered Illustrator appearance stack.
#[derive(Clone, Debug, Default)]
pub struct AppearanceStack {
    pub entries: Vec<AppearanceEntry>,
}

impl AppearanceStack {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns true when exact rendering requires an offscreen render target / shader pass.
    pub fn requires_offscreen(&self) -> bool {
        self.entries.iter().any(|entry| match entry {
            AppearanceEntry::Fill(fill) => fill.blend_mode != BlendMode::Normal,
            AppearanceEntry::Stroke(stroke) => stroke.blend_mode != BlendMode::Normal,
            AppearanceEntry::Effect(effect) => {
                matches!(
                    effect.effect_type,
                    EffectType::GaussianBlur | EffectType::LiveEffect | EffectType::Unknown(_)
                ) || effect.blend_mode != BlendMode::Normal
            }
        })
    }
}

#[derive(Clone, Debug)]
pub enum AppearanceEntry {
    Fill(FillLayer),
    Stroke(StrokeLayer),
    Effect(EffectLayer),
}

#[derive(Clone, Debug)]
pub struct FillLayer {
    pub paint: PaintSource,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

impl FillLayer {
    pub fn solid(color: egui::Color32) -> Self {
        Self::paint(PaintSource::Solid(color))
    }

    pub fn paint(paint: PaintSource) -> Self {
        Self {
            paint,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        }
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

impl From<FillLayer> for AppearanceEntry {
    fn from(layer: FillLayer) -> Self {
        Self::Fill(layer)
    }
}

#[derive(Clone, Debug)]
pub struct StrokeLayer {
    pub paint: PaintSource,
    pub width: f32,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub cap: Option<StrokeCap>,
    pub join: Option<StrokeJoin>,
    pub dash: Option<Vec<f32>>,
    pub miter_limit: Option<f32>,
}

impl StrokeLayer {
    pub fn new(width: f32, paint: PaintSource) -> Self {
        Self {
            paint,
            width,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            cap: None,
            join: None,
            dash: None,
            miter_limit: None,
        }
    }

    pub fn solid(width: f32, color: egui::Color32) -> Self {
        Self::new(width, PaintSource::Solid(color))
    }

    pub fn dash(mut self, dash: Vec<f32>) -> Self {
        self.dash = Some(dash);
        self
    }

    pub fn cap(mut self, cap: StrokeCap) -> Self {
        self.cap = Some(cap);
        self
    }

    pub fn join(mut self, join: StrokeJoin) -> Self {
        self.join = Some(join);
        self
    }

    pub fn miter_limit(mut self, miter_limit: f32) -> Self {
        self.miter_limit = Some(miter_limit);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

impl From<StrokeLayer> for AppearanceEntry {
    fn from(layer: StrokeLayer) -> Self {
        Self::Stroke(layer)
    }
}

#[derive(Clone, Debug)]
pub struct EffectLayer {
    pub effect_type: EffectType,
    pub params: EffectDef,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

impl EffectLayer {
    pub fn new(effect: EffectDef) -> Self {
        Self {
            effect_type: effect.effect_type.clone(),
            params: effect,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        }
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

impl From<EffectLayer> for AppearanceEntry {
    fn from(layer: EffectLayer) -> Self {
        Self::Effect(layer)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaintSource {
    Solid(egui::Color32),
    LinearGradient(GradientDef),
    RadialGradient(GradientDef),
    Pattern(PatternDef),
    MeshGradient {
        corners: [egui::Pos2; 4],
        colors: [egui::Color32; 4],
        subdivisions: usize,
    },
    ProceduralNoise(NoiseDef),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PatternDef {
    pub name: String,
    pub seed: u32,
    pub foreground: egui::Color32,
    pub background: egui::Color32,
    pub cell_size: f32,
    pub mark_size: f32,
}

impl Default for PatternDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            seed: 0,
            foreground: egui::Color32::from_gray(80),
            background: egui::Color32::TRANSPARENT,
            cell_size: 8.0,
            mark_size: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoiseDef {
    pub seed: u32,
    pub cell_size: f32,
    pub opacity: f32,
}

impl Default for NoiseDef {
    fn default() -> Self {
        Self {
            seed: 0,
            cell_size: 2.0,
            opacity: 0.15,
        }
    }
}
