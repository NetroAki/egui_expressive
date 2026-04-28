//! Code-output Illustrator fidelity scene graph.
//!
//! This module is the retained appearance/render-plan layer that sits beside the existing
//! layout-inference codegen path. It preserves Illustrator ordering semantics: multiple fills and
//! strokes, effect layers, procedural paints, blend modes, masks, and isolated groups can be
//! represented without falling back to screenshots.

use crate::codegen::{BlendMode, EffectDef, EffectType, GradientDef, StrokeCap, StrokeJoin};

/// A full Illustrator artboard scene in logical points.
#[derive(Clone, Debug, Default)]
pub struct ArtboardScene {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<SceneNode>,
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

/// Render a retained artboard scene at the current egui cursor.
pub fn render_scene(ui: &mut egui::Ui, scene: &ArtboardScene) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(scene.width, scene.height), egui::Sense::hover());
    let painter = ui.painter().clone();
    for node in &scene.nodes {
        render_node(ui, &painter, rect.min.to_vec2(), node, 1.0);
    }
}

pub fn render_node(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    origin: egui::Vec2,
    node: &SceneNode,
    parent_opacity: f32,
) {
    let effective_opacity = node.opacity * parent_opacity;
    if effective_opacity <= 0.0 {
        return;
    }

    let rotated_geometry;
    let geometry = if node.rotation_deg.abs() > 0.001 {
        rotated_geometry = rotate_geometry(&node.geometry, node.rotation_deg);
        &rotated_geometry
    } else {
        &node.geometry
    };

    match geometry {
        Geometry::Group { .. } => {}
        Geometry::MeshPatch {
            corners,
            colors,
            subdivisions,
        } => {
            let corners = corners.map(|p| p + origin);
            let mut blended_colors = *colors;
            for c in &mut blended_colors {
                *c = resolve_color(ui, *c, effective_opacity, &node.blend_mode);
            }
            painter.add(crate::draw::mesh_gradient_patch(
                corners,
                blended_colors,
                *subdivisions,
            ));
        }
        geometry => render_geometry_appearance(
            ui,
            painter,
            origin,
            geometry,
            &node.appearance,
            effective_opacity,
            &node.blend_mode,
        ),
    }

    if node.clip_children && !node.children.is_empty() {
        let polygon = geometry_to_polygon(geometry, origin);
        let mut layers = Vec::new();
        for child in &node.children {
            collect_node_layers(ui, origin, child, effective_opacity, &mut layers);
        }
        if !layers.is_empty() {
            crate::draw::clipped_layers_gpu(ui, &polygon, layers);
        }
    } else {
        for child in &node.children {
            render_node(ui, painter, origin, child, effective_opacity);
        }
    }
}

fn collect_node_layers(
    ui: &mut egui::Ui,
    origin: egui::Vec2,
    node: &SceneNode,
    parent_opacity: f32,
    layers: &mut Vec<crate::draw::BlendLayer>,
) {
    let effective_opacity = node.opacity * parent_opacity;
    if effective_opacity <= 0.0 {
        return;
    }

    let rotated_geometry;
    let geometry = if node.rotation_deg.abs() > 0.001 {
        rotated_geometry = rotate_geometry(&node.geometry, node.rotation_deg);
        &rotated_geometry
    } else {
        &node.geometry
    };

    match geometry {
        Geometry::Group { .. } => {}
        Geometry::MeshPatch {
            corners,
            colors,
            subdivisions,
        } => {
            let mut blended_colors = *colors;
            for color in &mut blended_colors {
                let [r, g, b, a] = color.to_srgba_unmultiplied();
                *color = egui::Color32::from_rgba_unmultiplied(
                    r,
                    g,
                    b,
                    (a as f32 * effective_opacity).round().clamp(0.0, 255.0) as u8,
                );
            }
            layers.push(crate::draw::BlendLayer {
                shapes: vec![crate::draw::mesh_gradient_patch(
                    corners.map(|p| p + origin),
                    blended_colors,
                    *subdivisions,
                )],
                clip_polygons: Vec::new(),
                blend_mode: node.blend_mode.clone(),
                opacity: 1.0,
            });
        }
        geometry => {
            for entry in &node.appearance.entries {
                let (shapes, blend_mode) = match entry {
                    AppearanceEntry::Fill(fill) => (
                        paint_fill(
                            ui,
                            origin,
                            geometry,
                            fill,
                            effective_opacity,
                            &node.blend_mode,
                            true,
                        ),
                        if fill.blend_mode != BlendMode::Normal {
                            fill.blend_mode.clone()
                        } else {
                            node.blend_mode.clone()
                        },
                    ),
                    AppearanceEntry::Stroke(stroke) => (
                        paint_stroke(
                            ui,
                            origin,
                            geometry,
                            stroke,
                            effective_opacity,
                            &node.blend_mode,
                            true,
                        ),
                        if stroke.blend_mode != BlendMode::Normal {
                            stroke.blend_mode.clone()
                        } else {
                            node.blend_mode.clone()
                        },
                    ),
                    AppearanceEntry::Effect(effect) => (
                        paint_effect(
                            ui,
                            origin,
                            geometry,
                            effect,
                            effective_opacity,
                            &node.blend_mode,
                            true,
                        ),
                        if effect.blend_mode != BlendMode::Normal {
                            effect.blend_mode.clone()
                        } else {
                            node.blend_mode.clone()
                        },
                    ),
                };
                if !shapes.is_empty() {
                    layers.push(crate::draw::BlendLayer {
                        shapes,
                        clip_polygons: Vec::new(),
                        blend_mode,
                        opacity: 1.0,
                    });
                }
            }
        }
    }

    if node.clip_children {
        let polygon = geometry_to_polygon(geometry, origin);
        let mut child_layers = Vec::new();
        for child in &node.children {
            collect_node_layers(ui, origin, child, effective_opacity, &mut child_layers);
        }
        for mut layer in child_layers {
            if polygon.len() >= 3 {
                layer.clip_polygons.push(polygon.clone());
            }
            layers.push(layer);
        }
    } else {
        for child in &node.children {
            collect_node_layers(ui, origin, child, effective_opacity, layers);
        }
    }
}

pub fn render_geometry_appearance(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    origin: egui::Vec2,
    geometry: &Geometry,
    appearance: &AppearanceStack,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
) {
    let requires_offscreen =
        appearance.requires_offscreen() || *node_blend_mode != BlendMode::Normal;

    if requires_offscreen {
        let mut layers = Vec::new();
        for entry in &appearance.entries {
            let (shapes, blend_mode, _opacity) = match entry {
                AppearanceEntry::Fill(fill) => (
                    paint_fill(
                        ui,
                        origin,
                        geometry,
                        fill,
                        node_opacity,
                        node_blend_mode,
                        true,
                    ),
                    if fill.blend_mode != BlendMode::Normal {
                        fill.blend_mode.clone()
                    } else {
                        node_blend_mode.clone()
                    },
                    fill.opacity * node_opacity,
                ),
                AppearanceEntry::Stroke(stroke) => (
                    paint_stroke(
                        ui,
                        origin,
                        geometry,
                        stroke,
                        node_opacity,
                        node_blend_mode,
                        true,
                    ),
                    if stroke.blend_mode != BlendMode::Normal {
                        stroke.blend_mode.clone()
                    } else {
                        node_blend_mode.clone()
                    },
                    stroke.opacity * node_opacity,
                ),
                AppearanceEntry::Effect(effect) => (
                    paint_effect(
                        ui,
                        origin,
                        geometry,
                        effect,
                        node_opacity,
                        node_blend_mode,
                        true,
                    ),
                    if effect.blend_mode != BlendMode::Normal {
                        effect.blend_mode.clone()
                    } else {
                        node_blend_mode.clone()
                    },
                    effect.opacity * node_opacity,
                ),
            };
            if !shapes.is_empty() {
                layers.push(crate::draw::BlendLayer {
                    shapes,
                    clip_polygons: Vec::new(),
                    blend_mode,
                    opacity: 1.0, // Opacity is already baked into the shapes by paint_* functions
                });
            }
        }
        if !layers.is_empty() {
            crate::draw::composite_layers_gpu(ui, layers);
        }
    } else {
        for entry in &appearance.entries {
            let shapes = match entry {
                AppearanceEntry::Fill(fill) => paint_fill(
                    ui,
                    origin,
                    geometry,
                    fill,
                    node_opacity,
                    node_blend_mode,
                    false,
                ),
                AppearanceEntry::Stroke(stroke) => paint_stroke(
                    ui,
                    origin,
                    geometry,
                    stroke,
                    node_opacity,
                    node_blend_mode,
                    false,
                ),
                AppearanceEntry::Effect(effect) => paint_effect(
                    ui,
                    origin,
                    geometry,
                    effect,
                    node_opacity,
                    node_blend_mode,
                    false,
                ),
            };
            for shape in shapes {
                painter.add(shape);
            }
        }
    }
}

fn paint_fill(
    ui: &mut egui::Ui,
    origin: egui::Vec2,
    geometry: &Geometry,
    fill: &FillLayer,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
    force_normal_blend: bool,
) -> Vec<egui::Shape> {
    let opacity = fill.opacity * node_opacity;
    let blend_mode = if force_normal_blend {
        &BlendMode::Normal
    } else if fill.blend_mode != BlendMode::Normal {
        &fill.blend_mode
    } else {
        node_blend_mode
    };

    let mut shapes = Vec::new();
    let offset_gradient_point =
        |point: [f32; 2]| egui::pos2(point[0] + origin.x, point[1] + origin.y);

    match (&fill.paint, geometry) {
        (
            PaintSource::Solid(color),
            Geometry::Rect {
                rect,
                corner_radius,
            },
        ) => {
            shapes.push(egui::Shape::rect_filled(
                offset_rect(*rect, origin),
                *corner_radius,
                resolve_color(ui, *color, opacity, blend_mode),
            ));
        }
        (PaintSource::Solid(color), Geometry::Ellipse { rect }) => {
            let rect = offset_rect(*rect, origin);
            shapes.push(egui::Shape::ellipse_filled(
                rect.center(),
                egui::vec2(rect.width() * 0.5, rect.height() * 0.5),
                resolve_color(ui, *color, opacity, blend_mode),
            ));
        }
        (PaintSource::Solid(color), Geometry::Path { points, closed }) => {
            let points = offset_points(points, origin);
            if *closed {
                shapes.push(egui::Shape::Path(egui::epaint::PathShape {
                    points,
                    closed: true,
                    fill: resolve_color(ui, *color, opacity, blend_mode),
                    stroke: egui::Stroke::NONE.into(),
                }));
            }
        }
        (
            PaintSource::LinearGradient(gradient),
            Geometry::Rect {
                rect,
                corner_radius,
            },
        ) => {
            let rect = offset_rect(*rect, origin);
            let stops = gradient_stops(gradient, opacity, ui, blend_mode);
            if *corner_radius > 0.001 || gradient.transform.is_some() {
                let points = if *corner_radius > 0.001 {
                    crate::draw::rounded_rect_path(rect, *corner_radius)
                } else {
                    vec![
                        rect.left_top(),
                        rect.right_top(),
                        rect.right_bottom(),
                        rect.left_bottom(),
                    ]
                };
                if let Some(shape) = crate::draw::gradient_path_mesh_with_transform(
                    &points,
                    &stops,
                    gradient.angle_deg,
                    false,
                    crate::draw::GradientPathGeometry {
                        transform: gradient
                            .transform
                            .map(|matrix| offset_transform(matrix, origin)),
                        ..Default::default()
                    },
                ) {
                    shapes.push(shape);
                }
            } else {
                shapes.push(crate::draw::linear_gradient_rect(
                    rect,
                    &stops,
                    crate::draw::GradientDir::Angle(gradient.angle_deg),
                ));
            }
        }
        (PaintSource::LinearGradient(gradient), Geometry::Path { points, closed }) => {
            if *closed {
                let points = offset_points(points, origin);
                if let Some(shape) = crate::draw::gradient_path_mesh_with_transform(
                    &points,
                    &gradient_stops(gradient, opacity, ui, blend_mode),
                    gradient.angle_deg,
                    false,
                    crate::draw::GradientPathGeometry {
                        transform: gradient
                            .transform
                            .map(|matrix| offset_transform(matrix, origin)),
                        ..Default::default()
                    },
                ) {
                    shapes.push(shape);
                }
            }
        }
        (PaintSource::LinearGradient(gradient), Geometry::Ellipse { rect }) => {
            let points = ellipse_points(offset_rect(*rect, origin), 48);
            if let Some(shape) = crate::draw::gradient_path_mesh_with_transform(
                &points,
                &gradient_stops(gradient, opacity, ui, blend_mode),
                gradient.angle_deg,
                false,
                crate::draw::GradientPathGeometry {
                    transform: gradient
                        .transform
                        .map(|matrix| offset_transform(matrix, origin)),
                    ..Default::default()
                },
            ) {
                shapes.push(shape);
            }
        }
        (
            PaintSource::RadialGradient(gradient),
            Geometry::Rect {
                rect,
                corner_radius,
            },
        ) => {
            let rect = offset_rect(*rect, origin);
            let stops = gradient_stops(gradient, opacity, ui, blend_mode);
            if gradient.center.is_some()
                || gradient.focal_point.is_some()
                || gradient.radius.is_some()
                || gradient.transform.is_some()
                || *corner_radius > 0.001
            {
                let points = if *corner_radius > 0.001 {
                    crate::draw::rounded_rect_path(rect, *corner_radius)
                } else {
                    vec![
                        rect.left_top(),
                        rect.right_top(),
                        rect.right_bottom(),
                        rect.left_bottom(),
                    ]
                };
                if let Some(shape) = crate::draw::gradient_path_mesh_with_transform(
                    &points,
                    &stops,
                    gradient.angle_deg,
                    true,
                    crate::draw::GradientPathGeometry {
                        center: gradient.center.map(offset_gradient_point),
                        focal_point: gradient.focal_point.map(offset_gradient_point),
                        radius: gradient.radius,
                        transform: gradient
                            .transform
                            .map(|matrix| offset_transform(matrix, origin)),
                    },
                ) {
                    shapes.push(shape);
                }
            } else {
                shapes.push(crate::draw::radial_gradient_rect_stops(rect, &stops, 48));
            }
        }
        (PaintSource::RadialGradient(gradient), Geometry::Path { points, closed }) => {
            if *closed {
                let points = offset_points(points, origin);
                if let Some(shape) = crate::draw::gradient_path_mesh_with_transform(
                    &points,
                    &gradient_stops(gradient, opacity, ui, blend_mode),
                    gradient.angle_deg,
                    true,
                    crate::draw::GradientPathGeometry {
                        center: gradient.center.map(offset_gradient_point),
                        focal_point: gradient.focal_point.map(offset_gradient_point),
                        radius: gradient.radius,
                        transform: gradient
                            .transform
                            .map(|matrix| offset_transform(matrix, origin)),
                    },
                ) {
                    shapes.push(shape);
                }
            }
        }
        (PaintSource::RadialGradient(gradient), Geometry::Ellipse { rect }) => {
            let points = ellipse_points(offset_rect(*rect, origin), 48);
            if let Some(shape) = crate::draw::gradient_path_mesh_with_transform(
                &points,
                &gradient_stops(gradient, opacity, ui, blend_mode),
                gradient.angle_deg,
                true,
                crate::draw::GradientPathGeometry {
                    center: gradient.center.map(offset_gradient_point),
                    focal_point: gradient.focal_point.map(offset_gradient_point),
                    radius: gradient.radius,
                    transform: gradient
                        .transform
                        .map(|matrix| offset_transform(matrix, origin)),
                },
            ) {
                shapes.push(shape);
            }
        }
        (PaintSource::Pattern(pattern), _) => {
            let points = geometry_to_polygon(geometry, origin);
            let foreground = resolve_color(ui, pattern.foreground, opacity, blend_mode);
            let background = resolve_color(ui, pattern.background, opacity, blend_mode);
            shapes.extend(crate::draw::pattern_fill_path(
                &points,
                pattern.seed,
                foreground,
                background,
                pattern.cell_size,
                pattern.mark_size,
            ));
        }
        (
            PaintSource::MeshGradient {
                corners,
                colors,
                subdivisions,
            },
            _,
        ) => {
            let mut blended_colors = *colors;
            for c in &mut blended_colors {
                *c = resolve_color(ui, *c, opacity, blend_mode);
            }
            shapes.push(crate::draw::mesh_gradient_patch(
                corners.map(|p| p + origin),
                blended_colors,
                *subdivisions,
            ));
        }
        (PaintSource::ProceduralNoise(noise), _) => {
            shapes.extend(crate::draw::noise_rect(
                offset_rect(geometry.bounds(), origin),
                noise.seed,
                noise.cell_size,
                noise.opacity * opacity,
            ));
        }
        _ => {}
    }
    shapes
}

fn paint_stroke(
    ui: &mut egui::Ui,
    origin: egui::Vec2,
    geometry: &Geometry,
    stroke: &StrokeLayer,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
    force_normal_blend: bool,
) -> Vec<egui::Shape> {
    let opacity = stroke.opacity * node_opacity;
    let blend_mode = if force_normal_blend {
        &BlendMode::Normal
    } else if stroke.blend_mode != BlendMode::Normal {
        &stroke.blend_mode
    } else {
        node_blend_mode
    };

    let color = match &stroke.paint {
        PaintSource::Solid(color) => resolve_color(ui, *color, opacity, blend_mode),
        _ => egui::Color32::TRANSPARENT,
    };
    if color == egui::Color32::TRANSPARENT || stroke.width <= 0.0 {
        return vec![];
    }
    let egui_stroke = egui::Stroke::new(stroke.width, color);
    let needs_rich_stroke = stroke.dash.is_some()
        || stroke.cap.is_some()
        || stroke.join.is_some()
        || stroke.miter_limit.is_some();
    let mut shapes = Vec::new();
    match geometry {
        Geometry::Rect {
            rect,
            corner_radius,
        } => {
            let rect = offset_rect(*rect, origin);
            if needs_rich_stroke {
                shapes.extend(stroke_path_shapes(
                    crate::draw::rounded_rect_path(rect, *corner_radius),
                    true,
                    stroke,
                    color,
                    egui_stroke,
                ));
            } else {
                shapes.push(egui::Shape::Rect(egui::epaint::RectShape::stroke(
                    rect,
                    *corner_radius,
                    egui_stroke,
                    egui::StrokeKind::Outside,
                )));
            }
        }
        Geometry::Ellipse { rect } => {
            let rect = offset_rect(*rect, origin);
            if needs_rich_stroke {
                shapes.extend(stroke_path_shapes(
                    ellipse_points(rect, 48),
                    true,
                    stroke,
                    color,
                    egui_stroke,
                ));
            } else {
                shapes.push(egui::Shape::ellipse_stroke(
                    rect.center(),
                    egui::vec2(rect.width() * 0.5, rect.height() * 0.5),
                    egui_stroke,
                ));
            }
        }
        Geometry::Path { points, closed } => {
            shapes.extend(stroke_path_shapes(
                offset_points(points, origin),
                *closed,
                stroke,
                color,
                egui_stroke,
            ));
        }
        _ => {}
    }
    shapes
}

fn draw_stroke_cap(cap: Option<&StrokeCap>) -> crate::draw::StrokeCap {
    match cap {
        Some(StrokeCap::Round) => crate::draw::StrokeCap::Round,
        Some(StrokeCap::Square) => crate::draw::StrokeCap::Square,
        _ => crate::draw::StrokeCap::Butt,
    }
}

fn stroke_path_shapes(
    mut points: Vec<egui::Pos2>,
    closed: bool,
    stroke: &StrokeLayer,
    color: egui::Color32,
    egui_stroke: egui::Stroke,
) -> Vec<egui::Shape> {
    let needs_rich_stroke = stroke.dash.is_some()
        || stroke.cap.is_some()
        || stroke.join.is_some()
        || stroke.miter_limit.is_some();
    if closed && points.len() > 2 && points.first() != points.last() {
        points.push(points[0]);
    }
    if needs_rich_stroke {
        let rich = crate::draw::RichStroke {
            width: stroke.width,
            color,
            dash: stroke.dash.as_ref().map(|dashes| crate::draw::DashPattern {
                dashes: dashes.clone(),
                offset: 0.0,
            }),
            cap: draw_stroke_cap(stroke.cap.as_ref()),
            join: draw_stroke_join(stroke.join.as_ref(), stroke.miter_limit),
        };
        crate::draw::dashed_path_shapes(&points, &rich)
    } else if closed {
        vec![egui::Shape::closed_line(points, egui_stroke)]
    } else {
        vec![egui::Shape::line(points, egui_stroke)]
    }
}

fn draw_stroke_join(
    join: Option<&StrokeJoin>,
    miter_limit: Option<f32>,
) -> crate::draw::StrokeJoin {
    match join {
        Some(StrokeJoin::Round) => crate::draw::StrokeJoin::Round,
        Some(StrokeJoin::Bevel) => crate::draw::StrokeJoin::Bevel,
        _ if miter_limit.is_some_and(|limit| limit <= 1.0) => crate::draw::StrokeJoin::Bevel,
        _ => crate::draw::StrokeJoin::Miter,
    }
}

fn paint_effect(
    ui: &egui::Ui,
    origin: egui::Vec2,
    geometry: &Geometry,
    effect: &EffectLayer,
    node_opacity: f32,
    node_blend_mode: &BlendMode,
    force_normal_blend: bool,
) -> Vec<egui::Shape> {
    let opacity = effect.opacity * node_opacity;
    let blend_mode = if force_normal_blend {
        &BlendMode::Normal
    } else if effect.blend_mode != BlendMode::Normal {
        &effect.blend_mode
    } else {
        node_blend_mode
    };

    let rect = offset_rect(geometry.bounds(), origin);
    let color = resolve_color(ui, effect.params.color, opacity, blend_mode);
    let mut shapes = Vec::new();

    match effect.effect_type {
        EffectType::DropShadow => {
            shapes.extend(crate::draw::box_shadow(
                rect,
                color,
                effect.params.blur,
                effect.params.spread,
                crate::draw::ShadowOffset::new(effect.params.x, effect.params.y),
            ));
        }
        EffectType::OuterGlow => {
            shapes.extend(crate::blur::soft_shadow(
                rect,
                color,
                effect.params.blur,
                0.0,
                crate::draw::ShadowOffset::zero(),
                crate::blur::BlurQuality::Medium,
            ));
        }
        EffectType::GaussianBlur | EffectType::Feather => {
            shapes.extend(crate::blur::soft_shadow(
                rect,
                color,
                effect.params.blur.max(effect.params.radius),
                0.0,
                crate::draw::ShadowOffset::zero(),
                crate::blur::BlurQuality::High,
            ));
        }
        EffectType::InnerShadow | EffectType::InnerGlow => {
            shapes.extend(crate::draw::inner_shadow(rect, color, effect.params.blur));
        }
        _ => {}
    }
    shapes
}

fn resolve_color(
    ui: &egui::Ui,
    color: egui::Color32,
    opacity: f32,
    blend_mode: &BlendMode,
) -> egui::Color32 {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let color = egui::Color32::from_rgba_unmultiplied(
        r,
        g,
        b,
        (a as f32 * opacity).clamp(0.0, 255.0) as u8,
    );
    if *blend_mode == BlendMode::Normal {
        color
    } else {
        crate::draw::blend_color(color, ui.visuals().window_fill(), blend_mode.clone())
    }
}

fn gradient_stops(
    gradient: &GradientDef,
    opacity: f32,
    ui: &egui::Ui,
    blend_mode: &BlendMode,
) -> Vec<(f32, egui::Color32)> {
    gradient
        .stops
        .iter()
        .map(|stop| {
            (
                stop.position,
                resolve_color(ui, stop.color, opacity, blend_mode),
            )
        })
        .collect()
}

fn sample_layout_path(points: &[crate::codegen::PathPoint], closed: bool) -> Vec<egui::Pos2> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![egui::pos2(points[0].anchor[0], points[0].anchor[1])];
    }
    let mut sampled = Vec::new();
    let segment_count = if closed {
        points.len()
    } else {
        points.len() - 1
    };
    for idx in 0..segment_count {
        let next_idx = (idx + 1) % points.len();
        let current = &points[idx];
        let next = &points[next_idx];
        let p0 = egui::pos2(current.anchor[0], current.anchor[1]);
        let p1 = egui::pos2(current.right_ctrl[0], current.right_ctrl[1]);
        let p2 = egui::pos2(next.left_ctrl[0], next.left_ctrl[1]);
        let p3 = egui::pos2(next.anchor[0], next.anchor[1]);
        if sampled.is_empty() {
            sampled.push(p0);
        }
        let is_line = p0.distance(p1) < 0.01 && p2.distance(p3) < 0.01;
        let steps = if is_line { 1 } else { 12 };
        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            sampled.push(cubic_bezier(p0, p1, p2, p3, t));
        }
    }
    sampled
}

fn cubic_bezier(
    p0: egui::Pos2,
    p1: egui::Pos2,
    p2: egui::Pos2,
    p3: egui::Pos2,
    t: f32,
) -> egui::Pos2 {
    let mt = 1.0 - t;
    let v = p0.to_vec2() * (mt * mt * mt)
        + p1.to_vec2() * (3.0 * mt * mt * t)
        + p2.to_vec2() * (3.0 * mt * t * t)
        + p3.to_vec2() * (t * t * t);
    egui::pos2(v.x, v.y)
}

fn offset_rect(rect: egui::Rect, origin: egui::Vec2) -> egui::Rect {
    rect.translate(origin)
}

fn offset_points(points: &[egui::Pos2], origin: egui::Vec2) -> Vec<egui::Pos2> {
    points.iter().map(|p| *p + origin).collect()
}

fn offset_transform(matrix: [f32; 6], origin: egui::Vec2) -> crate::draw::Transform2D {
    let [a, b, c, d, e, f] = matrix;
    crate::draw::Transform2D {
        a,
        b,
        c,
        d,
        e: origin.x + e - a * origin.x - c * origin.y,
        f: origin.y + f - b * origin.x - d * origin.y,
    }
}

fn rotate_geometry(geometry: &Geometry, angle_deg: f32) -> Geometry {
    let transform = crate::draw::Transform2D::rotate_around(angle_deg, geometry.bounds().center());
    match geometry {
        Geometry::Group { bounds } => Geometry::Group {
            bounds: transform.apply_to_rect(*bounds),
        },
        Geometry::Rect {
            rect,
            corner_radius,
        } => Geometry::Path {
            points: crate::draw::rounded_rect_path(*rect, *corner_radius)
                .into_iter()
                .map(|point| transform.apply(point))
                .collect(),
            closed: true,
        },
        Geometry::Ellipse { rect } => Geometry::Path {
            points: ellipse_points(*rect, 48)
                .into_iter()
                .map(|point| transform.apply(point))
                .collect(),
            closed: true,
        },
        Geometry::Path { points, closed } => Geometry::Path {
            points: points.iter().map(|point| transform.apply(*point)).collect(),
            closed: *closed,
        },
        Geometry::MeshPatch {
            corners,
            colors,
            subdivisions,
        } => Geometry::MeshPatch {
            corners: corners.map(|point| transform.apply(point)),
            colors: *colors,
            subdivisions: *subdivisions,
        },
    }
}

fn ellipse_points(rect: egui::Rect, segments: usize) -> Vec<egui::Pos2> {
    let center = rect.center();
    let rx = rect.width() * 0.5;
    let ry = rect.height() * 0.5;
    let segments = segments.max(adaptive_ellipse_segments(rect));
    (0..segments.max(3))
        .map(|idx| {
            let angle = std::f32::consts::TAU * idx as f32 / segments.max(3) as f32;
            center + egui::vec2(angle.cos() * rx, angle.sin() * ry)
        })
        .collect()
}

fn adaptive_ellipse_segments(rect: egui::Rect) -> usize {
    let rx = rect.width().abs() * 0.5;
    let ry = rect.height().abs() * 0.5;
    let perimeter_estimate =
        std::f32::consts::PI * (3.0 * (rx + ry) - ((3.0 * rx + ry) * (rx + 3.0 * ry)).sqrt());
    (perimeter_estimate / 4.0).ceil().clamp(48.0, 160.0) as usize
}

fn geometry_to_polygon(geometry: &Geometry, origin: egui::Vec2) -> Vec<egui::Pos2> {
    match geometry {
        Geometry::Rect {
            rect,
            corner_radius,
        } if *corner_radius > 0.001 => crate::draw::rounded_rect_path(*rect, *corner_radius)
            .into_iter()
            .map(|point| point + origin)
            .collect(),
        Geometry::Group { bounds } | Geometry::Rect { rect: bounds, .. } => {
            let r = offset_rect(*bounds, origin);
            vec![
                r.min,
                egui::pos2(r.max.x, r.min.y),
                r.max,
                egui::pos2(r.min.x, r.max.y),
            ]
        }
        Geometry::Ellipse { rect } => ellipse_points(offset_rect(*rect, origin), 48),
        Geometry::Path { points, .. } => offset_points(points, origin),
        Geometry::MeshPatch { corners, .. } => corners.map(|p| p + origin).to_vec(),
    }
}

fn bounds_for_points(points: &[egui::Pos2; 4]) -> egui::Rect {
    bounds_for_slice(points)
}

fn bounds_for_slice(points: &[egui::Pos2]) -> egui::Rect {
    if points.is_empty() {
        return egui::Rect::NOTHING;
    }
    let mut min = points[0];
    let mut max = points[0];
    for p in points.iter().skip(1) {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    egui::Rect::from_min_max(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appearance_stack_detects_offscreen_required_effects() {
        let stack = AppearanceStack {
            entries: vec![AppearanceEntry::Effect(EffectLayer {
                effect_type: EffectType::GaussianBlur,
                params: EffectDef {
                    effect_type: EffectType::GaussianBlur,
                    radius: 8.0,
                    ..EffectDef::default()
                },
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            })],
        };

        assert!(stack.requires_offscreen());
    }

    #[test]
    fn scene_node_rect_preserves_bounds() {
        let node = SceneNode::rect(
            "rect",
            egui::Rect::from_min_size(egui::pos2(1.0, 2.0), egui::vec2(3.0, 4.0)),
            2.0,
        );
        assert_eq!(node.geometry.bounds().min, egui::pos2(1.0, 2.0));
        assert_eq!(node.geometry.bounds().max, egui::pos2(4.0, 6.0));
    }

    #[test]
    fn scene_node_preserves_layout_rotation() {
        let mut elem = crate::codegen::LayoutElement::new(
            "rot".to_string(),
            crate::codegen::ElementType::Shape,
            0.0,
            0.0,
            10.0,
            20.0,
        );
        elem.rotation_deg = 45.0;
        let node = SceneNode::from_layout_element(&elem);
        assert_eq!(node.rotation_deg, 45.0);
    }

    #[test]
    fn scene_node_path_geometry_does_not_double_rotate() {
        let mut elem = crate::codegen::LayoutElement::new(
            "path_rot".to_string(),
            crate::codegen::ElementType::Shape,
            0.0,
            0.0,
            10.0,
            10.0,
        );
        elem.rotation_deg = 45.0;
        elem.path_closed = true;
        elem.path_points = vec![
            crate::codegen::PathPoint {
                anchor: [0.0, 0.0],
                left_ctrl: [0.0, 0.0],
                right_ctrl: [0.0, 0.0],
            },
            crate::codegen::PathPoint {
                anchor: [10.0, 0.0],
                left_ctrl: [10.0, 0.0],
                right_ctrl: [10.0, 0.0],
            },
            crate::codegen::PathPoint {
                anchor: [10.0, 10.0],
                left_ctrl: [10.0, 10.0],
                right_ctrl: [10.0, 10.0],
            },
        ];
        let node = SceneNode::from_layout_element(&elem);
        assert_eq!(node.rotation_deg, 0.0);
        assert!(matches!(node.geometry, Geometry::Path { .. }));
    }

    #[test]
    fn rotate_rect_geometry_converts_to_closed_path() {
        let geometry = Geometry::Rect {
            rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 10.0)),
            corner_radius: 4.0,
        };
        let rotated = rotate_geometry(&geometry, 30.0);
        let Geometry::Path { points, closed } = rotated else {
            panic!("expected rotated rect path");
        };
        assert!(closed);
        assert!(points.len() > 4);
    }

    #[test]
    fn bezier_layout_path_is_tessellated() {
        let points = vec![
            crate::codegen::PathPoint {
                anchor: [0.0, 0.0],
                left_ctrl: [0.0, 0.0],
                right_ctrl: [0.0, 10.0],
            },
            crate::codegen::PathPoint {
                anchor: [10.0, 0.0],
                left_ctrl: [10.0, 10.0],
                right_ctrl: [10.0, 0.0],
            },
        ];

        let sampled = sample_layout_path(&points, false);
        assert!(sampled.len() > points.len());
        assert!(sampled.iter().any(|p| p.y > 0.0));
    }

    #[test]
    fn path_gradient_fill_produces_mesh_shape() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let fill = FillLayer {
                    paint: PaintSource::LinearGradient(GradientDef {
                        gradient_type: crate::codegen::GradientType::Linear,
                        angle_deg: 0.0,
                        center: None,
                        focal_point: None,
                        radius: None,
                        transform: None,
                        stops: vec![
                            crate::codegen::GradientStop {
                                position: 0.0,
                                color: egui::Color32::RED,
                            },
                            crate::codegen::GradientStop {
                                position: 1.0,
                                color: egui::Color32::BLUE,
                            },
                        ],
                    }),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                };
                let geometry = Geometry::Path {
                    points: vec![
                        egui::pos2(0.0, 0.0),
                        egui::pos2(20.0, 0.0),
                        egui::pos2(20.0, 20.0),
                    ],
                    closed: true,
                };
                let shapes = paint_fill(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &fill,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(matches!(shapes.first(), Some(egui::Shape::Mesh(_))));
            });
        });
    }

    #[test]
    fn ellipse_gradient_fill_produces_mesh_shape() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let fill = FillLayer {
                    paint: PaintSource::RadialGradient(GradientDef {
                        gradient_type: crate::codegen::GradientType::Radial,
                        angle_deg: 0.0,
                        center: Some([10.0, 10.0]),
                        focal_point: Some([10.0, 10.0]),
                        radius: Some(10.0),
                        transform: None,
                        stops: vec![
                            crate::codegen::GradientStop {
                                position: 0.0,
                                color: egui::Color32::RED,
                            },
                            crate::codegen::GradientStop {
                                position: 1.0,
                                color: egui::Color32::BLUE,
                            },
                        ],
                    }),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                };
                let geometry = Geometry::Ellipse {
                    rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 20.0)),
                };
                let shapes = paint_fill(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &fill,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(matches!(shapes.first(), Some(egui::Shape::Mesh(_))));
            });
        });
    }

    #[test]
    fn pattern_fill_produces_editable_vector_shapes() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let fill = FillLayer {
                    paint: PaintSource::Pattern(PatternDef {
                        name: "dots".to_string(),
                        seed: 7,
                        foreground: egui::Color32::BLACK,
                        background: egui::Color32::TRANSPARENT,
                        cell_size: 4.0,
                        mark_size: 1.0,
                    }),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                };
                let geometry = Geometry::Rect {
                    rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 20.0)),
                    corner_radius: 4.0,
                };
                let shapes = paint_fill(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &fill,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(shapes.len() > 1);
                assert!(shapes.iter().any(|shape| matches!(
                    shape,
                    egui::Shape::LineSegment { .. } | egui::Shape::Circle(_)
                )));
            });
        });
    }

    #[test]
    fn dashed_rect_stroke_uses_rich_path_shapes() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let stroke = StrokeLayer {
                    paint: PaintSource::Solid(egui::Color32::BLACK),
                    width: 2.0,
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    cap: Some(StrokeCap::Round),
                    join: Some(StrokeJoin::Bevel),
                    dash: Some(vec![2.0, 2.0]),
                    miter_limit: Some(1.0),
                };
                let geometry = Geometry::Rect {
                    rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 20.0)),
                    corner_radius: 4.0,
                };
                let shapes = paint_stroke(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &stroke,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(shapes.len() > 1);
                assert!(shapes
                    .iter()
                    .all(|shape| !matches!(shape, egui::Shape::Rect(_))));
            });
        });
    }

    #[test]
    fn dashed_ellipse_stroke_uses_rich_path_shapes() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let stroke = StrokeLayer {
                    paint: PaintSource::Solid(egui::Color32::BLACK),
                    width: 2.0,
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    cap: Some(StrokeCap::Round),
                    join: Some(StrokeJoin::Round),
                    dash: Some(vec![2.0, 2.0]),
                    miter_limit: None,
                };
                let geometry = Geometry::Ellipse {
                    rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 12.0)),
                };
                let shapes = paint_stroke(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &stroke,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(shapes.len() > 1);
                assert!(shapes
                    .iter()
                    .all(|shape| !matches!(shape, egui::Shape::Rect(_))));
            });
        });
    }

    #[test]
    fn test_render_scene_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let scene = ArtboardScene::default();
                render_scene(ui, &scene);
            });
        });
    }

    #[test]
    fn nested_clip_children_preserve_descendant_layers_with_masks() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let mut outer = SceneNode::rect(
                    "outer_clip",
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(20.0)),
                    0.0,
                );
                outer.clip_children = true;

                let mut inner = SceneNode::rect(
                    "inner_clip",
                    egui::Rect::from_min_size(egui::pos2(2.0, 2.0), egui::Vec2::splat(10.0)),
                    0.0,
                );
                inner.clip_children = true;

                let mut child = SceneNode::rect(
                    "child",
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(30.0)),
                    0.0,
                );
                child
                    .appearance
                    .entries
                    .push(AppearanceEntry::Fill(FillLayer {
                        paint: PaintSource::Solid(egui::Color32::RED),
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                    }));

                inner.children.push(child);
                outer.children.push(inner);

                let mut layers = Vec::new();
                collect_node_layers(ui, egui::Vec2::ZERO, &outer, 1.0, &mut layers);
                assert_eq!(layers.len(), 1);
                assert_eq!(layers[0].clip_polygons.len(), 2);
            });
        });
    }

    #[test]
    fn test_render_geometry_appearance_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let geometry = Geometry::Rect {
                    rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                    corner_radius: 0.0,
                };
                let appearance = AppearanceStack::default();
                let painter = ui.painter().clone();
                render_geometry_appearance(
                    ui,
                    &painter,
                    egui::Vec2::ZERO,
                    &geometry,
                    &appearance,
                    1.0,
                    &BlendMode::Normal,
                );
            });
        });
    }

    #[test]
    fn test_paint_effect_drop_shadow() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let geometry = Geometry::Rect {
                    rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                    corner_radius: 0.0,
                };
                let effect = EffectLayer {
                    effect_type: EffectType::DropShadow,
                    params: EffectDef::default(),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                };
                let shapes = paint_effect(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &effect,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(!shapes.is_empty());
            });
        });
    }

    #[test]
    fn test_scene_group_opacity_and_blend_mode() {
        let mut scene = ArtboardScene::default();
        let mut group = SceneNode::rect(
            "group",
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0)),
            0.0,
        );
        group.opacity = 0.5;
        group.blend_mode = BlendMode::Multiply;

        let mut child = SceneNode::rect(
            "child",
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(50.0)),
            0.0,
        );
        child
            .appearance
            .entries
            .push(AppearanceEntry::Fill(FillLayer {
                paint: PaintSource::Solid(egui::Color32::RED),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            }));

        group.children.push(child);
        scene.nodes.push(group);

        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                render_scene(ui, &scene);
            });
        });
    }
}
