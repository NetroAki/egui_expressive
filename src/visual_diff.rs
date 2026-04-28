//! Image-diff helpers for design-tool parity tests.
//!
//! These utilities do not promise that every renderer will be pixel-identical.
//! They provide a shared, explicit tolerance contract so Illustrator reference
//! exports and egui_expressive renders can be compared without hand-waving.

use image::{ImageResult, Rgba, RgbaImage};
use std::path::Path;

/// Tolerances for comparing two RGBA images.
#[derive(Clone, Copy, Debug)]
pub struct VisualDiffConfig {
    /// Maximum allowed absolute channel delta for any compared channel.
    pub max_channel_delta: u8,
    /// Maximum allowed mean absolute channel delta over the whole image.
    pub max_mean_delta: f32,
    /// Maximum ratio of pixels whose channel delta exceeds `max_channel_delta`.
    pub max_bad_pixel_ratio: f32,
    /// Include alpha in channel-delta measurements.
    pub compare_alpha: bool,
}

impl Default for VisualDiffConfig {
    fn default() -> Self {
        Self {
            max_channel_delta: 2,
            max_mean_delta: 0.5,
            max_bad_pixel_ratio: 0.001,
            compare_alpha: true,
        }
    }
}

/// Result of comparing two images.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualDiffReport {
    pub expected_size: [u32; 2],
    pub actual_size: [u32; 2],
    pub total_pixels: u64,
    pub bad_pixels: u64,
    pub max_channel_delta: u8,
    pub mean_channel_delta: f32,
    pub bad_pixel_ratio: f32,
    pub dimension_mismatch: bool,
    pub passed: bool,
}

impl VisualDiffReport {
    pub fn summary(&self) -> String {
        format!(
            "visual diff: passed={} size={:?}->{:?} bad_pixels={}/{} ({:.4}%) max_delta={} mean_delta={:.3}",
            self.passed,
            self.expected_size,
            self.actual_size,
            self.bad_pixels,
            self.total_pixels,
            self.bad_pixel_ratio * 100.0,
            self.max_channel_delta,
            self.mean_channel_delta
        )
    }
}

/// Compare two in-memory RGBA images with an explicit tolerance contract.
pub fn diff_rgba_images(
    expected: &RgbaImage,
    actual: &RgbaImage,
    config: VisualDiffConfig,
) -> VisualDiffReport {
    let expected_size = [expected.width(), expected.height()];
    let actual_size = [actual.width(), actual.height()];
    let total_pixels = u64::from(expected.width()) * u64::from(expected.height());

    if expected_size != actual_size {
        return VisualDiffReport {
            expected_size,
            actual_size,
            total_pixels,
            bad_pixels: total_pixels,
            max_channel_delta: u8::MAX,
            mean_channel_delta: f32::INFINITY,
            bad_pixel_ratio: 1.0,
            dimension_mismatch: true,
            passed: false,
        };
    }

    let channels = if config.compare_alpha { 4 } else { 3 };
    let mut bad_pixels = 0_u64;
    let mut max_delta = 0_u8;
    let mut total_delta = 0_u64;

    for (expected_px, actual_px) in expected.pixels().zip(actual.pixels()) {
        let mut pixel_max_delta = 0_u8;
        for channel in 0..channels {
            let delta = expected_px[channel].abs_diff(actual_px[channel]);
            pixel_max_delta = pixel_max_delta.max(delta);
            max_delta = max_delta.max(delta);
            total_delta += u64::from(delta);
        }
        if pixel_max_delta > config.max_channel_delta {
            bad_pixels += 1;
        }
    }

    let channel_count = (total_pixels as f32) * (channels as f32);
    let mean_channel_delta = if channel_count > 0.0 {
        total_delta as f32 / channel_count
    } else {
        0.0
    };
    let bad_pixel_ratio = if total_pixels > 0 {
        bad_pixels as f32 / total_pixels as f32
    } else {
        0.0
    };
    let passed = max_delta <= config.max_channel_delta
        && mean_channel_delta <= config.max_mean_delta
        && bad_pixel_ratio <= config.max_bad_pixel_ratio;

    VisualDiffReport {
        expected_size,
        actual_size,
        total_pixels,
        bad_pixels,
        max_channel_delta: max_delta,
        mean_channel_delta,
        bad_pixel_ratio,
        dimension_mismatch: false,
        passed,
    }
}

/// Load two image files, convert them to RGBA, and compare them.
pub fn diff_image_paths(
    expected_path: impl AsRef<Path>,
    actual_path: impl AsRef<Path>,
    config: VisualDiffConfig,
) -> ImageResult<VisualDiffReport> {
    let expected = image::open(expected_path)?.to_rgba8();
    let actual = image::open(actual_path)?.to_rgba8();
    Ok(diff_rgba_images(&expected, &actual, config))
}

/// Build a red heatmap showing per-pixel differences.
pub fn diff_heatmap(expected: &RgbaImage, actual: &RgbaImage) -> RgbaImage {
    let width = expected.width().min(actual.width());
    let height = expected.height().min(actual.height());
    let mut heatmap = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let e = expected.get_pixel(x, y);
            let a = actual.get_pixel(x, y);
            let delta =
                e.0.iter()
                    .zip(a.0.iter())
                    .map(|(left, right)| left.abs_diff(*right))
                    .max()
                    .unwrap_or(0);
            heatmap.put_pixel(x, y, Rgba([delta, 0, 0, 255]));
        }
    }

    heatmap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_pass() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255]));
        let report = diff_rgba_images(&img, &img, VisualDiffConfig::default());
        assert!(report.passed, "{}", report.summary());
        assert_eq!(report.bad_pixels, 0);
    }

    #[test]
    fn channel_delta_beyond_tolerance_fails() {
        let expected = RgbaImage::from_pixel(1, 1, Rgba([10, 20, 30, 255]));
        let actual = RgbaImage::from_pixel(1, 1, Rgba([20, 20, 30, 255]));
        let report = diff_rgba_images(&expected, &actual, VisualDiffConfig::default());
        assert!(!report.passed);
        assert_eq!(report.max_channel_delta, 10);
        assert_eq!(report.bad_pixels, 1);
    }

    #[test]
    fn dimensions_must_match() {
        let expected = RgbaImage::new(1, 1);
        let actual = RgbaImage::new(2, 1);
        let report = diff_rgba_images(&expected, &actual, VisualDiffConfig::default());
        assert!(!report.passed);
        assert!(report.dimension_mismatch);
    }
}
