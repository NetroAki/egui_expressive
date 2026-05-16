use crate::scene::PathContour;
use egui::{Color32, Pos2, Rect};

use super::core::{shaped_glyph_run_advance_width, TypeSpec};
use super::shaping::shape_text_with_font_bytes;

pub fn render_shaped_glyph_run(
    painter: &egui::Painter,
    pos: Pos2,
    run: &super::core::ShapedGlyphRun,
    spec: &TypeSpec,
) -> Rect {
    let mut contour_bounds_total = None::<Rect>;
    let origin = egui::pos2(pos.x, pos.y - spec.baseline_shift);
    let mut cursor = origin;
    let mut rendered_contours = false;
    let uses_absolute_contours = run
        .glyphs
        .iter()
        .any(|glyph| glyph.contours_are_absolute && !glyph.contours.is_empty());

    for glyph in &run.glyphs {
        let glyph_origin = if glyph.contours_are_absolute {
            pos
        } else {
            egui::pos2(
                cursor.x + glyph.offset_x * spec.horizontal_scale,
                cursor.y + glyph.offset_y * spec.vertical_scale,
            )
        };
        if !glyph.contours.is_empty() {
            if let Some((shape, glyph_bounds)) = contour_mesh(
                glyph_origin,
                &glyph.contours,
                spec.color.unwrap_or(Color32::BLACK),
            ) {
                painter.add(shape);
                contour_bounds_total = rect_union(contour_bounds_total, glyph_bounds);
                rendered_contours = true;
            }
        }
        cursor.x += glyph.advance_x.max(0.0) * spec.horizontal_scale;
        cursor.y += glyph.advance_y.max(0.0) * spec.vertical_scale;
    }

    if rendered_contours {
        if uses_absolute_contours {
            return contour_bounds_total
                .unwrap_or_else(|| Rect::from_min_size(pos, egui::Vec2::ZERO));
        }
        let advance_bounds = Rect::from_min_size(
            origin,
            egui::vec2(
                shaped_glyph_run_advance_width(run, spec),
                spec.effective_size(),
            ),
        );
        contour_bounds_total
            .unwrap_or(advance_bounds)
            .union(advance_bounds)
    } else {
        let width = shaped_glyph_run_advance_width(run, spec);
        let fallback = super::text::render_text(painter, pos, &run.text, spec, None);
        if width > 0.0 {
            fallback.union(Rect::from_min_size(
                origin,
                egui::vec2(width, spec.effective_size()),
            ))
        } else {
            fallback
        }
    }
}

pub fn render_text_with_font_bytes(
    painter: &egui::Painter,
    pos: Pos2,
    text: &str,
    spec: &TypeSpec,
    font_data: &[u8],
) -> Option<Rect> {
    // `rustybuzz` shaping yields glyph ids/advances but not vector outlines.
    // Return `None` when no contour data is present rather than pretending to
    // render the supplied font bytes through egui's current UI font fallback.
    let run = shape_text_with_font_bytes(font_data, text, spec)?;
    let has_contours = run.glyphs.iter().any(|glyph| !glyph.contours.is_empty());
    has_contours.then(|| render_shaped_glyph_run(painter, pos, &run, spec))
}

fn rect_union(a: Option<Rect>, b: Rect) -> Option<Rect> {
    Some(match a {
        Some(existing) => existing.union(b),
        None => b,
    })
}

fn contour_bounds(points: &[Pos2]) -> Option<Rect> {
    let mut min = None::<Pos2>;
    let mut max = None::<Pos2>;
    for point in points {
        min = Some(match min {
            Some(acc) => egui::pos2(acc.x.min(point.x), acc.y.min(point.y)),
            None => *point,
        });
        max = Some(match max {
            Some(acc) => egui::pos2(acc.x.max(point.x), acc.y.max(point.y)),
            None => *point,
        });
    }
    match (min, max) {
        (Some(min), Some(max)) => Some(Rect::from_min_max(min, max)),
        _ => None,
    }
}

fn contour_mesh(
    origin: Pos2,
    contours: &[PathContour],
    color: Color32,
) -> Option<(egui::Shape, Rect)> {
    use lyon_tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
    };

    let mut path_builder = lyon_tessellation::path::Path::builder();
    let mut bounds = None::<Rect>;
    let mut has_contour = false;

    for contour in contours {
        if !contour.closed || contour.points.len() < 3 {
            continue;
        }
        let mut points: Vec<Pos2> = contour
            .points
            .iter()
            .map(|point| *point + origin.to_vec2())
            .collect();
        if points.len() > 3 && points.first() == points.last() {
            points.pop();
        }
        let Some(first) = points.first().copied() else {
            continue;
        };
        bounds = rect_union(
            bounds,
            contour_bounds(&points).unwrap_or_else(|| Rect::from_min_max(first, first)),
        );
        path_builder.begin(lyon_tessellation::math::point(first.x, first.y));
        for point in points.iter().skip(1) {
            path_builder.line_to(lyon_tessellation::math::point(point.x, point.y));
        }
        path_builder.end(true);
        has_contour = true;
    }

    if !has_contour {
        return None;
    }

    let path = path_builder.build();
    let mut geometry: VertexBuffers<egui::epaint::Vertex, u16> = VertexBuffers::new();
    FillTessellator::new()
        .tessellate_path(
            &path,
            &FillOptions::default().with_fill_rule(lyon_tessellation::FillRule::EvenOdd),
            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| egui::epaint::Vertex {
                pos: egui::pos2(vertex.position().x, vertex.position().y),
                uv: egui::epaint::WHITE_UV,
                color,
            }),
        )
        .ok()?;

    if geometry.vertices.is_empty() || geometry.indices.is_empty() {
        return None;
    }

    let mesh = egui::epaint::Mesh {
        vertices: geometry.vertices,
        indices: geometry
            .indices
            .into_iter()
            .map(|index| index as u32)
            .collect(),
        ..Default::default()
    };
    Some((
        egui::Shape::mesh(mesh),
        bounds.unwrap_or_else(|| Rect::from_min_size(origin, egui::vec2(0.0, 0.0))),
    ))
}
