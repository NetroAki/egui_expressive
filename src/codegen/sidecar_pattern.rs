use super::*;

pub fn parse_color_value(value: &serde_json::Value) -> Option<Color32> {
    fn rgba_from_obj(obj: &serde_json::Map<String, serde_json::Value>) -> Option<Color32> {
        let r = obj.get("r")?.as_f64()?.round().clamp(0.0, 255.0) as u8;
        let g = obj.get("g")?.as_f64()?.round().clamp(0.0, 255.0) as u8;
        let b = obj.get("b")?.as_f64()?.round().clamp(0.0, 255.0) as u8;
        let a = obj
            .get("a")
            .and_then(|v| v.as_f64())
            .map(|a| if a <= 1.0 { a * 255.0 } else { a })
            .unwrap_or(255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        Some(Color32::from_rgba_unmultiplied(r, g, b, a))
    }
    if let Some(c_str) = value.as_str() {
        return crate::svg::parse_svg_color(c_str);
    }
    value.as_object().and_then(|obj| {
        if let Some(c_str) = obj.get("color").and_then(|v| v.as_str()) {
            crate::svg::parse_svg_color(c_str)
        } else if let Some(color_obj) = obj.get("color").and_then(|v| v.as_object()) {
            rgba_from_obj(color_obj)
        } else {
            rgba_from_obj(obj)
        }
    })
}

pub fn parse_gradient(v: &serde_json::Value) -> Option<GradientDef> {
    let g = v.as_object()?;
    let type_name = g.get("type").and_then(|t| t.as_str());
    let parse_point = |value: Option<&serde_json::Value>| -> Option<[f32; 2]> {
        let value = value?;
        if let Some(arr) = value.as_array() {
            return Some([arr.first()?.as_f64()? as f32, arr.get(1)?.as_f64()? as f32]);
        }
        let obj = value.as_object()?;
        Some([
            obj.get("x")?.as_f64()? as f32,
            obj.get("y")?.as_f64()? as f32,
        ])
    };
    let parse_transform = |value: Option<&serde_json::Value>| -> Option<[f32; 6]> {
        let value = value?;
        if let Some(arr) = value.as_array() {
            return Some([
                arr.first()?.as_f64()? as f32,
                arr.get(1)?.as_f64()? as f32,
                arr.get(2)?.as_f64()? as f32,
                arr.get(3)?.as_f64()? as f32,
                arr.get(4)?.as_f64()? as f32,
                arr.get(5)?.as_f64()? as f32,
            ]);
        }
        let obj = value.as_object()?;
        let number = |names: &[&str]| -> Option<f32> {
            names
                .iter()
                .find_map(|name| obj.get(*name).and_then(|v| v.as_f64()))
                .map(|v| v as f32)
        };
        Some([
            number(&["a", "mValueA"])?,
            number(&["b", "mValueB"])?,
            number(&["c", "mValueC"])?,
            number(&["d", "mValueD"])?,
            number(&["e", "tx", "mValueTX"])?,
            number(&["f", "ty", "mValueTY"])?,
        ])
    };
    let gradient_type = match type_name {
        Some("radial") => GradientType::Radial,
        Some("linear") | None => GradientType::Linear,
        Some(_) => return None,
    };
    let angle_deg = g
        .get("angle")
        .and_then(|a| a.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.0);
    let stops = g
        .get("stops")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|stop| {
                    let position = stop.get("position")?.as_f64()? as f32;
                    let color = stop
                        .get("color")?
                        .as_str()
                        .and_then(crate::svg::parse_svg_color)
                        .unwrap_or(egui::Color32::BLACK);
                    let opacity = stop
                        .get("opacity")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0)
                        .clamp(0.0, 1.0) as f32;
                    let [r, g, b, a] = color.to_srgba_unmultiplied();
                    let color = Color32::from_rgba_unmultiplied(
                        r,
                        g,
                        b,
                        (a as f32 * opacity).round() as u8,
                    );
                    Some(GradientStop { position, color })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(GradientDef {
        gradient_type,
        angle_deg,
        center: parse_point(g.get("center")),
        focal_point: parse_point(g.get("focalPoint").or_else(|| g.get("focal_point"))),
        radius: g.get("radius").and_then(|r| r.as_f64()).map(|r| r as f32),
        transform: parse_transform(g.get("transform").or_else(|| g.get("matrix"))),
        stops,
    })
}

fn parse_pattern_tile_shapes(
    value: Option<&serde_json::Value>,
) -> Vec<crate::scene::PatternTileShape> {
    let Some(array) = value.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let parse_pos = |value: Option<&serde_json::Value>| -> Option<egui::Pos2> {
        let value = value?;
        if let Some(arr) = value.as_array() {
            return Some(egui::pos2(
                arr.first()?.as_f64()? as f32,
                arr.get(1)?.as_f64()? as f32,
            ));
        }
        let obj = value.as_object()?;
        if let (Some(x), Some(y)) = (obj.get("x"), obj.get("y")) {
            return Some(egui::pos2(x.as_f64()? as f32, y.as_f64()? as f32));
        }
        None
    };
    let parse_tile_point = |point: &serde_json::Value| -> Option<crate::scene::PatternTilePoint> {
        if let Some(arr) = point.as_array() {
            let anchor = egui::pos2(arr.first()?.as_f64()? as f32, arr.get(1)?.as_f64()? as f32);
            return Some(crate::scene::PatternTilePoint {
                anchor,
                left_ctrl: anchor,
                right_ctrl: anchor,
            });
        }
        let obj = point.as_object()?;
        let anchor = parse_pos(
            obj.get("anchor")
                .or_else(|| obj.get("anchorPoint"))
                .or(Some(point)),
        )?;
        let left_ctrl = parse_pos(
            obj.get("leftDir")
                .or_else(|| obj.get("left_dir"))
                .or_else(|| obj.get("leftCtrl"))
                .or_else(|| obj.get("left_ctrl")),
        )
        .unwrap_or(anchor);
        let right_ctrl = parse_pos(
            obj.get("rightDir")
                .or_else(|| obj.get("right_dir"))
                .or_else(|| obj.get("rightCtrl"))
                .or_else(|| obj.get("right_ctrl")),
        )
        .unwrap_or(anchor);
        Some(crate::scene::PatternTilePoint {
            anchor,
            left_ctrl,
            right_ctrl,
        })
    };
    array
        .iter()
        .filter_map(|shape| {
            let obj = shape.as_object()?;
            let points = obj
                .get("points")?
                .as_array()?
                .iter()
                .filter_map(parse_tile_point)
                .collect::<Vec<_>>();
            if points.len() < 2 {
                return None;
            }
            Some(crate::scene::PatternTileShape {
                points,
                closed: obj.get("closed").and_then(|v| v.as_bool()).unwrap_or(true),
                fill: obj.get("fill").and_then(parse_color_value),
                stroke: obj.get("stroke").and_then(parse_color_value),
                stroke_width: obj
                    .get("strokeWidth")
                    .or_else(|| obj.get("stroke_width"))
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32)
                    .unwrap_or(0.0)
                    .max(0.0),
            })
        })
        .collect()
}

pub fn parse_pattern(v: &serde_json::Value) -> Option<crate::scene::PatternDef> {
    if let Some(name) = v.as_str() {
        let seed = stable_pattern_seed(name);
        let (foreground, background) = seeded_pattern_colors(seed);
        return Some(crate::scene::PatternDef {
            name: name.to_string(),
            seed,
            foreground,
            background,
            cell_size: 8.0,
            mark_size: 1.0,
            rotation_deg: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            transform: None,
            tile_shapes: Vec::new(),
        });
    }
    let g = v.as_object()?;
    let type_name = g.get("type").and_then(|t| t.as_str());
    match type_name {
        Some("linear") | Some("radial") => return None,
        Some(_) => {}
        None => {
            let has_pattern_metadata = g.contains_key("patternName")
                || g.contains_key("pattern_name")
                || g.contains_key("name")
                || g.contains_key("seed")
                || g.contains_key("cellSize")
                || g.contains_key("cell_size");
            if !has_pattern_metadata {
                return None;
            }
        }
    }
    let name = g
        .get("patternName")
        .or_else(|| g.get("pattern_name"))
        .or_else(|| g.get("name"))
        .and_then(|v| v.as_str())
        .or(type_name)
        .unwrap_or("pattern")
        .to_string();
    let seed = g
        .get("seed")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or_else(|| stable_pattern_seed(&name));
    let (seeded_foreground, seeded_background) = seeded_pattern_colors(seed);
    let foreground = g
        .get("foreground")
        .or_else(|| g.get("fg"))
        .and_then(parse_color_value)
        .unwrap_or(seeded_foreground);
    let background = g
        .get("background")
        .or_else(|| g.get("bg"))
        .and_then(parse_color_value)
        .unwrap_or(seeded_background);
    let cell_size = g
        .get("cellSize")
        .or_else(|| g.get("cell_size"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(8.0)
        .clamp(2.0, 64.0);
    let mark_size = g
        .get("markSize")
        .or_else(|| g.get("mark_size"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(1.0)
        .clamp(0.5, 16.0);
    let rotation_deg = g
        .get("rotationDeg")
        .or_else(|| g.get("rotation_deg"))
        .or_else(|| g.get("rotation"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let scale = g
        .get("scale")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            let sx = arr.first()?.as_f64()? as f32;
            let sy = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(sx as f64) as f32;
            Some((sx, sy))
        })
        .unwrap_or_else(|| {
            let sx = g
                .get("scaleX")
                .or_else(|| g.get("scale_x"))
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0);
            let sy = g
                .get("scaleY")
                .or_else(|| g.get("scale_y"))
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(sx);
            (sx, sy)
        });
    let offset = g
        .get("offset")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            let ox = arr.first()?.as_f64()? as f32;
            let oy = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            Some((ox, oy))
        })
        .unwrap_or_else(|| {
            let ox = g
                .get("offsetX")
                .or_else(|| g.get("offset_x"))
                .or_else(|| g.get("translateX"))
                .or_else(|| g.get("translate_x"))
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(0.0);
            let oy = g
                .get("offsetY")
                .or_else(|| g.get("offset_y"))
                .or_else(|| g.get("translateY"))
                .or_else(|| g.get("translate_y"))
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(0.0);
            (ox, oy)
        });
    let transform = g
        .get("matrix")
        .or_else(|| g.get("transform"))
        .or_else(|| g.get("patternMatrix"))
        .or_else(|| g.get("pattern_matrix"))
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            if arr.len() < 6 {
                return None;
            }
            Some([
                arr.first()?.as_f64()? as f32,
                arr.get(1)?.as_f64()? as f32,
                arr.get(2)?.as_f64()? as f32,
                arr.get(3)?.as_f64()? as f32,
                arr.get(4)?.as_f64()? as f32,
                arr.get(5)?.as_f64()? as f32,
            ])
        });
    Some(crate::scene::PatternDef {
        name,
        seed,
        foreground,
        background,
        cell_size,
        mark_size,
        rotation_deg,
        scale_x: scale.0,
        scale_y: scale.1,
        offset_x: offset.0,
        offset_y: offset.1,
        transform,
        tile_shapes: parse_pattern_tile_shapes(
            g.get("tileGeometry")
                .or_else(|| g.get("tile_geometry"))
                .or_else(|| g.get("tileShapes"))
                .or_else(|| g.get("tile_shapes")),
        ),
    })
}
