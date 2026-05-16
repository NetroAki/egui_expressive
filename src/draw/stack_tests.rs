#[cfg(test)]
use super::*;

/// Alignment for stacked/layered content within a bounding rect.
pub enum StackAlign {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl StackAlign {
    /// Convert to egui's `Align2`.
    pub fn to_align2(self) -> egui::Align2 {
        match self {
            Self::TopLeft => egui::Align2::LEFT_TOP,
            Self::TopCenter => egui::Align2::CENTER_TOP,
            Self::TopRight => egui::Align2::RIGHT_TOP,
            Self::CenterLeft => egui::Align2::LEFT_CENTER,
            Self::Center => egui::Align2::CENTER_CENTER,
            Self::CenterRight => egui::Align2::RIGHT_CENTER,
            Self::BottomLeft => egui::Align2::LEFT_BOTTOM,
            Self::BottomCenter => egui::Align2::CENTER_BOTTOM,
            Self::BottomRight => egui::Align2::RIGHT_BOTTOM,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::BlendMode;

    fn opaque(r: u8, g: u8, b: u8) -> egui::Color32 {
        egui::Color32::from_rgb(r, g, b)
    }

    #[test]
    fn test_mesh_gradient_patch_generates_subdivided_mesh() {
        let shape = mesh_gradient_patch(
            [
                egui::pos2(0.0, 0.0),
                egui::pos2(10.0, 0.0),
                egui::pos2(10.0, 10.0),
                egui::pos2(0.0, 10.0),
            ],
            [
                egui::Color32::RED,
                egui::Color32::GREEN,
                egui::Color32::BLUE,
                egui::Color32::WHITE,
            ],
            2,
        );

        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert_eq!(mesh.vertices.len(), 9);
        assert_eq!(mesh.indices.len(), 24);
        assert_eq!(mesh.vertices[0].pos, egui::pos2(0.0, 0.0));
        assert_eq!(mesh.vertices[8].pos, egui::pos2(10.0, 10.0));
    }

    #[test]
    fn test_noise_rect_is_deterministic_and_subdivided() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(4.0, 4.0));
        let a = noise_rect(rect, 42, 2.0, 0.5);
        let b = noise_rect(rect, 42, 2.0, 0.5);
        assert_eq!(a.len(), 4);
        assert_eq!(format!("{:?}", a), format!("{:?}", b));
    }

    #[test]
    fn test_radial_gradient_rect_stops_preserves_multiple_rings() {
        let shape = radial_gradient_rect_stops(
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(10.0, 10.0)),
            &[
                (0.0, egui::Color32::RED),
                (0.5, egui::Color32::GREEN),
                (1.0, egui::Color32::BLUE),
            ],
            8,
        );
        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert!(mesh.vertices.len() > 10);
        assert!(mesh.indices.len() > 24);
    }

    #[test]
    fn test_radial_gradient_path_mesh_has_inner_stop_vertex() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(10.0, 0.0),
            egui::pos2(10.0, 10.0),
            egui::pos2(0.0, 10.0),
        ];
        let shape = gradient_path_mesh(
            &points,
            &[(0.0, egui::Color32::RED), (1.0, egui::Color32::BLUE)],
            0.0,
            true,
        )
        .expect("radial path mesh");
        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert!(mesh.vertices.len() > points.len());
        assert!(mesh
            .vertices
            .iter()
            .any(|v| v.pos == egui::pos2(5.0, 5.0) && v.color == egui::Color32::RED));
    }

    #[test]
    fn test_radial_gradient_path_mesh_uses_explicit_focal_point_and_radius() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(10.0, 0.0),
            egui::pos2(10.0, 10.0),
            egui::pos2(0.0, 10.0),
        ];
        let focal = egui::pos2(3.0, 4.0);
        let shape = gradient_path_mesh_with_geometry(
            &points,
            &[(0.0, egui::Color32::RED), (1.0, egui::Color32::BLUE)],
            0.0,
            true,
            Some(egui::pos2(2.0, 2.0)),
            Some(focal),
            Some(20.0),
        )
        .expect("radial path mesh");
        let egui::Shape::Mesh(mesh) = shape else {
            panic!("expected mesh shape");
        };
        assert!(mesh
            .vertices
            .iter()
            .any(|v| v.pos == focal && v.color == egui::Color32::RED));
    }

    #[test]
    fn test_radial_gradient_t_uses_centered_outer_circle() {
        let t = radial_gradient_t(
            egui::pos2(11.0, 5.0),
            egui::pos2(5.0, 5.0),
            egui::pos2(7.0, 5.0),
            10.0,
        );
        assert!((t - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_transform_inverse_roundtrip() {
        let transform = Transform2D::translate(3.0, -2.0).then(Transform2D::scale(2.0, 4.0));
        let inverse = transform.inverse().expect("invertible transform");
        let point = egui::pos2(7.0, 11.0);
        let roundtrip = inverse.apply(transform.apply(point));
        assert!((roundtrip.x - point.x).abs() < 0.001);
        assert!((roundtrip.y - point.y).abs() < 0.001);
    }

    #[test]
    fn test_blend_color_normal() {
        // Normal: result is fg (fully opaque)
        let result = blend_color(opaque(200, 100, 50), opaque(50, 50, 50), BlendMode::Normal);
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert_eq!(r, 200);
        assert_eq!(g, 100);
        assert_eq!(b, 50);
    }

    #[test]
    fn test_blend_color_multiply() {
        // Multiply: white * white = white
        let result = blend_color(
            opaque(255, 255, 255),
            opaque(255, 255, 255),
            BlendMode::Multiply,
        );
        let [r, _g, _b, _] = result.to_srgba_unmultiplied();
        assert_eq!(r, 255);
        // Multiply: black * anything = black
        let result2 = blend_color(opaque(0, 0, 0), opaque(200, 100, 50), BlendMode::Multiply);
        let [r2, g2, b2, _] = result2.to_srgba_unmultiplied();
        assert_eq!(r2, 0);
        assert_eq!(g2, 0);
        assert_eq!(b2, 0);
    }

    #[test]
    fn test_blend_color_screen() {
        // Screen: black screen anything = anything
        let result = blend_color(opaque(0, 0, 0), opaque(200, 100, 50), BlendMode::Screen);
        let [r, g, _b, _] = result.to_srgba_unmultiplied();
        assert!((r as i32 - 200).abs() <= 2, "r={}", r);
        assert!((g as i32 - 100).abs() <= 2, "g={}", g);
        // Screen: white screen anything = white
        let result2 = blend_color(
            opaque(255, 255, 255),
            opaque(100, 100, 100),
            BlendMode::Screen,
        );
        let [r2, _, _, _] = result2.to_srgba_unmultiplied();
        assert_eq!(r2, 255);
    }

    #[test]
    fn test_blend_color_difference() {
        // Difference: same color = black
        let result = blend_color(
            opaque(100, 100, 100),
            opaque(100, 100, 100),
            BlendMode::Difference,
        );
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert!(
            r <= 2 && g <= 2 && b <= 2,
            "expected near-black, got ({},{},{})",
            r,
            g,
            b
        );
    }

    #[test]
    fn test_blend_color_exclusion() {
        // Exclusion: same color = near-black (2*c*(1-c) subtracted)
        let result = blend_color(
            opaque(128, 128, 128),
            opaque(128, 128, 128),
            BlendMode::Exclusion,
        );
        let [r, _, _, _] = result.to_srgba_unmultiplied();
        // 0.5 + 0.5 - 2*0.5*0.5 = 0.5 → ~128
        assert!((r as i32 - 128).abs() <= 3, "r={}", r);
    }

    #[test]
    fn test_blend_color_hsl_modes_no_panic() {
        // HSL modes should not panic for any input
        for mode in [
            BlendMode::Hue,
            BlendMode::Saturation,
            BlendMode::Color,
            BlendMode::Luminosity,
        ] {
            let _ = blend_color(opaque(200, 100, 50), opaque(50, 150, 200), mode);
        }
    }

    #[test]
    fn test_blend_color_color_dodge_white_fg() {
        // ColorDodge: white fg → white result
        let result = blend_color(
            opaque(255, 255, 255),
            opaque(100, 100, 100),
            BlendMode::ColorDodge,
        );
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_blend_color_hard_light() {
        // HardLight with black fg → black result (2*0*bg = 0)
        let result = blend_color(opaque(0, 0, 0), opaque(200, 100, 50), BlendMode::HardLight);
        let [r, g, b, _] = result.to_srgba_unmultiplied();
        assert!(
            r <= 2 && g <= 2 && b <= 2,
            "expected near-black, got ({},{},{})",
            r,
            g,
            b
        );
    }

    #[test]
    fn test_composite_layers_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                composite_layers(ui, vec![]);
            });
        });
    }

    #[test]
    fn test_composite_layers_behavior() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let shape1 = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let shape2 = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::BLUE,
                ));
                let layer1 = BlendLayer::new(vec![shape1])
                    .blend_mode(BlendMode::Normal)
                    .opacity(1.0);
                let layer2 = BlendLayer::new(vec![shape2])
                    .blend_mode(BlendMode::Multiply)
                    .opacity(0.5);

                composite_layers(ui, vec![layer1, layer2]);
            });
        });
    }

    #[test]
    fn test_rasterize_composited_layers_per_pixel_blend() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0));
        let red = egui::Shape::Rect(egui::epaint::RectShape::filled(
            rect,
            egui::CornerRadius::ZERO,
            egui::Color32::RED,
        ));
        let blue = egui::Shape::Rect(egui::epaint::RectShape::filled(
            rect,
            egui::CornerRadius::ZERO,
            egui::Color32::BLUE,
        ));
        let (_, size, pixels, unhandled) = rasterize_composited_layers_result(&[
            BlendLayer::new(vec![red]),
            BlendLayer::new(vec![blue]).blend_mode(BlendMode::Multiply),
        ])
        .expect("layers rasterize");
        assert_eq!(size, [2, 2]);
        assert!(unhandled.is_empty());
        let [r, g, b, a] = pixels[0].to_srgba_unmultiplied();
        assert_eq!((r, g, b, a), (0, 0, 0, 255));
    }

    #[test]
    fn test_rasterize_composited_layers_handles_ellipse() {
        let ellipse = egui::Shape::ellipse_filled(
            egui::pos2(5.0, 3.0),
            egui::vec2(5.0, 3.0),
            egui::Color32::RED,
        );
        let (_, size, pixels, unhandled) =
            rasterize_composited_layers_result(&[BlendLayer::new(vec![ellipse])])
                .expect("ellipse rasterizes");
        assert_eq!(size, [10, 6]);
        assert!(unhandled.is_empty());
        assert_ne!(
            pixels[(3 * size[0] + 5) as usize],
            egui::Color32::TRANSPARENT
        );
    }

    #[test]
    fn test_rasterize_composited_layers_preserves_rounded_rect_corners() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0));
        let rounded = egui::Shape::Rect(egui::epaint::RectShape::filled(
            rect,
            egui::CornerRadius::same(5),
            egui::Color32::RED,
        ));
        let (_, size, pixels, unhandled) =
            rasterize_composited_layers_result(&[BlendLayer::new(vec![rounded])])
                .expect("rect rasterizes");
        assert_eq!(size, [10, 10]);
        assert!(unhandled.is_empty());
        assert_eq!(pixels[0], egui::Color32::TRANSPARENT);
        assert_ne!(
            pixels[(5 * size[0] + 5) as usize],
            egui::Color32::TRANSPARENT
        );
    }

    #[test]
    fn test_rasterize_composited_layers_uses_stroke_aware_bounds() {
        let circle = egui::Shape::circle_stroke(
            egui::pos2(5.0, 5.0),
            5.0,
            egui::Stroke::new(4.0, egui::Color32::RED),
        );
        let (_, size, _, unhandled) =
            rasterize_composited_layers_result(&[BlendLayer::new(vec![circle])])
                .expect("stroke rasterizes");
        assert_eq!(size, [14, 14]);
        assert!(unhandled.is_empty());
    }

    #[test]
    fn test_rasterize_composited_layers_reports_oversized_group() {
        let huge = egui::Shape::Rect(egui::epaint::RectShape::filled(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(4097.0, 2.0)),
            egui::CornerRadius::ZERO,
            egui::Color32::RED,
        ));

        let error = rasterize_composited_layers_result(&[BlendLayer::new(vec![huge])])
            .expect_err("oversized blend group should report an explicit error");
        assert!(matches!(
            error,
            RasterizeBlendError::LayerTooLarge {
                width: 4097,
                height: 2,
                max: 4096
            }
        ));
    }

    #[test]
    fn test_rasterize_composited_layers_reports_unsupported_shapes() {
        let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0)),
            egui::CornerRadius::ZERO,
            egui::Color32::RED,
        ));
        let error =
            rasterize_composited_layers_result(&[BlendLayer::new(vec![rect, egui::Shape::Noop])])
                .expect_err("unsupported shape should report an explicit error");

        assert_eq!(error, RasterizeBlendError::UnsupportedShapes { count: 1 });
    }

    #[test]
    fn test_rasterize_composited_layers_reports_invalid_layer_clip_polygon() {
        let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0)),
            egui::CornerRadius::ZERO,
            egui::Color32::RED,
        ));
        let mut layer = BlendLayer::new(vec![rect]);
        layer
            .clip_polygons
            .push(vec![egui::Pos2::ZERO, egui::Pos2::ZERO, egui::Pos2::ZERO]);

        let error = rasterize_composited_layers_result(&[layer])
            .expect_err("degenerate layer clip polygon should report an explicit error");

        assert_eq!(error, RasterizeBlendError::InvalidClipMask);
    }

    #[test]
    fn test_blend_layer_clip_polygon_preserves_invalid_request_for_reporting() {
        let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0)),
            egui::CornerRadius::ZERO,
            egui::Color32::RED,
        ));
        let layer = BlendLayer::new(vec![rect]).clip_polygon(vec![egui::Pos2::ZERO]);

        let error = rasterize_composited_layers_result(&[layer])
            .expect_err("builder-provided invalid clip polygon should remain reportable");

        assert_eq!(error, RasterizeBlendError::InvalidClipMask);
    }

    #[test]
    fn test_composite_layers_report_records_empty_input() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let report = composite_layers_report(ui, vec![]);
                assert_eq!(report.issues.len(), 1);
                assert_eq!(
                    report.issues[0].kind,
                    crate::render::RenderIssueKind::EmptyInput
                );
            });
        });
    }

    #[test]
    fn test_clipped_layers_gpu_report_paints_fallback_on_raster_error() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let report = clipped_layers_gpu_report(
                    ui,
                    &[
                        egui::pos2(0.0, 0.0),
                        egui::pos2(2.0, 0.0),
                        egui::pos2(2.0, 2.0),
                    ],
                    vec![BlendLayer::new(vec![rect, egui::Shape::Noop])],
                );

                assert_eq!(
                    report.issues[0].kind,
                    crate::render::RenderIssueKind::UnsupportedShape
                );
            });
        });
        assert!(
            !output.shapes.is_empty(),
            "compatibility fallback should paint original shapes when clipped compositing cannot rasterize exactly"
        );
    }

    #[test]
    fn test_clipped_layers_gpu_report_zero_area_polygon_is_invalid() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(4.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let report = clipped_layers_gpu_report(
                    ui,
                    &[egui::Pos2::ZERO, egui::Pos2::ZERO, egui::Pos2::ZERO],
                    vec![BlendLayer::new(vec![rect])],
                );

                assert_eq!(
                    report.issues.last().expect("invalid polygon issue").kind,
                    crate::render::RenderIssueKind::InvalidBounds
                );
                assert_eq!(
                    report.actual_quality,
                    crate::render::RenderQuality::Approximate
                );
            });
        });
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn test_polygon_alpha_mask_clears_outside_pixels() {
        let mut pixels = vec![egui::Color32::WHITE; 4];
        apply_polygon_alpha_mask(
            &mut pixels,
            2,
            2,
            egui::Pos2::ZERO,
            &[
                egui::pos2(0.0, 0.0),
                egui::pos2(1.0, 0.0),
                egui::pos2(1.0, 1.0),
                egui::pos2(0.0, 1.0),
            ],
        );
        assert!(pixels.contains(&egui::Color32::TRANSPARENT));
        assert!(pixels.contains(&egui::Color32::WHITE));
    }

    #[test]
    fn test_clip_mask_even_odd_hole_clears_inner_pixels() {
        let mut pixels = vec![egui::Color32::WHITE; 16];
        let outer = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(4.0, 0.0),
            egui::pos2(4.0, 4.0),
            egui::pos2(0.0, 4.0),
        ];
        let inner = vec![
            egui::pos2(1.0, 1.0),
            egui::pos2(3.0, 1.0),
            egui::pos2(3.0, 3.0),
            egui::pos2(1.0, 3.0),
        ];
        let mask = ClipMask::compound_even_odd(vec![outer, inner]);

        apply_clip_mask(&mut pixels, 4, 4, egui::Pos2::ZERO, &mask);

        assert_eq!(pixels[0], egui::Color32::WHITE);
        assert_eq!(pixels[(4 + 1) as usize], egui::Color32::TRANSPARENT);
        assert_eq!(pixels[(2 * 4 + 2) as usize], egui::Color32::TRANSPARENT);
    }

    #[test]
    fn test_clip_mask_alpha_samples_threshold() {
        let mut pixels = vec![egui::Color32::WHITE; 4];
        let mask = ClipMask::alpha(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(2.0)),
            [2, 2],
            vec![255, 0, 0, 255],
            128,
        );

        apply_clip_mask(&mut pixels, 2, 2, egui::Pos2::ZERO, &mask);

        assert_eq!(pixels[0], egui::Color32::WHITE);
        assert_eq!(pixels[1], egui::Color32::TRANSPARENT);
        assert_eq!(pixels[2], egui::Color32::TRANSPARENT);
        assert_eq!(pixels[3], egui::Color32::WHITE);
    }

    #[test]
    fn test_clipped_layers_mask_report_uses_cpu_exact_path() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(4.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let mask = ClipMask::rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::splat(4.0),
                ));
                let report =
                    clipped_layers_mask_report(ui, &mask, vec![BlendLayer::new(vec![rect])]);
                assert!(report.is_exact());
                assert_eq!(
                    report.backend,
                    crate::render::RenderBackendKind::CpuOffscreen
                );
            });
        });
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn test_clipped_layers_mask_report_invalid_mask_paints_unmasked_fallback() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(4.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let invalid = ClipMask::compound_even_odd(vec![vec![egui::Pos2::ZERO]]);
                let report =
                    clipped_layers_mask_report(ui, &invalid, vec![BlendLayer::new(vec![rect])]);
                assert_eq!(
                    report.issues[0].kind,
                    crate::render::RenderIssueKind::InvalidBounds
                );
            });
        });
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn test_clipped_layers_mask_report_zero_area_rect_is_invalid() {
        let ctx = egui::Context::default();
        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Shape::Rect(egui::epaint::RectShape::filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(4.0)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::RED,
                ));
                let invalid = ClipMask::rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::ZERO,
                ));
                let report =
                    clipped_layers_mask_report(ui, &invalid, vec![BlendLayer::new(vec![rect])]);
                assert_eq!(
                    report.issues[0].kind,
                    crate::render::RenderIssueKind::InvalidBounds
                );
                assert_eq!(
                    report.actual_quality,
                    crate::render::RenderQuality::Approximate
                );
            });
        });
        assert!(!output.shapes.is_empty());
    }

    #[test]
    fn test_clipped_shape_approx_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                clipped_shape_approx(ui, &[], true, |_| {});
            });
        });
    }

    #[cfg(feature = "clip-mask")]
    #[test]
    fn test_clipped_shape_cpu_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                clipped_shape_cpu(ui, &[], |_| {});
            });
        });
    }

    #[cfg(feature = "clip-mask")]
    #[test]
    fn test_clipped_shape_cpu_behavior() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let polygon1 = vec![
                    egui::pos2(0.0, 0.0),
                    egui::pos2(10.0, 0.0),
                    egui::pos2(10.0, 10.0),
                ];
                let polygon2 = vec![
                    egui::pos2(10.0, 10.0),
                    egui::pos2(20.0, 10.0),
                    egui::pos2(20.0, 20.0),
                ];
                clipped_shape_cpu(ui, &polygon1, |ui| {
                    ui.label("Clipped 1");
                });
                clipped_shape_cpu(ui, &polygon2, |ui| {
                    ui.label("Clipped 2");
                });
            });
        });
    }

    #[test]
    fn test_paint_image_from_path_missing() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0));
                let success = paint_image_from_path(
                    ui,
                    ui.painter(),
                    rect,
                    "nonexistent.png",
                    "test_id",
                    egui::Color32::WHITE,
                );
                assert!(!success);
            });
        });
    }

    #[test]
    fn test_bevel_join_emits_segmented_geometry() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(10.0, 0.0),
            egui::pos2(10.0, 10.0),
        ];
        let stroke = RichStroke {
            width: 2.0,
            color: egui::Color32::WHITE,
            dash: None,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Bevel,
        };

        let shapes = dashed_path_shapes(&points, &stroke);
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn test_layered_painter_from_ui_and_layers() {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                let layered = LayeredPainter::from_ui(ui);
                let clip = ui.clip_rect();
                assert_eq!(layered.background().clip_rect(), clip);
                assert_eq!(layered.main().clip_rect(), clip);
                assert_eq!(layered.foreground().clip_rect(), clip);
            });
        });
    }

    #[test]
    fn test_transform_apply_to_shape_and_rect() {
        let transform = Transform2D::translate(10.0, 5.0).then(Transform2D::scale(2.0, 3.0));
        let rect = egui::Rect::from_min_size(egui::pos2(1.0, 2.0), egui::vec2(3.0, 4.0));
        let transformed_rect = transform.apply_to_rect(rect);
        assert_eq!(transformed_rect.min, egui::pos2(22.0, 21.0));
        assert_eq!(transformed_rect.max, egui::pos2(28.0, 33.0));

        let shape = Shape::LineSegment {
            points: [egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)],
            stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
        };
        match transform.apply_to_shape(shape) {
            Shape::LineSegment { points, .. } => {
                assert_eq!(points[0], egui::pos2(20.0, 15.0));
                assert_eq!(points[1], egui::pos2(22.0, 18.0));
            }
            other => panic!("unexpected shape: {:?}", other),
        }
    }

    #[test]
    fn test_stack_align_to_align2() {
        assert_eq!(StackAlign::TopLeft.to_align2(), egui::Align2::LEFT_TOP);
        assert_eq!(StackAlign::Center.to_align2(), egui::Align2::CENTER_CENTER);
        assert_eq!(
            StackAlign::BottomRight.to_align2(),
            egui::Align2::RIGHT_BOTTOM
        );
    }
}
