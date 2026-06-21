#[cfg(all(target_os = "android", feature = "shared-showcase", not(feature = "android-smoke-minimal")))]
#[path = "../../../examples/cross_platform_showcase.rs"]
mod cross_platform_showcase;

#[cfg(all(
    target_os = "android",
    any(feature = "android-smoke-minimal", not(feature = "shared-showcase"))
))]
use eframe::egui;

#[cfg(target_os = "android")]
macro_rules! android_smoke_log {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        #[cfg(feature = "android-smoke-verbose")]
        log::info!(concat!("[android_smoke_trace] ", $fmt) $(, $arg)*);
    };
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("egui_expressive_showcase")
            .with_max_level(android_log_level()),
    );
    install_android_panic_hook();

    android_smoke_log!("logger initialized; entering android_main");

    android_smoke_log!(
        "android_main entered; package=dev.egui_expressive.showcase; verbose={}; minimal_smoke={}; shared_showcase={}",
        cfg!(feature = "android-smoke-verbose"),
        cfg!(feature = "android-smoke-minimal"),
        cfg!(feature = "shared-showcase")
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Glow,
        run_and_return: false,
        persist_window: false,
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        ..Default::default()
    };

    android_smoke_log!(
        "native options prepared; renderer=Glow; run_and_return=false; persist_window=false; vsync=true; hardware_acceleration=Required"
    );

    if let Err(error) = eframe::run_native(
        "egui_expressive Showcase",
        options,
        Box::new(|cc| {
            android_smoke_log!(
                "app creator called; pixels_per_point={:.3}; cumulative_frame_nr={}",
                cc.egui_ctx.pixels_per_point(),
                cc.egui_ctx.cumulative_frame_nr()
            );
            Ok(create_android_app(cc))
        }),
    ) {
        log::error!("failed to run egui_expressive Android showcase: {error}");
    }

    android_smoke_log!("eframe::run_native returned");
}

#[cfg(target_os = "android")]
fn android_log_level() -> log::LevelFilter {
    if cfg!(feature = "android-smoke-verbose") {
        // Keep dependency chatter out of logcat. Detailed app diagnostics are
        // emitted at INFO with an `[android_smoke_trace]` marker.
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Info
    }
}

#[cfg(target_os = "android")]
fn install_android_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("Android panic: {panic_info}");
    }));
}

#[cfg(all(target_os = "android", feature = "android-smoke-minimal"))]
fn create_android_app(_cc: &eframe::CreationContext<'_>) -> Box<dyn eframe::App> {
    android_smoke_log!("creating AndroidSmokeApp diagnostic fallback");
    Box::new(AndroidSmokeApp::default())
}

#[cfg(all(target_os = "android", not(feature = "android-smoke-minimal"), feature = "shared-showcase"))]
fn create_android_app(cc: &eframe::CreationContext<'_>) -> Box<dyn eframe::App> {
    android_smoke_log!("creating shared CrossPlatformShowcase");
    Box::new(cross_platform_showcase::CrossPlatformShowcase::new(&cc.egui_ctx))
}

#[cfg(all(target_os = "android", not(feature = "android-smoke-minimal"), not(feature = "shared-showcase")))]
fn create_android_app(_cc: &eframe::CreationContext<'_>) -> Box<dyn eframe::App> {
    android_smoke_log!("shared-showcase feature disabled; falling back to AndroidSmokeApp");
    Box::new(AndroidSmokeApp::default())
}

#[cfg(all(
    target_os = "android",
    any(feature = "android-smoke-minimal", not(feature = "shared-showcase"))
))]
struct AndroidSmokeApp {
    frame_count: u64,
    last_rect: egui::Rect,
    last_ppp: f32,
}

#[cfg(all(
    target_os = "android",
    any(feature = "android-smoke-minimal", not(feature = "shared-showcase"))
))]
impl Default for AndroidSmokeApp {
    fn default() -> Self {
        Self {
            frame_count: 0,
            last_rect: egui::Rect::NOTHING,
            last_ppp: 1.0,
        }
    }
}

#[cfg(all(
    target_os = "android",
    any(feature = "android-smoke-minimal", not(feature = "shared-showcase"))
))]
impl eframe::App for AndroidSmokeApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Intentionally bright. If screenshots stay black, eframe/glow did not clear
        // the Android surface, so the failure is below widget layout.
        egui::Color32::from_rgb(126, 34, 206).to_normalized_gamma_f32()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(feature = "android-smoke-verbose")]
        if self.frame_count <= 5 || self.frame_count % 60 == 0 {
            log::info!(
                "[android_smoke_trace] logic; cumulative_frame_nr={}; ppp={:.3}",
                ctx.cumulative_frame_nr(),
                ctx.pixels_per_point()
            );
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.frame_count += 1;

        let input_snapshot = ui.ctx().input(|input| {
            (
                input.content_rect(),
                input.viewport_rect(),
                input.viewport().inner_rect,
                input.stable_dt,
            )
        });
        let (content_rect, screen_rect, inner_rect, stable_dt) = input_snapshot;
        self.last_rect = content_rect;
        self.last_ppp = ui.ctx().pixels_per_point();

        let cpu_ms = frame
            .info()
            .cpu_usage
            .map(|seconds| seconds * 1000.0)
            .unwrap_or(-1.0);

        if self.frame_count <= 20 || self.frame_count % 60 == 0 {
            android_smoke_log!(
                "ui frame={}; content_rect={:?}; screen_rect={:?}; inner_rect={:?}; available={:?}; ppp={:.3}; stable_dt_ms={:.3}; cpu_ms={:.3}",
                self.frame_count,
                content_rect,
                screen_rect,
                inner_rect,
                ui.available_size(),
                self.last_ppp,
                stable_dt * 1000.0,
                cpu_ms
            );
        }

        let panel_rect = ui.max_rect();
        ui.painter()
            .rect_filled(panel_rect, 0.0, egui::Color32::from_rgb(16, 185, 129));
        ui.painter().rect_filled(
            panel_rect.shrink(16.0),
            24.0,
            egui::Color32::from_rgb(30, 64, 175),
        );

        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.heading(egui::RichText::new("egui_expressive Android smoke").size(34.0));
            ui.label(
                egui::RichText::new(
                    "If this is visible, NativeActivity + egui_glow renderer + egui paint works.",
                )
                .size(22.0)
                .color(egui::Color32::WHITE),
            );
            ui.add_space(24.0);
            ui.label(
                egui::RichText::new(format!(
                    "frame={}  ppp={:.2}  content={:.0}x{:.0}",
                    self.frame_count,
                    self.last_ppp,
                    content_rect.width(),
                    content_rect.height()
                ))
                .size(24.0)
                .color(egui::Color32::YELLOW),
            );
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Logging feature: android-smoke-verbose")
                    .size(18.0)
                    .color(egui::Color32::LIGHT_GRAY),
            );
        });
    }
}

#[cfg(not(target_os = "android"))]
pub fn host_build_marker() -> &'static str {
    "egui_expressive Android showcase host crate"
}
