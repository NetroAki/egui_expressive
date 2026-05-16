//! Cross-platform app-provided backdrop snapshot contracts.
//!
//! This module deliberately does not capture native host framebuffers. Host apps
//! may install a provider that returns exact RGBA pixels for a requested egui
//! logical rectangle in a single egui context/surface; higher-fidelity render
//! paths can then blur that supplied snapshot without claiming OS/native backdrop
//! capture. Multi-window or multi-viewport capture needs a later provider
//! contract extension.

use std::sync::Arc;

const BACKDROP_SNAPSHOT_PROVIDER_ID: &str = "egui_expressive.backdrop_snapshot_provider";
#[cfg(feature = "wgpu")]
const APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID: &str =
    "egui_expressive.app_owned_offscreen_backdrop_source";
pub const MAX_BACKDROP_SNAPSHOT_AXIS: u32 = 4_096;

pub type SharedBackdropSnapshotProvider = Arc<dyn BackdropSnapshotProvider + Send + Sync + 'static>;

#[cfg(feature = "wgpu")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AppOwnedBackdropSurfaceId(pub u64);

#[cfg(feature = "wgpu")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AppOwnedBackdropFrameId(pub u64);

#[cfg(feature = "wgpu")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AppOwnedBackdropAlphaMode {
    Straight,
}

#[cfg(feature = "wgpu")]
#[derive(Clone, Debug)]
pub struct AppOwnedOffscreenBackdropSource {
    pub surface_id: AppOwnedBackdropSurfaceId,
    pub frame_id: AppOwnedBackdropFrameId,
    pub pixels_per_point: f32,
    pub physical_size: [u32; 2],
    pub format: egui_wgpu::wgpu::TextureFormat,
    pub sample_count: u32,
    pub alpha_mode: AppOwnedBackdropAlphaMode,
    pub texture_view: Arc<egui_wgpu::wgpu::TextureView>,
}

#[cfg(feature = "wgpu")]
pub type SharedAppOwnedOffscreenBackdropSource = Arc<AppOwnedOffscreenBackdropSource>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackdropCaptureRequest {
    pub rect: egui::Rect,
    pub pixels_per_point: f32,
    pub requested_width: u32,
    pub requested_height: u32,
}

impl BackdropCaptureRequest {
    pub fn new(rect: egui::Rect, pixels_per_point: f32) -> Result<Self, BackdropCaptureError> {
        if !rect.min.x.is_finite()
            || !rect.min.y.is_finite()
            || !rect.max.x.is_finite()
            || !rect.max.y.is_finite()
            || rect.width() <= 0.0
            || rect.height() <= 0.0
        {
            return Err(BackdropCaptureError::InvalidRect);
        }
        if !pixels_per_point.is_finite() || pixels_per_point <= 0.0 {
            return Err(BackdropCaptureError::InvalidScale);
        }

        let requested_width = physical_axis(rect.width(), pixels_per_point)?;
        let requested_height = physical_axis(rect.height(), pixels_per_point)?;

        Ok(Self {
            rect,
            pixels_per_point,
            requested_width,
            requested_height,
        })
    }

    pub fn expected_len(self) -> Result<usize, BackdropCaptureError> {
        expected_rgba_len(self.requested_width, self.requested_height)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackdropSnapshot {
    pub width: u32,
    pub height: u32,
    /// Tightly packed row-major 8-bit sRGB RGBA pixels with straight alpha.
    pub pixels: Vec<u8>,
}

impl BackdropSnapshot {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, BackdropCaptureError> {
        let expected_len = expected_rgba_len(width, height)?;
        if pixels.len() != expected_len {
            return Err(BackdropCaptureError::InvalidPixelLength {
                expected: expected_len,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn validate_for_request(
        &self,
        request: &BackdropCaptureRequest,
    ) -> Result<(), BackdropCaptureError> {
        if self.width != request.requested_width || self.height != request.requested_height {
            return Err(BackdropCaptureError::SnapshotSizeMismatch {
                expected: [request.requested_width, request.requested_height],
                actual: [self.width, self.height],
            });
        }
        let expected = request.expected_len()?;
        if self.pixels.len() != expected {
            return Err(BackdropCaptureError::InvalidPixelLength {
                expected,
                actual: self.pixels.len(),
            });
        }
        Ok(())
    }
}

pub trait BackdropSnapshotProvider {
    fn capture_backdrop_snapshot(
        &self,
        request: &BackdropCaptureRequest,
    ) -> Result<BackdropSnapshot, BackdropCaptureError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackdropCaptureError {
    InvalidRect,
    InvalidScale,
    SizeBudgetExceeded {
        width: u32,
        height: u32,
    },
    SnapshotSizeMismatch {
        expected: [u32; 2],
        actual: [u32; 2],
    },
    InvalidPixelLength {
        expected: usize,
        actual: usize,
    },
    ProviderUnavailable,
    CaptureFailed(String),
}

pub fn install_backdrop_snapshot_provider(
    ctx: &egui::Context,
    provider: SharedBackdropSnapshotProvider,
) {
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(BACKDROP_SNAPSHOT_PROVIDER_ID), provider);
    });
}

pub fn load_backdrop_snapshot_provider(
    ctx: &egui::Context,
) -> Option<SharedBackdropSnapshotProvider> {
    ctx.data(|data| data.get_temp(egui::Id::new(BACKDROP_SNAPSHOT_PROVIDER_ID)))
}

#[cfg(feature = "wgpu")]
pub fn install_app_owned_offscreen_backdrop_source(
    ctx: &egui::Context,
    source: SharedAppOwnedOffscreenBackdropSource,
) {
    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new(APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID),
            source,
        );
    });
}

#[cfg(feature = "wgpu")]
pub fn load_app_owned_offscreen_backdrop_source(
    ctx: &egui::Context,
) -> Option<SharedAppOwnedOffscreenBackdropSource> {
    ctx.data(|data| data.get_temp(egui::Id::new(APP_OWNED_OFFSCREEN_BACKDROP_SOURCE_ID)))
}

fn physical_axis(size_points: f32, pixels_per_point: f32) -> Result<u32, BackdropCaptureError> {
    let pixels = (size_points * pixels_per_point).ceil();
    if !pixels.is_finite() || pixels <= 0.0 {
        return Err(BackdropCaptureError::InvalidRect);
    }
    if pixels > MAX_BACKDROP_SNAPSHOT_AXIS as f32 {
        return Err(BackdropCaptureError::SizeBudgetExceeded {
            width: pixels as u32,
            height: pixels as u32,
        });
    }
    Ok(pixels as u32)
}

fn expected_rgba_len(width: u32, height: u32) -> Result<usize, BackdropCaptureError> {
    if width == 0
        || height == 0
        || width > MAX_BACKDROP_SNAPSHOT_AXIS
        || height > MAX_BACKDROP_SNAPSHOT_AXIS
    {
        return Err(BackdropCaptureError::SizeBudgetExceeded { width, height });
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(BackdropCaptureError::SizeBudgetExceeded { width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SolidProvider;

    impl BackdropSnapshotProvider for SolidProvider {
        fn capture_backdrop_snapshot(
            &self,
            request: &BackdropCaptureRequest,
        ) -> Result<BackdropSnapshot, BackdropCaptureError> {
            BackdropSnapshot::new(
                request.requested_width,
                request.requested_height,
                vec![255; request.expected_len()?],
            )
        }
    }

    #[test]
    fn capture_request_uses_ceil_physical_size() {
        let rect = egui::Rect::from_min_size(egui::pos2(4.0, 8.0), egui::vec2(10.1, 4.2));
        let request = BackdropCaptureRequest::new(rect, 2.0).unwrap();
        assert_eq!(request.requested_width, 21);
        assert_eq!(request.requested_height, 9);
        assert_eq!(request.expected_len().unwrap(), 21 * 9 * 4);
    }

    #[test]
    fn capture_request_rejects_invalid_scale_and_bounds() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1.0, 1.0));
        assert_eq!(
            BackdropCaptureRequest::new(rect, 0.0).unwrap_err(),
            BackdropCaptureError::InvalidScale
        );
        let empty = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(0.0, 1.0));
        assert_eq!(
            BackdropCaptureRequest::new(empty, 1.0).unwrap_err(),
            BackdropCaptureError::InvalidRect
        );
    }

    #[test]
    fn snapshot_validation_requires_exact_size_and_length() {
        let request = BackdropCaptureRequest::new(
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(2.0, 2.0)),
            1.0,
        )
        .unwrap();
        let snapshot = BackdropSnapshot::new(2, 2, vec![128; 16]).unwrap();
        assert!(snapshot.validate_for_request(&request).is_ok());

        let wrong = BackdropSnapshot::new(1, 4, vec![128; 16]).unwrap();
        assert!(matches!(
            wrong.validate_for_request(&request),
            Err(BackdropCaptureError::SnapshotSizeMismatch { .. })
        ));
        assert!(matches!(
            BackdropSnapshot::new(2, 2, vec![0; 15]),
            Err(BackdropCaptureError::InvalidPixelLength { .. })
        ));
    }

    #[test]
    fn provider_installation_is_context_scoped() {
        let ready_ctx = egui::Context::default();
        let other_ctx = egui::Context::default();
        assert!(load_backdrop_snapshot_provider(&ready_ctx).is_none());
        assert!(load_backdrop_snapshot_provider(&other_ctx).is_none());

        install_backdrop_snapshot_provider(&ready_ctx, Arc::new(SolidProvider));

        assert!(load_backdrop_snapshot_provider(&ready_ctx).is_some());
        assert!(load_backdrop_snapshot_provider(&other_ctx).is_none());
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn app_owned_backdrop_tokens_are_stable_value_types() {
        let surface = AppOwnedBackdropSurfaceId(7);
        let frame = AppOwnedBackdropFrameId(11);
        assert_eq!(surface, AppOwnedBackdropSurfaceId(7));
        assert_eq!(frame, AppOwnedBackdropFrameId(11));
        assert_eq!(
            AppOwnedBackdropAlphaMode::Straight,
            AppOwnedBackdropAlphaMode::Straight
        );
    }
}
