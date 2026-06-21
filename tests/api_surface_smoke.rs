//! Stage 11 – API Surface Smoke Tests
//!
//! Compile-time reachability checks for representative public API paths.
//! No rendering, no `egui::Context`, no behavior-heavy assertions.

use egui_expressive::{
    app_provided_backdrop_blur_report, app_provided_backdrop_blur_shape,
    install_backdrop_capture_source_contract, install_backdrop_snapshot_provider,
    install_backdrop_snapshot_provider_with_source_contract, load_backdrop_capture_source_contract,
    load_backdrop_snapshot_provider, BackdropCaptureError, BackdropCaptureRequest,
    BackdropSnapshot, BackdropSnapshotProvider, BlurQuality, BreakpointName, Breakpoints, DataCell,
    DataColumn, DataGridModel, DataGridState, DataRow, FocusScope, FormSchema, InteractionState,
    Knob, LargeCanvas, Meter, Responsive, SelectionModel, SnapGrid, StateMachine, StateSlot,
    StepGrid, Tw, ViewportCuller, VisualDiffConfig,
};

// ---------------------------------------------------------------------------
// Representative public reachability paths
// ---------------------------------------------------------------------------

#[test]
fn responsive_value_construct() {
    let _r: Responsive<u32> = Responsive::new(0).sm(1).md(2).lg(3).xl(4).xxl(5);
}

#[test]
fn breakpoints_construct() {
    let _bp = Breakpoints::default();
    let _bp = Breakpoints::tailwind();
}

#[test]
fn breakpoint_name_variants_exist() {
    let _ = [
        BreakpointName::Xs,
        BreakpointName::Sm,
        BreakpointName::Md,
        BreakpointName::Lg,
        BreakpointName::Xl,
        BreakpointName::Xxl,
    ];
}

#[test]
fn data_grid_model_construct() {
    let _m = DataGridModel::default();
    let col = DataColumn::new("id", "ID").width(200.0);
    let row = DataRow::new("r1", vec![DataCell::new("hello")]);
    let _m2 = DataGridModel::new(vec![col], vec![row]);
}

#[test]
fn data_grid_state_construct() {
    let _s = DataGridState::default();
}

#[test]
fn visual_diff_config_construct() {
    let _cfg = VisualDiffConfig::default();
}

// ---------------------------------------------------------------------------
// Additional constructor-based reachability checks
// ---------------------------------------------------------------------------

#[test]
fn tw_builder_construct() {
    let _tw = Tw::new();
    let _tw2 = Tw::default();
    let _tw3 = Tw::new().backdrop_blur_app_provided(8.0);
    let _source = egui_expressive::tailwind::TwBackdropSource::AppProvidedSnapshot;
}

#[test]
fn form_schema_construct() {
    let _s = FormSchema::default();
    let _s2 = FormSchema::new(vec![]);
}

#[test]
fn focus_scope_construct() {
    let _scope = FocusScope::new("test_scope");
}

#[test]
fn selection_model_construct() {
    let _sm: SelectionModel<u32> = SelectionModel::default();
}

#[test]
fn snap_grid_construct() {
    let _ = SnapGrid::disabled();
    let _ = SnapGrid::uniform(16.0);
    let _ = SnapGrid::new(Some(8.0), Some(16.0));
    let _ = SnapGrid::default();
}

#[test]
fn blur_quality_variants() {
    let _ = BlurQuality::Fast;
    let _ = BlurQuality::Medium;
    let _ = BlurQuality::High;
}

// ---------------------------------------------------------------------------
// Type-reference reachability checks
// ---------------------------------------------------------------------------

#[test]
fn state_slot_and_machine_type_references() {
    // StateSlot and StateMachine require egui::Context for methods,
    // so we only prove they resolve via PhantomData references.
    let _ = std::marker::PhantomData::<StateSlot<String>>;
    let _ = std::marker::PhantomData::<StateMachine<String>>;
}

#[test]
fn interaction_state_construct() {
    let _is = InteractionState::default();
}

#[test]
fn large_canvas_and_viewport_culler_type_references() {
    // LargeCanvas and ViewportCuller require constructor params we
    // don't have without a live egui context; prove they resolve.
    let _ = std::marker::PhantomData::<LargeCanvas>;
    let _ = std::marker::PhantomData::<ViewportCuller>;
}

// ---------------------------------------------------------------------------
// Default-feature widget reachability checks
// ---------------------------------------------------------------------------

#[test]
fn widget_type_references_compile() {
    let _ = std::marker::PhantomData::<Knob>;
    let _ = std::marker::PhantomData::<Meter>;
    let _ = std::marker::PhantomData::<StepGrid>;
}

#[test]
fn breadcrumbs_type_reference() {
    let _items: Vec<egui_expressive::BreadcrumbItem> = vec![];
}

#[test]
fn backdrop_snapshot_public_api_paths_compile() {
    let _source_install_fn: fn(&egui::Context, egui_expressive::BackdropCaptureSourceContract) =
        install_backdrop_capture_source_contract;
    let _source_load_fn: fn(
        &egui::Context,
    ) -> Option<egui_expressive::BackdropCaptureSourceContract> =
        load_backdrop_capture_source_contract;
    let _install_with_source_fn: fn(
        &egui::Context,
        egui_expressive::SharedBackdropSnapshotProvider,
        egui_expressive::BackdropCaptureSourceContract,
    ) = install_backdrop_snapshot_provider_with_source_contract;
    let _report_fn: fn(&egui::Context, egui::Rect, f32) -> egui_expressive::RenderReport =
        app_provided_backdrop_blur_report;
    let _shape_fn: fn(
        &egui::Ui,
        egui::Rect,
        f32,
    ) -> (Option<egui::Shape>, egui_expressive::RenderReport) = app_provided_backdrop_blur_shape;
    let _expected_contract_fn: fn(
        egui::Rect,
        f32,
        egui_expressive::BackdropCaptureSourceContract,
        egui_expressive::BackdropCaptureSourceId,
        egui_expressive::BackdropCaptureProviderId,
    ) -> Result<
        egui_expressive::BackdropCaptureRequest,
        egui_expressive::BackdropCaptureError,
    > = egui_expressive::BackdropCaptureRequest::with_expected_source_contract;
    let _install_fn: fn(&egui::Context, egui_expressive::SharedBackdropSnapshotProvider) =
        install_backdrop_snapshot_provider;
    let _load_fn: fn(&egui::Context) -> Option<egui_expressive::SharedBackdropSnapshotProvider> =
        load_backdrop_snapshot_provider;

    let _ = std::marker::PhantomData::<BackdropCaptureRequest>;
    let _ = std::marker::PhantomData::<egui_expressive::BackdropCaptureSourceId>;
    let _ = std::marker::PhantomData::<egui_expressive::BackdropCaptureProviderId>;
    let _ = std::marker::PhantomData::<egui_expressive::BackdropCaptureSurfaceToken>;
    let _ = std::marker::PhantomData::<egui_expressive::BackdropCaptureFrameToken>;
    let _ = egui_expressive::BackdropCaptureConsent::AppOwnedSurface;
    let _ = egui_expressive::BackdropFrameFreshness::CurrentFrame;
    let _ = egui_expressive::BackdropOcclusionState::Unoccluded;
    let _ = egui_expressive::BackdropOcclusionState::NotChecked;
    let _ = egui_expressive::BackdropCapturePixelFormat::Rgba8SrgbStraightAlpha;
    let _ = std::marker::PhantomData::<BackdropSnapshot>;
    let _ = std::marker::PhantomData::<BackdropCaptureError>;
    let _ = std::marker::PhantomData::<dyn BackdropSnapshotProvider + Send + Sync>;
}

#[cfg(feature = "native-backdrop")]
#[test]
fn native_backdrop_common_api_paths_compile() {
    use egui_expressive::{
        NativeBackdropContractDiagnostic, NativeBackdropInitError, NativeBackdropPermissionState,
        NativeBackdropPlatform, NativeBackdropSmokeArtifact, NativeBackdropSourceScope,
        NativeBackdropSupportFamily, NativeBackdropSupportState, NATIVE_BACKDROP_FEATURE,
        NATIVE_BACKDROP_MACOS_FEATURE, NATIVE_BACKDROP_WAYLAND_FEATURE,
        NATIVE_BACKDROP_WINDOWS_FEATURE, NATIVE_BACKDROP_X11_FEATURE,
    };

    assert_eq!(NATIVE_BACKDROP_FEATURE, "native-backdrop");
    assert_eq!(NATIVE_BACKDROP_X11_FEATURE, "native-backdrop-x11");
    assert_eq!(NATIVE_BACKDROP_MACOS_FEATURE, "native-backdrop-macos");
    assert_eq!(NATIVE_BACKDROP_WINDOWS_FEATURE, "native-backdrop-windows");
    assert_eq!(NATIVE_BACKDROP_WAYLAND_FEATURE, "native-backdrop-wayland");
    assert_eq!(
        NativeBackdropPlatform::X11.feature_name(),
        NATIVE_BACKDROP_X11_FEATURE
    );
    let _ = std::marker::PhantomData::<NativeBackdropInitError>;
    let _ = NativeBackdropSupportFamily::Android.label();
    let _ = NativeBackdropContractDiagnostic::SmokeArtifactMissing.redacted_message();
    let _ = NativeBackdropSmokeArtifact {
        family: NativeBackdropSupportFamily::LinuxX11,
        support_state: NativeBackdropSupportState::ContractOnly,
        permission_state: NativeBackdropPermissionState::NotRequested,
        source_scope: NativeBackdropSourceScope::AppWindowSurface,
        redaction_confirmed: true,
    };
}

#[cfg(feature = "wgpu")]
#[test]
fn app_owned_offscreen_backdrop_public_api_paths_compile() {
    use egui_expressive::{
        app_owned_offscreen_backdrop_blur_report, app_owned_offscreen_backdrop_blur_shape,
        bind_app_owned_offscreen_backdrop_source_for_context, init_gpu_effects,
        init_gpu_effects_for_context, install_app_owned_offscreen_backdrop_source,
        load_app_owned_offscreen_backdrop_source, AppOwnedBackdropAlphaMode,
        AppOwnedBackdropFrameId, AppOwnedBackdropSurfaceId, AppOwnedOffscreenBackdropSource,
        OffscreenRequest, RenderBackendKind, RenderCapabilities, RenderFeature, RenderIssueKind,
        RenderQuality, SharedAppOwnedOffscreenBackdropSource, WgpuLifecycleFailure,
    };

    let _install_fn: fn(&egui::Context, SharedAppOwnedOffscreenBackdropSource) =
        install_app_owned_offscreen_backdrop_source;
    let _load_fn: fn(&egui::Context) -> Option<SharedAppOwnedOffscreenBackdropSource> =
        load_app_owned_offscreen_backdrop_source;
    type AppOwnedBackdropReportFn = fn(
        &egui::Context,
        egui::Rect,
        f32,
        AppOwnedBackdropSurfaceId,
        AppOwnedBackdropFrameId,
    ) -> egui_expressive::RenderReport;
    type AppOwnedBackdropShapeFn = fn(
        &egui::Ui,
        egui::Rect,
        f32,
        AppOwnedBackdropSurfaceId,
        AppOwnedBackdropFrameId,
    ) -> (Option<egui::Shape>, egui_expressive::RenderReport);

    let _report_fn: AppOwnedBackdropReportFn = app_owned_offscreen_backdrop_blur_report;
    let _shape_fn: AppOwnedBackdropShapeFn = app_owned_offscreen_backdrop_blur_shape;
    let _bind_fn: fn(
        &egui_wgpu::RenderState,
        &egui::Context,
        AppOwnedBackdropSurfaceId,
        AppOwnedBackdropFrameId,
    ) -> egui_expressive::RenderReport = bind_app_owned_offscreen_backdrop_source_for_context;
    let _init_fn: fn(&egui_wgpu::RenderState) = init_gpu_effects;
    let _init_for_context_fn: fn(&egui_wgpu::RenderState, &egui::Context) =
        init_gpu_effects_for_context;
    let _lifecycle_report_fn: fn(
        RenderFeature,
        RenderBackendKind,
        WgpuLifecycleFailure,
        &'static str,
    ) -> egui_expressive::RenderReport = egui_expressive::wgpu_lifecycle_report;
    let _host_framebuffer_report_fn: fn(
        &egui_expressive::RenderCapabilities,
        OffscreenRequest,
    ) -> egui_expressive::RenderReport = egui_expressive::host_framebuffer_backdrop_report;
    let _app_provided_report_fn: fn(
        &egui_expressive::RenderCapabilities,
        OffscreenRequest,
        egui_expressive::BackdropCaptureSourceContract,
    ) -> egui_expressive::RenderReport =
        egui_expressive::wgpu_app_provided_backdrop_snapshot_report;

    let callback_caps = RenderCapabilities::egui_wgpu_callback(4_096);
    let offscreen_caps = RenderCapabilities::wgpu_offscreen(4_096, true);
    assert_eq!(callback_caps.backend, RenderBackendKind::EguiWgpuCallback);
    assert_eq!(offscreen_caps.backend, RenderBackendKind::WgpuOffscreen);

    let report = egui_expressive::wgpu_lifecycle_report(
        RenderFeature::BackdropBlur,
        callback_caps.backend,
        WgpuLifecycleFailure::MissingContextBinding,
        WgpuLifecycleFailure::MissingContextBinding.default_message(),
    );
    assert_eq!(report.actual_quality, RenderQuality::Unsupported);
    assert_eq!(
        report.issues[0].kind,
        RenderIssueKind::MissingContextBinding
    );
    assert!(report.issues[0].kind.is_wgpu_lifecycle());

    let _ = std::marker::PhantomData::<AppOwnedBackdropSurfaceId>;
    let _ = std::marker::PhantomData::<AppOwnedBackdropFrameId>;
    let _ = std::marker::PhantomData::<AppOwnedBackdropAlphaMode>;
    let _ = std::marker::PhantomData::<AppOwnedOffscreenBackdropSource>;
    let _ = WgpuLifecycleFailure::MissingRuntime;
}
