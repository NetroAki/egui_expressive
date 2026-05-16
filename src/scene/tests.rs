use super::*;

#[cfg(feature = "wgpu")]
static GPU_INIT_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn appearance_stack_detects_offscreen_required_effects() {
    let stack = AppearanceStack {
        entries: vec![AppearanceEntry::Effect(EffectLayer {
            effect_type: EffectType::GaussianBlur,
            params: EffectDef {
                effect_type: EffectType::GaussianBlur,
                radius: 8.0,
                ..EffectDef::default()
            },
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        })],
    };

    assert!(stack.requires_offscreen());
}

#[test]
fn non_wgpu_gaussian_blur_and_feather_report_approximate() {
    let request = crate::render::OffscreenRequest {
        feature: crate::render::RenderFeature::Blur,
        width: 64,
        height: 64,
        requested_quality: crate::render::RenderQuality::Exact,
    };

    for effect_type in [EffectType::GaussianBlur, EffectType::Feather] {
        let report = scene_blur_effect_report(
            effect_type,
            &crate::render::RenderCapabilities::egui_native(),
            request,
            SceneBlurEffectContract::exact_solid_rect_source(),
        );
        assert_eq!(
            report.actual_quality,
            crate::render::RenderQuality::Approximate
        );
        assert_eq!(
            report.issues[0].kind,
            crate::render::RenderIssueKind::ApproximateFallback
        );
        assert!(report.issues[0].message.contains("soft_shadow"));
    }
}

#[test]
fn wgpu_scene_blur_report_is_exact_within_budget_only() {
    let capabilities = crate::render::RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
    let request = crate::render::OffscreenRequest {
        feature: crate::render::RenderFeature::Blur,
        width: 4_096,
        height: 12,
        requested_quality: crate::render::RenderQuality::Exact,
    };

    let report = scene_blur_effect_report(
        EffectType::GaussianBlur,
        &capabilities,
        request,
        SceneBlurEffectContract::exact_solid_rect_source(),
    );
    assert!(report.is_exact());

    let oversized = crate::render::OffscreenRequest {
        width: 4_097,
        ..request
    };
    let rejected = scene_blur_effect_report(
        EffectType::GaussianBlur,
        &capabilities,
        oversized,
        SceneBlurEffectContract::exact_solid_rect_source(),
    );
    assert_eq!(
        rejected.actual_quality,
        crate::render::RenderQuality::Unsupported
    );
    assert_eq!(
        rejected.issues[0].kind,
        crate::render::RenderIssueKind::SizeBudgetExceeded
    );
    assert!(rejected.issues[0].message.contains("per-axis 4096"));
}

#[test]
fn wgpu_scene_shaped_blur_report_is_exact_for_approved_contract_only() {
    let capabilities = crate::render::RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
    let request = crate::render::OffscreenRequest {
        feature: crate::render::RenderFeature::Blur,
        width: 128,
        height: 96,
        requested_quality: crate::render::RenderQuality::Exact,
    };

    for effect_type in [EffectType::GaussianBlur, EffectType::Feather] {
        let report = scene_blur_effect_report(
            effect_type,
            &capabilities,
            request,
            SceneBlurEffectContract::exact_shaped_source(),
        );
        assert!(report.is_exact());
    }

    let rejected = scene_blur_effect_report(
        EffectType::GaussianBlur,
        &capabilities,
        request,
        SceneBlurEffectContract {
            solid_rect_source: false,
            shaped_source: false,
            normal_blend: true,
            gpu_resources_ready: true,
        },
    );
    assert_eq!(
        rejected.issues[0].kind,
        crate::render::RenderIssueKind::UnsupportedFeature
    );
}

#[test]
fn wgpu_scene_shadow_report_is_exact_within_budget_only() {
    let capabilities = crate::render::RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
    let request = crate::render::OffscreenRequest {
        feature: crate::render::RenderFeature::Shadow,
        width: 4_096,
        height: 12,
        requested_quality: crate::render::RenderQuality::Exact,
    };

    let report = scene_shadow_effect_report(
        EffectType::DropShadow,
        4.0,
        &capabilities,
        request,
        SceneBlurEffectContract::exact_solid_rect_source(),
    );
    assert!(report.is_exact());

    let oversized = crate::render::OffscreenRequest {
        width: 4_097,
        ..request
    };
    let rejected = scene_shadow_effect_report(
        EffectType::OuterGlow,
        4.0,
        &capabilities,
        oversized,
        SceneBlurEffectContract::exact_solid_rect_source(),
    );
    assert_eq!(
        rejected.actual_quality,
        crate::render::RenderQuality::Unsupported
    );
    assert_eq!(
        rejected.issues[0].kind,
        crate::render::RenderIssueKind::SizeBudgetExceeded
    );
    assert!(rejected.issues[0].message.contains("Phase 9B"));

    for (effect_type, requested_radius) in [
        (EffectType::DropShadow, 0.0),
        (EffectType::DropShadow, 0.5),
        (EffectType::OuterGlow, 0.0),
    ] {
        let low_radius = scene_shadow_effect_report(
            effect_type,
            requested_radius,
            &capabilities,
            request,
            SceneBlurEffectContract::exact_solid_rect_source(),
        );
        assert_eq!(
            low_radius.actual_quality,
            crate::render::RenderQuality::Approximate
        );
        assert_eq!(
            low_radius.issues[0].kind,
            crate::render::RenderIssueKind::ApproximateFallback
        );
        assert!(low_radius.issues[0].message.contains("blur/radius >= 1.0"));
    }
}

#[test]
fn wgpu_scene_shaped_shadow_report_is_exact_for_approved_contract_only() {
    let capabilities = crate::render::RenderCapabilities::egui_wgpu_callback(4_096 * 4_096);
    let request = crate::render::OffscreenRequest {
        feature: crate::render::RenderFeature::Shadow,
        width: 128,
        height: 96,
        requested_quality: crate::render::RenderQuality::Exact,
    };

    for effect_type in [EffectType::DropShadow, EffectType::OuterGlow] {
        let report = scene_shadow_effect_report(
            effect_type,
            4.0,
            &capabilities,
            request,
            SceneBlurEffectContract::exact_shaped_source(),
        );
        assert!(report.is_exact());
    }

    let rejected = scene_shadow_effect_report(
        EffectType::DropShadow,
        4.0,
        &capabilities,
        request,
        SceneBlurEffectContract {
            solid_rect_source: false,
            shaped_source: false,
            normal_blend: true,
            gpu_resources_ready: true,
        },
    );
    assert_eq!(
        rejected.issues[0].kind,
        crate::render::RenderIssueKind::UnsupportedFeature
    );
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9a_scene_blur_uses_source_layer_callback_when_eligible() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::GaussianBlur,
                blur: 4.0,
                color: egui::Color32::from_rgb(40, 80, 120),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(matches!(shapes.as_slice(), [egui::Shape::Callback(_)]));
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9a_scene_blur_falls_back_when_gpu_effects_not_initialized() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::GaussianBlur,
                blur: 4.0,
                color: egui::Color32::from_rgb(40, 80, 120),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9a_scene_blur_falls_back_for_non_normal_blend() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            let mut effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::Feather,
                radius: 4.0,
                color: egui::Color32::from_rgb(40, 80, 120),
                ..Default::default()
            });
            effect.blend_mode = BlendMode::Multiply;

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9a_scene_blur_falls_back_for_open_path_source() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Path {
                points: vec![egui::pos2(0.0, 0.0), egui::pos2(16.0, 0.0)],
                closed: false,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::Feather,
                radius: 4.0,
                color: egui::Color32::from_rgb(40, 80, 120),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn r100_003a_shaped_scene_effects_use_source_layer_callback_when_eligible() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let rounded_rect = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 12.0)),
                corner_radius: 4.0,
            };
            let ellipse = Geometry::Ellipse {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 12.0)),
            };
            let closed_path = Geometry::Path {
                points: vec![
                    egui::pos2(0.0, 0.0),
                    egui::pos2(18.0, 2.0),
                    egui::pos2(12.0, 14.0),
                    egui::pos2(2.0, 10.0),
                ],
                closed: true,
            };
            let rotated_rect = rotate_geometry(
                &Geometry::Rect {
                    rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 12.0)),
                    corner_radius: 0.0,
                },
                25.0,
            );

            for (geometry, effect) in [
                (
                    rounded_rect,
                    EffectLayer::new(crate::codegen::EffectDef {
                        effect_type: EffectType::GaussianBlur,
                        blur: 4.0,
                        color: egui::Color32::from_rgb(40, 80, 120),
                        ..Default::default()
                    }),
                ),
                (
                    ellipse,
                    EffectLayer::new(crate::codegen::EffectDef {
                        effect_type: EffectType::DropShadow,
                        x: 2.0,
                        y: 3.0,
                        blur: 4.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                        ..Default::default()
                    }),
                ),
                (
                    closed_path,
                    EffectLayer::new(crate::codegen::EffectDef {
                        effect_type: EffectType::Feather,
                        radius: 4.0,
                        color: egui::Color32::from_rgb(80, 20, 120),
                        ..Default::default()
                    }),
                ),
                (
                    rotated_rect,
                    EffectLayer::new(crate::codegen::EffectDef {
                        effect_type: EffectType::DropShadow,
                        x: 2.0,
                        y: 3.0,
                        blur: 4.0,
                        spread: 0.0,
                        color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                        ..Default::default()
                    }),
                ),
            ] {
                let shapes = paint_effect(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &effect,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(matches!(shapes.as_slice(), [egui::Shape::Callback(_)]));
            }
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9a_scene_blur_falls_back_when_source_rect_exceeds_budget() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(4097.0, 12.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::GaussianBlur,
                blur: 4.0,
                color: egui::Color32::from_rgb(40, 80, 120),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9b_scene_drop_shadow_uses_source_layer_callback_when_eligible() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::DropShadow,
                x: 3.0,
                y: 2.0,
                blur: 4.0,
                spread: 1.0,
                color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(matches!(shapes.as_slice(), [egui::Shape::Callback(_)]));
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9b_scene_outer_glow_uses_source_layer_callback_when_eligible() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::OuterGlow,
                blur: 4.0,
                color: egui::Color32::from_rgba_unmultiplied(40, 80, 120, 160),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(matches!(shapes.as_slice(), [egui::Shape::Callback(_)]));
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9b_scene_shadow_respects_requested_blend_when_force_normal_painting() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            let mut effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::DropShadow,
                x: 3.0,
                y: 2.0,
                blur: 4.0,
                color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                ..Default::default()
            });
            effect.blend_mode = BlendMode::Multiply;

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                true,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9b_scene_shadow_falls_back_when_expanded_bounds_exceed_budget() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(4_090.0, 12.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::DropShadow,
                blur: 8.0,
                color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn phase9b_scene_shadow_falls_back_for_low_requested_radius() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(16.0, 12.0)),
                corner_radius: 0.0,
            };
            for (effect_type, blur, radius) in [
                (EffectType::DropShadow, 0.0, 0.0),
                (EffectType::DropShadow, 0.5, 0.0),
                (EffectType::OuterGlow, 0.0, 0.0),
            ] {
                let effect = EffectLayer::new(crate::codegen::EffectDef {
                    effect_type,
                    blur,
                    radius,
                    color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                    ..Default::default()
                });

                let shapes = paint_effect(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &effect,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(!shapes
                    .iter()
                    .any(|shape| matches!(shape, egui::Shape::Callback(_))));
                assert!(!shapes.is_empty());
            }
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn r100_003a_shaped_shadow_falls_back_when_spread_is_requested() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Ellipse {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 12.0)),
            };
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::DropShadow,
                x: 2.0,
                y: 3.0,
                blur: 4.0,
                spread: 1.0,
                color: egui::Color32::from_rgba_unmultiplied(10, 20, 30, 180),
                ..Default::default()
            });

            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes
                .iter()
                .any(|shape| matches!(shape, egui::Shape::Callback(_))));
            assert!(!shapes.is_empty());
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[cfg(feature = "wgpu")]
#[test]
fn r100_003a_group_and_mesh_sources_remain_non_exact() {
    let _guard = GPU_INIT_TEST_LOCK.lock().expect("gpu init test lock");
    crate::gpu::set_gpu_effects_initialized_for_tests(true);
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let effect = EffectLayer::new(crate::codegen::EffectDef {
                effect_type: EffectType::GaussianBlur,
                blur: 4.0,
                color: egui::Color32::from_rgb(40, 80, 120),
                ..Default::default()
            });
            let group = Geometry::Group {
                bounds: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 12.0)),
            };
            let mesh = Geometry::MeshPatch {
                corners: [
                    egui::pos2(0.0, 0.0),
                    egui::pos2(18.0, 0.0),
                    egui::pos2(18.0, 12.0),
                    egui::pos2(0.0, 12.0),
                ],
                colors: [
                    egui::Color32::RED,
                    egui::Color32::GREEN,
                    egui::Color32::BLUE,
                    egui::Color32::YELLOW,
                ],
                subdivisions: 2,
            };

            for geometry in [group, mesh] {
                let shapes = paint_effect(
                    ui,
                    egui::Vec2::ZERO,
                    &geometry,
                    &effect,
                    1.0,
                    &BlendMode::Normal,
                    false,
                );
                assert!(!shapes
                    .iter()
                    .any(|shape| matches!(shape, egui::Shape::Callback(_))));
                assert!(!shapes.is_empty());
            }
        });
    });
    crate::gpu::set_gpu_effects_initialized_for_tests(false);
}

#[test]
fn scene_node_rect_preserves_bounds() {
    let node = SceneNode::rect(
        "rect",
        egui::Rect::from_min_size(egui::pos2(1.0, 2.0), egui::vec2(3.0, 4.0)),
        2.0,
    );
    assert_eq!(node.geometry.bounds().min, egui::pos2(1.0, 2.0));
    assert_eq!(node.geometry.bounds().max, egui::pos2(4.0, 6.0));
}

#[test]
fn path_point_helpers_create_egui_points() {
    assert_eq!(
        path_points(&[(1.0, 2.0), (3.0, 4.0)]),
        vec![egui::pos2(1.0, 2.0), egui::pos2(3.0, 4.0)]
    );
    assert_eq!(
        offset_path_points(egui::pos2(10.0, 20.0), &[(1.0, 2.0), (3.0, 4.0)]),
        vec![egui::pos2(11.0, 22.0), egui::pos2(13.0, 24.0)]
    );
}

#[test]
fn scene_node_preserves_layout_rotation() {
    let mut elem = crate::codegen::LayoutElement::new(
        "rot".to_string(),
        crate::codegen::ElementType::Shape,
        0.0,
        0.0,
        10.0,
        20.0,
    );
    elem.rotation_deg = 45.0;
    let node = SceneNode::from_layout_element(&elem);
    assert_eq!(node.rotation_deg, 45.0);
}

#[test]
fn scene_node_path_geometry_does_not_double_rotate() {
    let mut elem = crate::codegen::LayoutElement::new(
        "path_rot".to_string(),
        crate::codegen::ElementType::Shape,
        0.0,
        0.0,
        10.0,
        10.0,
    );
    elem.rotation_deg = 45.0;
    elem.path_closed = true;
    elem.path_points = vec![
        crate::codegen::PathPoint {
            anchor: [0.0, 0.0],
            left_ctrl: [0.0, 0.0],
            right_ctrl: [0.0, 0.0],
        },
        crate::codegen::PathPoint {
            anchor: [10.0, 0.0],
            left_ctrl: [10.0, 0.0],
            right_ctrl: [10.0, 0.0],
        },
        crate::codegen::PathPoint {
            anchor: [10.0, 10.0],
            left_ctrl: [10.0, 10.0],
            right_ctrl: [10.0, 10.0],
        },
    ];
    let node = SceneNode::from_layout_element(&elem);
    assert_eq!(node.rotation_deg, 0.0);
    assert!(matches!(node.geometry, Geometry::Path { .. }));
}

#[test]
fn rotate_rect_geometry_converts_to_closed_path() {
    let geometry = Geometry::Rect {
        rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 10.0)),
        corner_radius: 4.0,
    };
    let rotated = rotate_geometry(&geometry, 30.0);
    let Geometry::Path { points, closed } = rotated else {
        panic!("expected rotated rect path");
    };
    assert!(closed);
    assert!(points.len() > 4);
}

#[test]
fn bezier_layout_path_is_tessellated() {
    let points = vec![
        crate::codegen::PathPoint {
            anchor: [0.0, 0.0],
            left_ctrl: [0.0, 0.0],
            right_ctrl: [0.0, 10.0],
        },
        crate::codegen::PathPoint {
            anchor: [10.0, 0.0],
            left_ctrl: [10.0, 10.0],
            right_ctrl: [10.0, 0.0],
        },
    ];

    let sampled = sample_layout_path(&points, false);
    assert!(sampled.len() > points.len());
    assert!(sampled.iter().any(|p| p.y > 0.0));
}

#[test]
fn path_gradient_fill_produces_mesh_shape() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let fill = FillLayer {
                paint: PaintSource::LinearGradient(GradientDef {
                    gradient_type: crate::codegen::GradientType::Linear,
                    angle_deg: 0.0,
                    center: None,
                    focal_point: None,
                    radius: None,
                    transform: None,
                    stops: vec![
                        crate::codegen::GradientStop {
                            position: 0.0,
                            color: egui::Color32::RED,
                        },
                        crate::codegen::GradientStop {
                            position: 1.0,
                            color: egui::Color32::BLUE,
                        },
                    ],
                }),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            };
            let geometry = Geometry::Path {
                points: vec![
                    egui::pos2(0.0, 0.0),
                    egui::pos2(20.0, 0.0),
                    egui::pos2(20.0, 20.0),
                ],
                closed: true,
            };
            let shapes = paint_fill(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &fill,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(matches!(shapes.first(), Some(egui::Shape::Mesh(_))));
        });
    });
}

#[test]
fn ellipse_gradient_fill_produces_mesh_shape() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let fill = FillLayer {
                paint: PaintSource::RadialGradient(GradientDef {
                    gradient_type: crate::codegen::GradientType::Radial,
                    angle_deg: 0.0,
                    center: Some([10.0, 10.0]),
                    focal_point: Some([10.0, 10.0]),
                    radius: Some(10.0),
                    transform: None,
                    stops: vec![
                        crate::codegen::GradientStop {
                            position: 0.0,
                            color: egui::Color32::RED,
                        },
                        crate::codegen::GradientStop {
                            position: 1.0,
                            color: egui::Color32::BLUE,
                        },
                    ],
                }),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            };
            let geometry = Geometry::Ellipse {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 20.0)),
            };
            let shapes = paint_fill(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &fill,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(matches!(shapes.first(), Some(egui::Shape::Mesh(_))));
        });
    });
}

#[test]
fn pattern_fill_produces_editable_vector_shapes() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let fill = FillLayer {
                paint: PaintSource::Pattern(PatternDef {
                    name: "dots".to_string(),
                    seed: 7,
                    foreground: egui::Color32::BLACK,
                    background: egui::Color32::TRANSPARENT,
                    cell_size: 4.0,
                    mark_size: 1.0,
                }),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            };
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 20.0)),
                corner_radius: 4.0,
            };
            let shapes = paint_fill(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &fill,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(shapes.len() > 1);
            assert!(shapes.iter().any(|shape| matches!(
                shape,
                egui::Shape::LineSegment { .. } | egui::Shape::Circle(_)
            )));
        });
    });
}

#[test]
fn dashed_rect_stroke_uses_rich_path_shapes() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let stroke = StrokeLayer {
                paint: PaintSource::Solid(egui::Color32::BLACK),
                width: 2.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                cap: Some(StrokeCap::Round),
                join: Some(StrokeJoin::Bevel),
                dash: Some(vec![2.0, 2.0]),
                miter_limit: Some(1.0),
            };
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 20.0)),
                corner_radius: 4.0,
            };
            let shapes = paint_stroke(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &stroke,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(shapes.len() > 1);
            assert!(shapes
                .iter()
                .all(|shape| !matches!(shape, egui::Shape::Rect(_))));
        });
    });
}

#[test]
fn dashed_ellipse_stroke_uses_rich_path_shapes() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let stroke = StrokeLayer {
                paint: PaintSource::Solid(egui::Color32::BLACK),
                width: 2.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                cap: Some(StrokeCap::Round),
                join: Some(StrokeJoin::Round),
                dash: Some(vec![2.0, 2.0]),
                miter_limit: None,
            };
            let geometry = Geometry::Ellipse {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 12.0)),
            };
            let shapes = paint_stroke(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &stroke,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(shapes.len() > 1);
            assert!(shapes
                .iter()
                .all(|shape| !matches!(shape, egui::Shape::Rect(_))));
        });
    });
}

#[test]
fn gradient_stroke_renders_representative_vector_shapes() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let stroke = StrokeLayer {
                paint: PaintSource::LinearGradient(GradientDef {
                    gradient_type: crate::codegen::GradientType::Linear,
                    angle_deg: 0.0,
                    center: None,
                    focal_point: None,
                    radius: None,
                    transform: None,
                    stops: vec![
                        crate::codegen::GradientStop {
                            position: 0.0,
                            color: egui::Color32::RED,
                        },
                        crate::codegen::GradientStop {
                            position: 1.0,
                            color: egui::Color32::BLUE,
                        },
                    ],
                }),
                width: 3.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                cap: None,
                join: None,
                dash: Some(vec![3.0, 2.0]),
                miter_limit: None,
            };
            let geometry = Geometry::Path {
                points: vec![
                    egui::pos2(0.0, 0.0),
                    egui::pos2(20.0, 0.0),
                    egui::pos2(20.0, 20.0),
                ],
                closed: false,
            };
            let shapes = paint_stroke(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &stroke,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(
                !shapes.is_empty(),
                "gradient strokes should render bounded vector shapes instead of disappearing"
            );
        });
    });
}

#[test]
fn pattern_stroke_renders_representative_vector_shapes() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let stroke = StrokeLayer {
                paint: PaintSource::Pattern(PatternDef {
                    name: "dots".to_string(),
                    seed: 4,
                    foreground: egui::Color32::GREEN,
                    background: egui::Color32::TRANSPARENT,
                    cell_size: 4.0,
                    mark_size: 1.0,
                }),
                width: 2.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
                cap: Some(StrokeCap::Round),
                join: None,
                dash: None,
                miter_limit: None,
            };
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(20.0, 12.0)),
                corner_radius: 0.0,
            };
            let shapes = paint_stroke(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &stroke,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(
                !shapes.is_empty(),
                "pattern strokes should render bounded vector shapes instead of disappearing"
            );
        });
    });
}

#[test]
fn test_render_scene_empty() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let scene = ArtboardScene::default();
            render_scene(ui, &scene);
        });
    });
}

#[test]
fn render_scene_clip_children_uses_mask_capable_group_path() {
    let mut clip = SceneNode::rect(
        "clip",
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(12.0)),
        0.0,
    )
    .with_clip_children(true);
    clip.children.push(
        SceneNode::rect(
            "child",
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(24.0)),
            0.0,
        )
        .with_fill(PaintSource::Solid(egui::Color32::RED)),
    );
    let scene = ArtboardScene {
        name: "clip-mask-routing".to_owned(),
        width: 32.0,
        height: 32.0,
        nodes: vec![clip],
    };

    let ctx = egui::Context::default();
    let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| render_scene(ui, &scene));
    });

    assert!(
        !output.shapes.is_empty(),
        "clip_children scene should render through the mask-capable offscreen group path"
    );
}

#[test]
fn nested_clip_children_preserve_descendant_layers_with_masks() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let mut outer = SceneNode::rect(
                "outer_clip",
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(20.0)),
                0.0,
            );
            outer.clip_children = true;

            let mut inner = SceneNode::rect(
                "inner_clip",
                egui::Rect::from_min_size(egui::pos2(2.0, 2.0), egui::Vec2::splat(10.0)),
                0.0,
            );
            inner.clip_children = true;

            let mut child = SceneNode::rect(
                "child",
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(30.0)),
                0.0,
            );
            child
                .appearance
                .entries
                .push(AppearanceEntry::Fill(FillLayer {
                    paint: PaintSource::Solid(egui::Color32::RED),
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                }));

            inner.children.push(child);
            outer.children.push(inner);

            let mut layers = Vec::new();
            collect_node_layers(ui, egui::Vec2::ZERO, &outer, 1.0, &mut layers);
            assert_eq!(layers.len(), 1);
            assert_eq!(layers[0].clip_polygons.len(), 2);
        });
    });
}

#[test]
fn nested_invalid_clip_children_preserve_polygon_for_reporting() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let mut clip = SceneNode::path(
                "invalid_clip",
                vec![egui::pos2(0.0, 0.0), egui::pos2(4.0, 0.0)],
                true,
            );
            clip.clip_children = true;
            clip.children.push(
                SceneNode::rect(
                    "child",
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(8.0)),
                    0.0,
                )
                .with_fill(PaintSource::Solid(egui::Color32::RED)),
            );

            let mut layers = Vec::new();
            collect_node_layers(ui, egui::Vec2::ZERO, &clip, 1.0, &mut layers);

            assert_eq!(layers.len(), 1);
            assert_eq!(layers[0].clip_polygons.len(), 1);
            assert_eq!(layers[0].clip_polygons[0].len(), 2);
        });
    });
}

#[test]
fn test_render_geometry_appearance_empty() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                corner_radius: 0.0,
            };
            let appearance = AppearanceStack::default();
            let painter = ui.painter().clone();
            render_geometry_appearance(
                ui,
                &painter,
                egui::Vec2::ZERO,
                &geometry,
                &appearance,
                1.0,
                &BlendMode::Normal,
            );
        });
    });
}

#[test]
fn test_paint_effect_drop_shadow() {
    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            let geometry = Geometry::Rect {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(10.0)),
                corner_radius: 0.0,
            };
            let effect = EffectLayer {
                effect_type: EffectType::DropShadow,
                params: EffectDef::default(),
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            };
            let shapes = paint_effect(
                ui,
                egui::Vec2::ZERO,
                &geometry,
                &effect,
                1.0,
                &BlendMode::Normal,
                false,
            );
            assert!(!shapes.is_empty());
        });
    });
}

#[test]
fn test_scene_group_opacity_and_blend_mode() {
    let mut scene = ArtboardScene::default();
    let mut group = SceneNode::rect(
        "group",
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(100.0)),
        0.0,
    );
    group.opacity = 0.5;
    group.blend_mode = BlendMode::Multiply;

    let mut child = SceneNode::rect(
        "child",
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(50.0)),
        0.0,
    );
    child
        .appearance
        .entries
        .push(AppearanceEntry::Fill(FillLayer {
            paint: PaintSource::Solid(egui::Color32::RED),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        }));

    group.children.push(child);
    scene.nodes.push(group);

    let ctx = egui::Context::default();
    let _ = ctx.run_ui(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show_inside(ctx, |ui| {
            render_scene(ui, &scene);
        });
    });
}
