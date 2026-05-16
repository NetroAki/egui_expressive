use super::super::*;

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
    // The row should be inferred from the generic SVG layout hint
    if let LayoutNode::Row { id, .. } = &nodes[0] {
        assert_eq!(id, "buttons");
    }
}

#[test]
fn test_generate_tokens_file() {
    let mut color_map = HashMap::new();
    color_map.insert("primary".to_string(), Color32::from_rgb(103, 80, 164));
    color_map.insert("surface".to_string(), Color32::from_rgb(28, 27, 31));

    let tokens = generate_tokens_file(&color_map, &[8.0, 16.0, 24.0]);
    assert!(tokens.contains("pub const PRIMARY: Color32"));
    assert!(tokens.contains("pub const SURFACE: Color32"));
    assert!(tokens.contains("pub const SPACING_SM: f32"));
}

#[test]
fn test_generate_state_file() {
    let artboards = vec![ArtboardState {
        name: "login_screen".to_string(),
        text_fields: vec!["email".to_string(), "password".to_string()],
        button_labels: vec!["Sign In".to_string(), "Forgot Password".to_string()],
    }];

    let state = generate_state_file(&artboards);
    assert!(state.contains("#[derive(Default, Clone)]"));
    assert!(state.contains("pub struct LoginScreenState"));
    assert!(state.contains("pub email: String"));
    assert!(state.contains("pub password: String"));
    assert!(state.contains("pub enum LoginScreenAction"));
    assert!(state.contains("SignIn"));
    assert!(state.contains("ForgotPassword"));
}

#[test]
fn test_generate_mod_file() {
    let artboard_names = vec!["login_screen", "dashboard"];
    let mod_file = generate_mod_file(&artboard_names);
    assert!(mod_file.contains("pub mod tokens;"));
    assert!(mod_file.contains("pub mod state;"));
    assert!(mod_file.contains("pub mod components;"));
    assert!(mod_file.contains("pub mod login_screen;"));
    assert!(mod_file.contains("pub mod dashboard;"));
}

#[test]
fn test_generate_components_file() {
    let components = vec![ComponentDef {
        name: "primary_button".to_string(),
        fill: Color32::from_rgb(103, 80, 164),
        rounding: 8.0,
        text_size: 14.0,
        text_color: Color32::WHITE,
    }];

    let components_file = generate_components_file(&components);
    assert!(components_file.contains("Reusable design primitives live in egui_expressive"));
    assert!(!components_file.contains("pub fn primary_button"));
}

#[test]
fn test_generate_artboard_file_produces_valid_rust() {
    let elements = vec![LayoutElement::new(
        "btn".to_string(),
        ElementType::Shape,
        10.0,
        20.0,
        80.0,
        40.0,
    )];
    let token_map = HashMap::new();
    let code = generate_artboard_file("My Artboard", 375.0, 812.0, &elements, &token_map);
    // Must contain a pub fn with a valid Rust identifier
    assert!(
        code.contains("pub fn draw_"),
        "missing pub fn draw_: {}",
        &code[..200.min(code.len())]
    );
    // Must contain egui imports
    assert!(code.contains("use egui"));
    // Must not contain nested pub fn (double-wrapping bug)
    let fn_count = code.matches("pub fn draw_").count();
    assert_eq!(
        fn_count, 1,
        "expected exactly 1 pub fn draw_, found {}",
        fn_count
    );
    assert!(
        code.contains("ui.allocate_space(egui::vec2(375.0, 812.0));"),
        "missing artboard size allocation: {}",
        &code[..300.min(code.len())]
    );
}

#[test]
fn test_rotated_radial_gradient_uses_rotated_mesh_points_without_post_rotation() {
    let node = LayoutNode::Shape {
        x: 10.0,
        y: 20.0,
        w: 80.0,
        h: 40.0,
        fill: Color32::WHITE,
        id: "radial".to_string(),
        style: VisualStyle {
            rotation_deg: 30.0,
            gradient: Some(GradientDef {
                gradient_type: GradientType::Radial,
                angle_deg: 0.0,
                center: Some([40.0, 40.0]),
                focal_point: Some([45.0, 40.0]),
                radius: Some(30.0),
                transform: None,
                stops: vec![
                    GradientStop {
                        position: 0.0,
                        color: Color32::WHITE,
                    },
                    GradientStop {
                        position: 1.0,
                        color: Color32::BLACK,
                    },
                ],
            }),
            ..VisualStyle::default()
        },
    };
    let code = generate_node(&node, 0, None);
    assert!(code.contains("let gradient_rect_pts = vec![_rot.apply"));
    assert!(!code.contains("grad_shape = _rot.apply_to_shape(grad_shape);"));
}

#[test]
fn test_rounded_linear_gradient_uses_path_mesh_clip() {
    let node = LayoutNode::Shape {
        x: 0.0,
        y: 0.0,
        w: 80.0,
        h: 40.0,
        fill: Color32::WHITE,
        id: "rounded_linear".to_string(),
        style: VisualStyle {
            corner_radius: 8.0,
            gradient: Some(GradientDef {
                gradient_type: GradientType::Linear,
                angle_deg: 45.0,
                center: None,
                focal_point: None,
                radius: None,
                transform: None,
                stops: vec![
                    GradientStop {
                        position: 0.0,
                        color: Color32::WHITE,
                    },
                    GradientStop {
                        position: 1.0,
                        color: Color32::BLACK,
                    },
                ],
            }),
            ..VisualStyle::default()
        },
    };
    let code = generate_node(&node, 0, None);
    assert!(code.contains("rounded_rect_path(rect, 8.0)"));
    assert!(code.contains("gradient_path_mesh_with_transform"));
    assert!(!code.contains("linear_gradient_rect"));
}

#[test]
fn test_circle_inference_uses_rich_scene_geometry() {
    let elem = LayoutElement::new(
        "circle".to_string(),
        ElementType::Circle,
        10.0,
        20.0,
        30.0,
        30.0,
    );
    let node = infer_element(&elem, &InferenceOptions::default());
    let LayoutNode::RichScene(scene_node) = node else {
        panic!("circle should infer as rich scene");
    };
    assert!(matches!(
        scene_node.geometry,
        crate::scene::Geometry::Ellipse { .. }
    ));
}

#[test]
fn test_rotated_linear_gradient_and_stroke_share_rotated_path() {
    let node = LayoutNode::Shape {
        x: 0.0,
        y: 0.0,
        w: 80.0,
        h: 40.0,
        fill: Color32::WHITE,
        id: "rotated_linear".to_string(),
        style: VisualStyle {
            rotation_deg: 20.0,
            stroke: Some((2.0, Color32::BLACK)),
            stroke_dash: Some(vec![2.0, 3.0]),
            stroke_cap: Some(StrokeCap::Round),
            stroke_join: Some(StrokeJoin::Bevel),
            gradient: Some(GradientDef {
                gradient_type: GradientType::Linear,
                angle_deg: 45.0,
                center: None,
                focal_point: None,
                radius: None,
                transform: None,
                stops: vec![
                    GradientStop {
                        position: 0.0,
                        color: Color32::WHITE,
                    },
                    GradientStop {
                        position: 1.0,
                        color: Color32::BLACK,
                    },
                ],
            }),
            ..VisualStyle::default()
        },
    };
    let code = generate_node(&node, 0, None);
    assert!(code.contains("gradient_path_mesh_with_transform"));
    assert!(code.contains("_rot.apply(rect.left_top())"));
    assert!(code.contains("egui_expressive::dashed_path"));
    assert!(code.contains("egui_expressive::StrokeCap::Round"));
    assert!(code.contains("egui_expressive::StrokeJoin::Bevel"));
    assert!(!code.contains("grad_shape = _rot.apply_to_shape(grad_shape);"));
}

#[test]
fn test_generate_all_artboards_partitions_elements() {
    let mut e1 = LayoutElement::new("e1".to_string(), ElementType::Shape, 0.0, 0.0, 50.0, 50.0);
    e1.artboard_name = Some("Home".to_string());
    let mut e2 = LayoutElement::new("e2".to_string(), ElementType::Shape, 0.0, 0.0, 50.0, 50.0);
    e2.artboard_name = Some("Settings".to_string());
    let elements = vec![e1, e2];
    let token_map = HashMap::new();
    let artboards = [("Home", 375.0f32, 812.0f32), ("Settings", 375.0, 812.0)];
    let files = generate_all_artboards(&elements, &artboards, &token_map);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].0, "home.rs");
    assert_eq!(files[1].0, "settings.rs");
    // Home file should contain e1's id, Settings file should contain e2's id
    // (both may appear since unassigned elements are included in all artboards)
    assert!(files[0].1.contains("pub fn draw_home"));
    assert!(files[1].1.contains("pub fn draw_settings"));
}
