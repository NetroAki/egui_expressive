use super::*;

#[test]
fn test_parse_json_sidecar_recursive_children() {
    let json = r#"{
            "artboard": { "name": "Test", "width": 100, "height": 100 },
            "elements": [{
                "id": "parent",
                "type": "group",
                "children": [{
                    "id": "child",
                    "type": "text",
                    "text": "Hello"
                }]
            }]
        }"#;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].id, "parent");
    assert_eq!(elements[0].children.len(), 1);
    assert_eq!(elements[0].children[0].id, "child");
    assert_eq!(elements[0].children[0].text.as_deref(), Some("Hello"));
}

#[test]
fn test_parse_json_sidecar_preserves_ellipse_geometry() {
    let json = r##"{
            "artboard": { "name": "Test", "width": 100, "height": 100 },
            "elements": [{ "id": "ell", "type": "ellipse", "x": 10, "y": 20, "w": 30, "h": 40, "fill": "#ff0000" }]
        }"##;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    assert_eq!(elements[0].el_type, ElementType::Ellipse);
    let node = crate::scene::SceneNode::from_layout_element(&elements[0]);
    assert!(matches!(
        node.geometry,
        crate::scene::Geometry::Ellipse { .. }
    ));
}

#[test]
fn test_parse_json_sidecar_appearance_stack() {
    let json = r##"{
            "artboard": { "name": "Test", "width": 100, "height": 100 },
            "elements": [{
                "id": "el",
                "type": "shape",
                "appearanceStack": [
                    { "type": "fill", "color": "#ff0000", "opacity": 0.5, "blendMode": "multiply",
                      "gradient": { "type": "linear", "angle": 45, "transform": [1, 0, 0, 1, 2, 3], "stops": [{ "position": 0.0, "color": "#ff0000", "opacity": 0.25 }, { "position": 1.0, "color": "#0000ff" }] } },
                    { "type": "stroke", "r": 0, "g": 255, "b": 0, "width": 2.0, "opacity": 1.0, "blendMode": "screen", "cap": "round", "join": "bevel", "dash": [2, 4], "miterLimit": 1.0,
                      "gradient": { "type": "linear", "angle": 0, "stops": [{ "position": 0.0, "color": "#00ff00" }, { "position": 1.0, "color": "#0000ff" }] } }
                ]
            }]
        }"##;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    let stack = &elements[0].appearance_stack.entries;
    assert_eq!(stack.len(), 2);
    match &stack[0] {
        crate::scene::AppearanceEntry::Fill(f) => {
            let crate::scene::PaintSource::LinearGradient(gradient) = &f.paint else {
                panic!("Expected LinearGradient");
            };
            assert_eq!(gradient.stops[0].color.to_srgba_unmultiplied()[3], 64);
            assert_eq!(gradient.transform, Some([1.0, 0.0, 0.0, 1.0, 2.0, 3.0]));
            assert_eq!(f.opacity, 0.5);
            assert_eq!(f.blend_mode, BlendMode::Multiply);
        }
        _ => panic!("Expected Fill"),
    }
    match &stack[1] {
        crate::scene::AppearanceEntry::Stroke(s) => {
            assert!(matches!(
                s.paint,
                crate::scene::PaintSource::LinearGradient(_)
            ));
            assert_eq!(s.width, 2.0);
            assert_eq!(s.blend_mode, BlendMode::Screen);
            assert_eq!(s.cap, Some(StrokeCap::Round));
            assert_eq!(s.join, Some(StrokeJoin::Bevel));
            assert_eq!(s.dash.as_deref(), Some(&[2.0, 4.0][..]));
            assert_eq!(s.miter_limit, Some(1.0));
        }
        _ => panic!("Expected Stroke"),
    }
}

#[test]
fn test_parse_json_sidecar_pattern_fill_uses_scene_pattern_source() {
    let json = r##"{
            "artboard": { "name": "Test", "width": 100, "height": 100 },
            "elements": [{
                "id": "pattern_rect",
                "type": "shape",
                "x": 0, "y": 0, "w": 20, "h": 20,
                "gradient": { "type": "conic", "patternName": "Diagonal Dots", "seed": 123, "cellSize": 10.0, "markSize": 2.0 },
                "stroke": "#000000", "strokeWidth": 1.0
            }]
        }"##;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    let stack = &elements[0].appearance_stack.entries;
    assert_eq!(stack.len(), 2);
    let crate::scene::AppearanceEntry::Fill(fill) = &stack[0] else {
        panic!("Expected pattern fill");
    };
    let crate::scene::PaintSource::Pattern(pattern) = &fill.paint else {
        panic!("Expected Pattern paint source");
    };
    assert_eq!(pattern.name, "Diagonal Dots");
    assert_eq!(pattern.seed, 123);
    assert_eq!(pattern.cell_size, 10.0);
    assert_eq!(pattern.mark_size, 2.0);

    let token_map = std::collections::HashMap::new();
    let code = generate_artboard_file("Pattern", 100.0, 100.0, &elements, &token_map);
    assert!(code.contains("PaintSource::Pattern"));
    assert!(code.contains("PatternDef"));
}

#[test]
fn test_parse_json_sidecar_appearance_fills_pattern_uses_scene_stack() {
    let json = r##"{
            "artboard": { "name": "Test", "width": 100, "height": 100 },
            "elements": [{
                "id": "pattern_appearance",
                "type": "shape",
                "appearanceFills": [
                    {
                        "opacity": 0.75,
                        "pattern": { "patternName": "Dots", "seed": 5, "cellSize": 6.0, "markSize": 1.0 }
                    },
                    {
                        "opacity": 0.25,
                        "gradient": { "type": "pattern", "patternName": "Grid", "seed": 6, "cellSize": 8.0, "markSize": 1.0 }
                    }
                ],
                "appearanceStrokes": [{ "color": "#000000", "width": 2.0, "dash": [2, 2] }]
            }]
        }"##;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    let stack = &elements[0].appearance_stack.entries;
    assert_eq!(stack.len(), 3);
    let crate::scene::AppearanceEntry::Fill(fill) = &stack[0] else {
        panic!("Expected pattern fill");
    };
    assert!(matches!(fill.paint, crate::scene::PaintSource::Pattern(_)));
    assert!(matches!(
        &stack[1],
        crate::scene::AppearanceEntry::Fill(crate::scene::FillLayer {
            paint: crate::scene::PaintSource::Pattern(_),
            ..
        })
    ));
    let crate::scene::AppearanceEntry::Stroke(stroke) = &stack[2] else {
        panic!("Expected appearance stroke");
    };
    assert_eq!(stroke.dash.as_deref(), Some(&[2.0, 2.0][..]));
}

#[test]
fn test_rich_element_generates_scene_node() {
    let json = r##"{
        "artboard": { "name": "RichTest", "width": 100, "height": 100 },
        "elements": [{
            "id": "rich_path",
            "type": "path",
            "pathPoints": [
                {"anchor": [0, 0], "leftCtrl": [0, 0], "rightCtrl": [0, 0]},
                {"anchor": [10, 10], "leftCtrl": [10, 10], "rightCtrl": [10, 10]}
            ],
            "pathClosed": true,
            "appearanceStack": [
                { "type": "fill", "color": "#ff0000", "opacity": 1.0, "blendMode": "normal" }
            ]
        }]
    }"##;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    let token_map = std::collections::HashMap::new();
    let code = generate_artboard_file("RichTest", 100.0, 100.0, &elements, &token_map);

    // Should contain RichScene generation
    assert!(code.contains("RichScene: rich_path"));
    assert!(code.contains("egui_expressive::scene::SceneNode"));
    assert!(code.contains("egui_expressive::scene::Geometry::Path"));
    assert!(code.contains("egui_expressive::scene::path_points"));
    assert!(code.contains("egui_expressive::scene::AppearanceStack"));
    assert!(code.contains("egui_expressive::scene::render_node"));
}

#[test]
fn test_rich_clipped_group_preserves_clip_and_children() {
    let json = r##"{
        "artboard": { "name": "ClipTest", "width": 100, "height": 100 },
        "elements": [{
            "id": "clip_group",
            "type": "group",
            "clipChildren": true,
            "children": [{
                "id": "child_rect",
                "type": "shape",
                "x": 10, "y": 10, "w": 20, "h": 20,
                "fill": "#ff0000"
            }]
        }]
    }"##;
    let (_, elements) = parse_json_sidecar(json).unwrap();
    let token_map = std::collections::HashMap::new();
    let code = generate_artboard_file("ClipTest", 100.0, 100.0, &elements, &token_map);

    assert!(code.contains("clip_children: true"));
    assert!(code.contains("id: \"child_rect\""));
    assert!(code.contains("egui_expressive::scene::render_node"));
}
