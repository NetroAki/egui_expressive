use super::*;

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
            let mask = crate::draw::ClipMask::from_polygon(polygon);
            crate::draw::clipped_layers_mask(ui, &mask, layers);
        }
    } else {
        for child in &node.children {
            render_node(ui, painter, origin, child, effective_opacity);
        }
    }
}

pub(crate) fn collect_node_layers(
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
            layer.clip_polygons.push(polygon.clone());
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
