use super::*;

#[test]
fn test_parse_naming_row() {
    assert!(matches!(parse_naming("row-login"), NamingHint::Row(_)));
    assert!(matches!(parse_naming("hstack-buttons"), NamingHint::Row(_)));
}

#[test]
fn test_parse_naming_column() {
    assert!(matches!(parse_naming("col-sidebar"), NamingHint::Column(_)));
    assert!(matches!(
        parse_naming("vstack-items"),
        NamingHint::Column(_)
    ));
}

#[test]
fn test_parse_naming_panel() {
    assert!(matches!(
        parse_naming("panel-left"),
        NamingHint::Panel(PanelSide::Left)
    ));
    assert!(matches!(
        parse_naming("panel-right"),
        NamingHint::Panel(PanelSide::Right)
    ));
    assert!(matches!(
        parse_naming("panel-top"),
        NamingHint::Panel(PanelSide::Top)
    ));
    assert!(matches!(
        parse_naming("panel-bottom"),
        NamingHint::Panel(PanelSide::Bottom)
    ));
}

#[test]
fn test_parse_naming_button() {
    assert!(matches!(parse_naming("btn-submit"), NamingHint::Button(_)));
    assert!(matches!(
        parse_naming("button-cancel"),
        NamingHint::Button(_)
    ));
}

#[test]
fn test_parse_naming_gap() {
    assert!(matches!(parse_naming("gap-16"), NamingHint::Gap(16.0)));
    assert!(matches!(parse_naming("gap-8"), NamingHint::Gap(8.0)));
}

#[test]
fn test_parse_naming_none() {
    assert!(matches!(parse_naming("Rectangle 1"), NamingHint::None));
    assert!(matches!(parse_naming("some random name"), NamingHint::None));
}

#[test]
fn test_generate_scroll_area_uses_id_salt() {
    let node = LayoutNode::ScrollArea {
        vertical: true,
        horizontal: false,
        children: vec![],
        id: "scroll-foo\"bar".to_string(),
    };

    let output = generate_node(&node, 0, None);

    assert!(output.contains("egui::ScrollArea::vertical().id_salt("));
    assert!(output.contains(r#"scroll-foo\"bar"#));
}

#[test]
fn test_cluster_into_rows() {
    let elements = vec![
        LayoutElement {
            id: "a".to_string(),
            el_type: ElementType::Shape,
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("a".to_string(), ElementType::Shape, 0.0, 0.0, 100.0, 50.0)
        },
        LayoutElement {
            id: "b".to_string(),
            el_type: ElementType::Shape,
            x: 110.0,
            y: 5.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("b".to_string(), ElementType::Shape, 110.0, 5.0, 100.0, 50.0)
        },
        LayoutElement {
            id: "c".to_string(),
            el_type: ElementType::Shape,
            x: 50.0,
            y: 100.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new(
                "c".to_string(),
                ElementType::Shape,
                50.0,
                100.0,
                100.0,
                50.0,
            )
        },
    ];

    let rows = cluster_into_rows(&elements, 0.5);
    // a and b should be in the same row (Y overlap), c in a different row
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].len(), 2);
    assert_eq!(rows[1].len(), 1);
}

#[test]
fn test_infer_horizontal_gap() {
    let elements = vec![
        LayoutElement {
            id: "a".to_string(),
            el_type: ElementType::Shape,
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("a".to_string(), ElementType::Shape, 0.0, 0.0, 100.0, 50.0)
        },
        LayoutElement {
            id: "b".to_string(),
            el_type: ElementType::Shape,
            x: 108.0,
            y: 0.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("b".to_string(), ElementType::Shape, 108.0, 0.0, 100.0, 50.0)
        },
        LayoutElement {
            id: "c".to_string(),
            el_type: ElementType::Shape,
            x: 216.0,
            y: 0.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("c".to_string(), ElementType::Shape, 216.0, 0.0, 100.0, 50.0)
        },
    ];

    let gap = infer_horizontal_gap(&elements);
    assert!((gap - 8.0).abs() < 0.1);
}

#[test]
fn test_infer_vertical_gap() {
    let elements = vec![
        LayoutElement {
            id: "a".to_string(),
            el_type: ElementType::Shape,
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("a".to_string(), ElementType::Shape, 0.0, 0.0, 100.0, 50.0)
        },
        LayoutElement {
            id: "b".to_string(),
            el_type: ElementType::Shape,
            x: 0.0,
            y: 58.0,
            w: 100.0,
            h: 50.0,
            fill: None,
            stroke: None,
            text: None,
            text_size: None,
            children: vec![],
            opacity: 1.0,
            rotation_deg: 0.0,
            corner_radius: 0.0,
            gradient: None,
            blend_mode: BlendMode::Normal,
            effects: vec![],
            stroke_dash: None,
            clip_children: false,
            text_align: None,
            letter_spacing: None,
            line_height: None,
            ..LayoutElement::new("b".to_string(), ElementType::Shape, 0.0, 58.0, 100.0, 50.0)
        },
    ];

    let gap = infer_vertical_gap(&elements);
    assert!((gap - 8.0).abs() < 0.1);
}

#[test]
fn test_parse_svg_elements() {
    let svg = r##"<svg>
            <rect id="bg-rect" x="0" y="0" width="375" height="812" fill="#121212"/>
            <g id="row-buttons">
                <rect id="btn-login" x="20" y="400" width="100" height="40"/>
                <rect id="btn-cancel" x="130" y="400" width="100" height="40"/>
            </g>
            <text id="label-welcome" x="20" y="50">Welcome</text>
        </svg>"##;

    let elements = parse_svg_elements(svg);
    assert!(!elements.is_empty());
}

#[test]
fn test_parse_json_sidecar() {
    let json = r##"{
            "artboard": {
                "name": "Login",
                "width": 375,
                "height": 812
            },
            "elements": [
                {"id": "bg", "type": "rect", "x": 0, "y": 0, "w": 375, "h": 812, "fill": "#121212"},
                {"id": "btn-login", "type": "rect", "x": 20, "y": 400, "w": 100, "h": 40},
                {"id": "label-welcome", "type": "text", "x": 20, "y": 50, "text": "Welcome"}
            ]
        }"##;

    let result = parse_json_sidecar(json);
    assert!(result.is_ok());

    let (artboard, elements) = result.unwrap();
    assert_eq!(artboard.name, "Login");
    assert_eq!(artboard.width, 375.0);
    assert_eq!(artboard.height, 812.0);
    assert_eq!(elements.len(), 3);
}

#[test]
fn test_svg_to_rust_scaffold() {
    let svg = r##"<svg>
            <rect id="bg" x="0" y="0" width="375" height="812" fill="#121212"/>
            <text id="label-welcome" x="20" y="50">Welcome</text>
            <rect id="btn-submit" x="20" y="100" width="100" height="40"/>
        </svg>"##;

    let options = InferenceOptions::default();
    let code = svg_to_rust_scaffold(svg, "login", &options);

    assert!(code.contains("pub fn draw_login"));
    assert!(code.contains("egui::Color32::from_rgb"));
    assert!(code.contains("ui.label"));
    assert!(code.contains("ui.button"));
}

#[test]
fn test_generate_rust_with_background() {
    let nodes = vec![
        LayoutNode::Label {
            text: "Hello".to_string(),
            size: 24.0,
            color: Some(Color32::WHITE),
            font_family: None,
            id: "greeting".to_string(),
        },
        LayoutNode::Button {
            label: "Click Me".to_string(),
            id: "main-btn".to_string(),
        },
    ];

    let code = generate_rust(
        "test",
        375.0,
        812.0,
        &nodes,
        Some(Color32::from_rgb(18, 18, 18)),
        None,
        None,
    );

    assert!(code.contains("pub fn draw_test"));
    assert!(code.contains("Background"));
    assert!(code.contains("egui::Color32::from_rgb(18, 18, 18)"));
}

#[test]
fn test_generate_rust_with_state() {
    let nodes = vec![LayoutNode::TextEdit {
        placeholder: "Enter email".to_string(),
        id: "email-input".to_string(),
    }];

    let code = generate_rust(
        "login",
        375.0,
        812.0,
        &nodes,
        None,
        Some("LoginState"),
        None,
    );

    assert!(code.contains("pub fn draw_login(ui: &mut Ui, state: &mut LoginState)"));
    assert!(code.contains("state.email_input"));
}

#[test]
fn test_infer_layout_with_naming_conventions() {
    let elements = vec![LayoutElement {
        id: "row-buttons".to_string(),
        el_type: ElementType::Group,
        x: 0.0,
        y: 0.0,
        w: 300.0,
        h: 50.0,
        fill: None,
        stroke: None,
        text: None,
        text_size: None,
        children: vec![
            LayoutElement {
                id: "btn-a".to_string(),
                el_type: ElementType::Shape,
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 40.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
                ..LayoutElement::new(
                    "btn-a".to_string(),
                    ElementType::Shape,
                    0.0,
                    0.0,
                    100.0,
                    40.0,
                )
            },
            LayoutElement {
                id: "btn-b".to_string(),
                el_type: ElementType::Shape,
                x: 110.0,
                y: 0.0,
                w: 100.0,
                h: 40.0,
                fill: None,
                stroke: None,
                text: None,
                text_size: None,
                children: vec![],
                opacity: 1.0,
                rotation_deg: 0.0,
                corner_radius: 0.0,
                gradient: None,
                blend_mode: BlendMode::Normal,
                effects: vec![],
                stroke_dash: None,
                clip_children: false,
                text_align: None,
                letter_spacing: None,
                line_height: None,
                ..LayoutElement::new(
                    "btn-b".to_string(),
                    ElementType::Shape,
                    110.0,
                    0.0,
                    100.0,
                    40.0,
                )
            },
        ],
        opacity: 1.0,
        rotation_deg: 0.0,
        corner_radius: 0.0,
        gradient: None,
        blend_mode: BlendMode::Normal,
        effects: vec![],
        stroke_dash: None,
        clip_children: false,
        text_align: None,
        letter_spacing: None,
        line_height: None,
        ..LayoutElement::new(
            "row-buttons".to_string(),
            ElementType::Group,
            0.0,
            0.0,
            300.0,
            50.0,
        )
    }];

    let options = InferenceOptions::default();
    let nodes = infer_layout(&elements, &options);

    assert!(!nodes.is_empty());
    // The row should be inferred from the naming convention
    if let LayoutNode::Row { id, .. } = &nodes[0] {
        assert_eq!(id, "buttons");
    }
}

fn r100_004a_effect(effect_type: EffectType) -> EffectDef {
    EffectDef {
        effect_type,
        x: 3.0,
        y: 4.0,
        blur: 6.0,
        spread: 2.0,
        color: Color32::from_rgba_unmultiplied(12, 34, 56, 180),
        blend_mode: BlendMode::Normal,
        depth: 5.0,
        angle: 135.0,
        highlight: Some(Color32::from_rgba_unmultiplied(240, 230, 220, 128)),
        shadow_color: Some(Color32::from_rgba_unmultiplied(10, 20, 30, 160)),
        radius: 7.0,
        amount: 0.35,
        scale: 4.0,
        seed: 42,
    }
}

fn r100_004a_shape_with_effects(effects: Vec<EffectDef>) -> LayoutNode {
    LayoutNode::Shape {
        x: 8.0,
        y: 12.0,
        w: 48.0,
        h: 24.0,
        fill: Color32::from_rgb(40, 60, 80),
        id: "effect-shape".to_string(),
        style: VisualStyle {
            effects,
            ..Default::default()
        },
    }
}

#[test]
fn r100_004a_direct_shape_effects_emit_bounded_diagnostics() {
    let node = r100_004a_shape_with_effects(vec![
        r100_004a_effect(EffectType::DropShadow),
        r100_004a_effect(EffectType::OuterGlow),
        r100_004a_effect(EffectType::InnerShadow),
        r100_004a_effect(EffectType::GaussianBlur),
        r100_004a_effect(EffectType::Feather),
        r100_004a_effect(EffectType::Noise),
    ]);

    let code = generate_node(&node, 0, None);

    for label in [
        "DropShadow",
        "OuterGlow",
        "InnerShadow",
        "GaussianBlur",
        "Feather",
        "Noise",
    ] {
        assert!(
            code.contains(&format!("R100-004A bounded codegen: {label}")),
            "missing bounded diagnostic for {label}:\n{code}"
        );
    }

    for helper in [
        "egui_expressive::box_shadow",
        "egui_expressive::soft_shadow",
        "egui_expressive::inner_shadow",
        "egui_expressive::noise_rect",
        "egui_expressive::blur::soft_shadow",
    ] {
        assert!(code.contains(helper), "missing helper {helper}:\n{code}");
    }
    assert!(!code.contains("GpuSourceLayerEffectCallback"));
}

#[test]
fn r100_004a_direct_shape_effects_emit_unsupported_diagnostics() {
    let node = r100_004a_shape_with_effects(vec![
        r100_004a_effect(EffectType::InnerGlow),
        r100_004a_effect(EffectType::Bevel),
        r100_004a_effect(EffectType::LiveEffect),
        r100_004a_effect(EffectType::Unknown("svg-filter".to_string())),
    ]);

    let code = generate_node(&node, 0, None);

    assert!(code.contains("R100-004A unsupported codegen: InnerGlow"));
    assert!(code.contains("R100-004A unsupported codegen: Bevel"));
    assert!(code.contains("R100-004A unsupported codegen: LiveEffect"));
    assert!(code.contains("R100-004A unsupported codegen: unrecognized effect \"svg-filter\""));
    assert!(!code.contains("// live_effect"));
    assert!(!code.contains("// unknown effect:"));
    assert!(!code.contains("GpuSourceLayerEffectCallback"));
}

#[test]
fn r100_004a_scene_codegen_remains_effect_data_only() {
    let mut node = crate::scene::SceneNode::rect(
        "scene-effect",
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(32.0, 24.0)),
        0.0,
    );
    node.appearance
        .entries
        .push(crate::scene::EffectLayer::new(r100_004a_effect(EffectType::DropShadow)).into());

    let code = generate_scene_node_code(&node, 0);

    assert!(code.contains("AppearanceEntry::Effect"));
    assert!(code.contains("EffectLayer"));
    assert!(code.contains("EffectType::DropShadow"));
    assert!(!code.contains("R100-004A bounded codegen"));
    assert!(!code.contains("GpuSourceLayerEffectCallback"));
    assert!(!code.contains("box_shadow"));
    assert!(!code.contains("soft_shadow"));
}
