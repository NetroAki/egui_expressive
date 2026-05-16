use egui_expressive::codegen::{generate_rust, LayoutNode};
use egui_expressive::scene::{Geometry, SceneNode};
use egui_expressive::{vectorize_rgba_to_scene_nodes, RasterVectorizeConfig};
use image::{Rgba, RgbaImage};

fn red_square_image() -> RgbaImage {
    let mut image = RgbaImage::from_pixel(16, 16, Rgba([0, 0, 0, 0]));
    for y in 4..12 {
        for x in 4..12 {
            image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }
    image
}

fn vector_group(id: &str, nodes: Vec<SceneNode>) -> SceneNode {
    let mut group = SceneNode::group(
        id,
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(64.0, 64.0)),
    );
    for node in nodes {
        group.push_child(node);
    }
    group
}

#[test]
fn raster_pixels_vectorize_to_scene_nodes_not_image_slots() {
    let image = red_square_image();
    let config = RasterVectorizeConfig {
        fit_rect: Some(egui::Rect::from_min_size(
            egui::pos2(10.0, 20.0),
            egui::vec2(80.0, 40.0),
        )),
        ..Default::default()
    };

    let nodes = vectorize_rgba_to_scene_nodes("linked_raster", &image, &config)
        .expect("raster tracing should produce vector nodes");
    assert!(!nodes.is_empty(), "expected traced vector paths");

    for node in &nodes {
        let bounds = node.geometry.bounds();
        assert!(
            bounds.min.x >= 9.9,
            "{} escaped fit rect: {bounds:?}",
            node.id
        );
        assert!(
            bounds.min.y >= 19.9,
            "{} escaped fit rect: {bounds:?}",
            node.id
        );
        assert!(
            bounds.max.x <= 90.1,
            "{} escaped fit rect: {bounds:?}",
            node.id
        );
        assert!(
            bounds.max.y <= 60.1,
            "{} escaped fit rect: {bounds:?}",
            node.id
        );
        assert!(
            matches!(node.geometry, Geometry::Path { .. }),
            "vectorized output must be path-like scene geometry"
        );
    }

    let code = generate_rust(
        "raster_vectorized",
        100.0,
        80.0,
        &[LayoutNode::RichScene(vector_group(
            "raster_vectorized",
            nodes,
        ))],
        None,
        None,
        None,
    );

    assert!(code.contains("scene::render_node"));
    assert!(code.contains("Geometry::Path"));
    assert!(code.contains("PaintSource::Solid"));
    assert!(!code.contains("paint_image_slot"));
    assert!(!code.contains("paint_image_from_path"));
    assert!(!code.contains("Image Slot"));
}

#[test]
fn transparent_raster_vectorizes_to_no_scene_nodes() {
    let image = RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0]));
    let nodes =
        vectorize_rgba_to_scene_nodes("transparent", &image, &RasterVectorizeConfig::default())
            .expect("transparent raster should be accepted");
    assert!(nodes.is_empty());
}
