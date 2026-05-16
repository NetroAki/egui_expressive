use super::*;

#[test]
fn test_parse_live_effect_xml() {
    let xml = r#"<LiveEffect name="Adobe Drop Shadow"><Dict data="R horz 7.0 R vert 7.0 I blnd 1 B enbl 1"/></LiveEffect>"#;
    let effects = parse_live_effect_xml(xml);
    assert!(!effects.is_empty());
    assert_eq!(effects[0].name, "Adobe Drop Shadow");
    assert_eq!(
        effects[0].params.params.get("horz"),
        Some(&Value::from(7.0))
    );
}

#[test]
fn test_parse_dict_data() {
    let data = "R horz 7.0 R vert 7.0 R blur 5.0 I opac 75 B enbl 1";
    let params = parse_dict_data(data);
    assert_eq!(params.get("horz"), Some(&Value::from(7.0)));
    assert_eq!(params.get("vert"), Some(&Value::from(7.0)));
    assert_eq!(params.get("blur"), Some(&Value::from(5.0)));
    assert_eq!(params.get("opac"), Some(&Value::from(75_i64)));
    assert_eq!(params.get("enbl"), Some(&Value::Bool(true)));
}

#[test]
fn test_parse_envelope_mesh() {
    let content = "%AI9_EnvelopeMesh 3 3\n[0 0] [50 0] [100 0]\n[0 50] [50 50] [100 50]\n[0 100] [50 100] [100 100]";
    let mesh = parse_envelope_mesh(content);
    assert!(mesh.is_some());
    let mesh = mesh.unwrap();
    assert_eq!(mesh.rows, 3);
    assert_eq!(mesh.cols, 3);
    assert!(mesh.points.len() >= 9);
}

#[test]
fn test_parse_3d_extrude() {
    let content = "%AI9_3D_Extrude 100 45 30 0";
    let effect = parse_3d_effect(content);
    assert!(effect.is_some());
    let effect = effect.unwrap();
    assert_eq!(effect.effect_type, "extrude");
    assert_eq!(effect.depth, 100.0);
}

#[test]
fn test_extract_layer_name() {
    let content = "%%Layer: MyLayer\n%AI8_BeginLayer";
    let name = extract_layer_name(content);
    assert_eq!(name, Some("MyLayer".to_string()));
}

#[test]
fn test_layer_name_not_artboard() {
    let content = "%%Layer: MyLayer\n%AI8_BeginLayer\n[ 1.0 0.0 0.0 1.0 ] Xa";
    let mut errors = vec![];
    let elem = parse_aip_private_stream(content.as_bytes(), 0, &mut errors).unwrap();
    assert_eq!(elem.name, Some("MyLayer".to_string()));
    assert_eq!(elem.artboard_name, None);
}

#[test]
fn test_element_to_layout_copies_path() {
    let mut elem = Element::default();
    elem.path_points.push(PathPoint {
        anchor: [1.0, 2.0],
        left_ctrl: [3.0, 4.0],
        right_ctrl: [5.0, 6.0],
    });
    elem.path_closed = true;
    let layout = element_to_layout(&elem, 0);
    assert_eq!(layout.path_points.len(), 1);
    assert_eq!(layout.path_points[0].anchor, [1.0, 2.0]);
    assert!(layout.path_closed);
}

#[test]
fn test_parse_appearance_stroke_properties() {
    let content = "1 J\n2 j\n4.0 M\n[2.0 4.0] 0 d\n2.5 w\n[ 0.0 1.0 0.0 1.0 ] xa";
    let (_, strokes) = parse_appearance(content);
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].g, 255);
    assert_eq!(strokes[0].width, 2.5);
    assert_eq!(strokes[0].cap.as_deref(), Some("round"));
    assert_eq!(strokes[0].join.as_deref(), Some("bevel"));
    assert_eq!(strokes[0].miter_limit, Some(4.0));
    assert_eq!(strokes[0].dash.as_deref(), Some(&[2.0, 4.0][..]));
}

#[test]
fn test_parse_appearance_case_sensitive() {
    let content = "[ 1.0 0.0 0.0 1.0 ] Xa\n[ 0.0 1.0 0.0 1.0 ] xa";
    let (fills, strokes) = parse_appearance(content);
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].r, 255);
    assert_eq!(fills[0].opacity, Some(1.0));
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].g, 255);
    assert_eq!(strokes[0].opacity, Some(1.0));
}

#[test]
fn test_parse_appearance_blend_mode() {
    let content = "/BM /Multiply\n[ 1.0 0.0 0.0 1.0 ] Xa";
    let (fills, _) = parse_appearance(content);
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].blend_mode, "Multiply");

    let content2 = "/BlendMode /Screen\n[ 0.0 1.0 0.0 1.0 ] xa";
    let (_, strokes) = parse_appearance(content2);
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].blend_mode, "Screen");
}

#[test]
fn test_parse_appearance_gradient_fallback() {
    let content = "0.0 0.0 0.0 1.0 k\n/Pattern cs\n sh\n";
    let (fills, _strokes) = parse_appearance(content);
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].r, 128); // Fallback color
}

#[test]
fn test_parse_appearance_stroke_pattern_surfaces_gradient_metadata() {
    let content = "2 w\n/Pattern CS\n";
    let (_fills, strokes) = parse_appearance(content);
    assert_eq!(strokes.len(), 1);
    assert!(strokes[0].gradient.is_some());
    let layout = element_to_layout(
        &Element {
            id: "stroke_pattern".to_string(),
            appearance_strokes: strokes,
            ..Default::default()
        },
        0,
    );
    let egui_expressive::scene::AppearanceEntry::Stroke(stroke) =
        &layout.appearance_stack.entries[0]
    else {
        panic!("expected stroke");
    };
    assert!(matches!(
        stroke.paint,
        egui_expressive::scene::PaintSource::Pattern(_)
    ));
}

#[test]
fn test_extract_ai_version() {
    let content = "%AI8_CreatorVersion 25.0";
    let version = extract_ai_version(content);
    assert_eq!(version, "25.0");
}

#[test]
fn test_parse_ctm_identity() {
    let content = "1 0 0 1 0 0 cm";
    let result = parse_ctms_from_stream(content);
    assert!(!result.is_empty());
    let (rot, sx, sy, tx, ty) = result.last().unwrap();
    assert!(
        (rot).abs() < 0.001,
        "identity rotation should be 0, got {}",
        rot
    );
    assert!((sx - 1.0).abs() < 0.001);
    assert!((sy - 1.0).abs() < 0.001);
    assert!((tx).abs() < 0.001);
    assert!((ty).abs() < 0.001);
}

#[test]
fn test_parse_pdf_painted_paths_extracts_code_drawn_fill() {
    let content = "q 1 0 0 1 10 20 cm 0.1 0.2 0.3 rg 0 0 30 40 re f Q";
    let elements = parse_pdf_painted_path_elements(content, 7);
    assert_eq!(elements.len(), 1);
    let element = &elements[0];
    assert_eq!(element.path_points.len(), 4);
    assert!(element.path_closed);
    assert_eq!(element.appearance_fills.len(), 1);
    assert_eq!(element.appearance_fills[0].r, 26);
    assert_eq!(element.appearance_fills[0].g, 51);
    assert_eq!(element.appearance_fills[0].b, 77);
    assert_eq!(element.bounds.unwrap(), [10.0, 20.0, 30.0, 40.0]);
}

#[test]
fn test_parse_pdf_painted_paths_extracts_stroke_style() {
    let content = "2 J 1 j 4 M 3 w 0 1 0 RG 10 10 m 50 10 l S";
    let elements = parse_pdf_painted_path_elements(content, 8);
    assert_eq!(elements.len(), 1);
    let strokes = &elements[0].appearance_strokes;
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].g, 255);
    assert_eq!(strokes[0].width, 3.0);
    assert_eq!(strokes[0].cap.as_deref(), Some("square"));
    assert_eq!(strokes[0].join.as_deref(), Some("round"));
    assert_eq!(strokes[0].miter_limit, Some(4.0));
}

#[test]
fn test_parse_ctm_90deg() {
    // 90 degree rotation: a=0, b=1, c=-1, d=0
    let content = "0 1 -1 0 0 0 cm";
    let result = parse_ctms_from_stream(content);
    assert!(!result.is_empty());
    let (rot, _sx, _sy, _tx, _ty) = result.last().unwrap();
    assert!((rot - 90.0).abs() < 0.01, "expected 90 deg, got {}", rot);
}

#[test]
fn test_detect_corner_radius_zero() {
    // A simple square has no control handles → radius 0
    let points = vec![
        PathPoint {
            anchor: [0.0, 0.0],
            left_ctrl: [0.0, 0.0],
            right_ctrl: [0.0, 0.0],
        },
        PathPoint {
            anchor: [100.0, 0.0],
            left_ctrl: [100.0, 0.0],
            right_ctrl: [100.0, 0.0],
        },
        PathPoint {
            anchor: [100.0, 100.0],
            left_ctrl: [100.0, 100.0],
            right_ctrl: [100.0, 100.0],
        },
        PathPoint {
            anchor: [0.0, 100.0],
            left_ctrl: [0.0, 100.0],
            right_ctrl: [0.0, 100.0],
        },
    ];
    assert_eq!(detect_corner_radius(&points), 0.0);
}

#[test]
fn test_detect_corner_radius_rounded() {
    // 8-point rounded rect with radius=50: handle distance = 50 * 0.5522847498 ≈ 27.614
    const KAPPA: f64 = 0.5522847498;
    let r = 50.0f64;
    let h = r * KAPPA;
    // Top edge: TL-right, TR-left
    let points = vec![
        PathPoint {
            anchor: [r, 0.0],
            left_ctrl: [r - h, 0.0],
            right_ctrl: [r + h, 0.0],
        }, // top-left corner right
        PathPoint {
            anchor: [100.0 - r, 0.0],
            left_ctrl: [100.0 - r - h, 0.0],
            right_ctrl: [100.0 - r + h, 0.0],
        }, // top-right corner left
        PathPoint {
            anchor: [100.0, r],
            left_ctrl: [100.0, r - h],
            right_ctrl: [100.0, r + h],
        }, // right-top corner
        PathPoint {
            anchor: [100.0, 100.0 - r],
            left_ctrl: [100.0, 100.0 - r - h],
            right_ctrl: [100.0, 100.0 - r + h],
        },
        PathPoint {
            anchor: [100.0 - r, 100.0],
            left_ctrl: [100.0 - r + h, 100.0],
            right_ctrl: [100.0 - r - h, 100.0],
        },
        PathPoint {
            anchor: [r, 100.0],
            left_ctrl: [r + h, 100.0],
            right_ctrl: [r - h, 100.0],
        },
        PathPoint {
            anchor: [0.0, 100.0 - r],
            left_ctrl: [0.0, 100.0 - r + h],
            right_ctrl: [0.0, 100.0 - r - h],
        },
        PathPoint {
            anchor: [0.0, r],
            left_ctrl: [0.0, r + h],
            right_ctrl: [0.0, r - h],
        },
    ];
    let detected = detect_corner_radius(&points);
    assert!(
        (detected - r).abs() < 2.0,
        "expected radius ~{}, got {}",
        r,
        detected
    );
}

#[test]
fn test_parse_path_geometry() {
    let content = "10 20 m 30 40 l 50 60 70 80 90 100 c h";
    let (points, closed) = parse_path_geometry(content);
    assert!(closed);
    assert_eq!(points.len(), 3);

    // m 10 20
    assert_eq!(points[0].anchor, [10.0, 20.0]);

    // l 30 40
    assert_eq!(points[1].anchor, [30.0, 40.0]);

    // c 50 60 70 80 90 100
    assert_eq!(points[2].anchor, [90.0, 100.0]);
    assert_eq!(points[2].left_ctrl, [70.0, 80.0]);
    assert_eq!(points[1].right_ctrl, [50.0, 60.0]);
}

#[test]
fn test_generate_per_artboard_output() {
    let result = AiParseResult {
        version: "1.0".to_string(),
        source_file: "test.ai".to_string(),
        ai_version: "25.0".to_string(),
        artboards: vec![Artboard {
            name: "Artboard_1".to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }],
        page_tiles: vec![],
        elements: vec![Element {
            id: "elem_1".to_string(),
            artboard_name: Some("Artboard_1".to_string()),
            ..Default::default()
        }],
        transform_candidates: vec![],
        errors: vec![],
    };

    let output = generate_per_artboard_output(&result);
    assert_eq!(output.len(), 1);
    let obj = output[0].as_object().unwrap();
    assert_eq!(obj.get("artboard").unwrap().as_str().unwrap(), "Artboard_1");
    assert_eq!(obj.get("element_count").unwrap().as_u64().unwrap(), 1);
    assert!(obj.get("elements").unwrap().as_array().unwrap().len() == 1);
}

#[test]
fn test_parse_ai_file_real_sample() {
    let path = Path::new("UI assets from illustrator.ai");
    if path.exists() {
        let result = parse_ai_file(path).unwrap();
        assert!(!result.elements.is_empty(), "Should find elements");
        assert!(!result.artboards.is_empty(), "Should find artboards");
        assert!(
            result
                .elements
                .iter()
                .any(|el| !el.appearance_fills.is_empty() || !el.appearance_strokes.is_empty()),
            "real Illustrator fixture should yield code-drawn vector appearances"
        );
        let per_artboard = generate_per_artboard_output(&result);
        assert!(
            !per_artboard.is_empty(),
            "Should generate per-artboard output"
        );

        let reference_png = Path::new("UI assets from illustrator.png");
        if reference_png.exists() {
            let reference = image::open(reference_png).unwrap().to_rgba8();
            assert_eq!([reference.width(), reference.height()], [5102, 3679]);
            let max_artboard_width = result
                .artboards
                .iter()
                .map(|artboard| artboard.width)
                .fold(0.0, f64::max);
            let max_artboard_height = result
                .artboards
                .iter()
                .map(|artboard| artboard.height)
                .fold(0.0, f64::max);
            assert!(reference.width() as f64 >= max_artboard_width);
            assert!(reference.height() as f64 >= max_artboard_height);
        }
    }
}
