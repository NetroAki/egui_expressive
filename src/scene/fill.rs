use super::*;

pub(crate) fn paint_fill(
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
