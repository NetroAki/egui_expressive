//! Raster-to-vector conversion for code-only export paths.
//!
//! This module is intentionally export-time only: it converts pixel input into
//! retained vector scene nodes. It must never introduce runtime image slots or
//! baked raster dependencies into generated UI code.

use crate::scene::{PaintSource, SceneNode};
use image::RgbaImage;
use std::path::Path;
use visioncortex::{CompoundPathElement, PointF64, PointI32};

/// Configuration for tracing a raster image into vector scene nodes.
#[derive(Clone, Debug)]
pub struct RasterVectorizeConfig {
    /// Destination bounds for the vectorized geometry. When omitted, output uses
    /// source-pixel coordinates.
    pub fit_rect: Option<egui::Rect>,
    /// vtracer tracing configuration.
    pub trace: vtracer::Config,
    /// Number of straight segments used when flattening each cubic Bézier curve
    /// into scene path points. The result is still vector geometry.
    pub curve_samples: usize,
    /// Optional safety cap for generated scene nodes.
    pub max_nodes: Option<usize>,
}

impl Default for RasterVectorizeConfig {
    fn default() -> Self {
        Self {
            fit_rect: None,
            trace: vtracer::Config::from_preset(vtracer::Preset::Poster),
            curve_samples: 12,
            max_nodes: Some(512),
        }
    }
}

/// Load an image file and trace it into vector scene nodes.
pub fn vectorize_image_file_to_scene_nodes(
    id_prefix: &str,
    path: &Path,
    config: &RasterVectorizeConfig,
) -> Result<Vec<SceneNode>, String> {
    let image = image::open(path)
        .map_err(|err| format!("could not read raster image {}: {err}", path.display()))?
        .to_rgba8();
    vectorize_rgba_to_scene_nodes(id_prefix, &image, config)
}

/// Trace an RGBA image into vector scene nodes.
pub fn vectorize_rgba_to_scene_nodes(
    id_prefix: &str,
    image: &RgbaImage,
    config: &RasterVectorizeConfig,
) -> Result<Vec<SceneNode>, String> {
    let width = usize::try_from(image.width()).map_err(|_| "image width too large".to_string())?;
    let height =
        usize::try_from(image.height()).map_err(|_| "image height too large".to_string())?;
    if width == 0 || height == 0 {
        return Ok(Vec::new());
    }

    let color_image = vtracer::ColorImage {
        pixels: image.as_raw().clone(),
        width,
        height,
    };
    let svg = vtracer::convert(color_image, config.trace.clone())?;
    svg_paths_to_scene_nodes(id_prefix, &svg, config)
}

fn svg_paths_to_scene_nodes(
    id_prefix: &str,
    svg: &vtracer::SvgFile,
    config: &RasterVectorizeConfig,
) -> Result<Vec<SceneNode>, String> {
    let source_w = svg.width.max(1) as f32;
    let source_h = svg.height.max(1) as f32;
    let target = config.fit_rect.unwrap_or_else(|| {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(source_w, source_h))
    });
    let mapper = |x: f64, y: f64| {
        egui::pos2(
            target.min.x + (x as f32 / source_w) * target.width(),
            target.min.y + (y as f32 / source_h) * target.height(),
        )
    };

    let mut nodes = Vec::new();
    for (path_idx, svg_path) in svg.paths.iter().enumerate() {
        if svg_path.color.a == 0 {
            continue;
        }
        let mut contours = Vec::new();
        for element in svg_path.path.iter() {
            if let Some(contour) = contour_from_compound_element(element, &mapper, config) {
                contours.push(contour);
            }
        }
        if contours.is_empty() {
            continue;
        }

        let fill = PaintSource::Solid(egui::Color32::from_rgba_unmultiplied(
            svg_path.color.r,
            svg_path.color.g,
            svg_path.color.b,
            svg_path.color.a,
        ));
        for (contour_idx, contour) in contours.into_iter().enumerate() {
            let id = format!("{id_prefix}_trace_{path_idx}_{contour_idx}");
            nodes.push(SceneNode::path(id, contour, true).with_fill(fill.clone()));
        }

        if let Some(max_nodes) = config.max_nodes {
            if nodes.len() > max_nodes {
                return Err(format!(
                    "raster vectorization exceeded max_nodes={max_nodes} for {id_prefix}"
                ));
            }
        }
    }
    Ok(nodes)
}

fn contour_from_compound_element(
    element: &CompoundPathElement,
    mapper: &impl Fn(f64, f64) -> egui::Pos2,
    config: &RasterVectorizeConfig,
) -> Option<Vec<egui::Pos2>> {
    let mut points = match element {
        CompoundPathElement::PathI32(path) => path
            .path
            .iter()
            .map(|point| map_i32(*point, mapper))
            .collect::<Vec<_>>(),
        CompoundPathElement::PathF64(path) => path
            .path
            .iter()
            .map(|point| map_f64(*point, mapper))
            .collect::<Vec<_>>(),
        CompoundPathElement::Spline(spline) => sample_spline(spline, mapper, config.curve_samples),
    };
    close_contour(&mut points);
    if points.len() < 3 {
        return None;
    }
    Some(points)
}

fn map_i32(point: PointI32, mapper: &impl Fn(f64, f64) -> egui::Pos2) -> egui::Pos2 {
    mapper(f64::from(point.x), f64::from(point.y))
}

fn map_f64(point: PointF64, mapper: &impl Fn(f64, f64) -> egui::Pos2) -> egui::Pos2 {
    mapper(point.x, point.y)
}

fn sample_spline(
    spline: &visioncortex::Spline,
    mapper: &impl Fn(f64, f64) -> egui::Pos2,
    curve_samples: usize,
) -> Vec<egui::Pos2> {
    let points = &spline.points;
    if points.is_empty() || !(points.len() - 1).is_multiple_of(3) {
        return Vec::new();
    }
    let samples = curve_samples.max(1);
    let mut out = vec![map_f64(points[0], mapper)];
    let mut idx = 1;
    while idx + 2 < points.len() {
        let p0 = points[idx - 1];
        let p1 = points[idx];
        let p2 = points[idx + 1];
        let p3 = points[idx + 2];
        for step in 1..=samples {
            let t = step as f64 / samples as f64;
            let point = cubic(p0, p1, p2, p3, t);
            out.push(map_f64(point, mapper));
        }
        idx += 3;
    }
    out
}

fn cubic(p0: PointF64, p1: PointF64, p2: PointF64, p3: PointF64, t: f64) -> PointF64 {
    let mt = 1.0 - t;
    PointF64 {
        x: p0.x * mt * mt * mt
            + 3.0 * p1.x * mt * mt * t
            + 3.0 * p2.x * mt * t * t
            + p3.x * t * t * t,
        y: p0.y * mt * mt * mt
            + 3.0 * p1.y * mt * mt * t
            + 3.0 * p2.y * mt * t * t
            + p3.y * t * t * t,
    }
}

fn close_contour(points: &mut Vec<egui::Pos2>) {
    while points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
}
