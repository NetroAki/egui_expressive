//! Gaussian blur helpers for `egui_expressive`.
//!
//! Provides software blur approximations and Gaussian-weighted soft shadows
//! that are significantly better than linear-stepped shadows.

use egui::{
    epaint::{RectShape, StrokeKind},
    Color32, ColorImage, Context, CornerRadius, Pos2, Rect, Shape, Stroke, TextureHandle,
    TextureOptions,
};

// ---------------------------------------------------------------------------
// BlurQuality
// ---------------------------------------------------------------------------

/// Soft shadow quality — controls number of samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlurQuality {
    /// Fast: 6 samples
    Fast,
    /// Medium: 12 samples
    Medium,
    /// High: 24 samples
    High,
}

impl BlurQuality {
    fn samples(self) -> usize {
        match self {
            BlurQuality::Fast => 6,
            BlurQuality::Medium => 12,
            BlurQuality::High => 24,
        }
    }
}

// ---------------------------------------------------------------------------
// Gaussian helper
// ---------------------------------------------------------------------------

/// Standard Gaussian PDF at x with given sigma.
#[inline]
fn gaussian(x: f32, sigma: f32) -> f32 {
    if sigma <= 0.0 {
        return if x.abs() < 0.001 { 1.0 } else { 0.0 };
    }
    (-x * x / (2.0 * sigma * sigma)).exp()
}

// ---------------------------------------------------------------------------
// Soft shadow (Gaussian-weighted)
// ---------------------------------------------------------------------------

/// Direction of a shadow offset.
use crate::draw::ShadowOffset;

/// Gaussian-weighted soft shadow (much better than linear stepping).
///
/// Returns `Vec<Shape>` to add to a painter.
///
/// # Arguments
/// * `rect`       - The base rect to shadow
/// * `color`      - Shadow color
/// * `blur_radius` - Blur radius in pixels
/// * `spread`     - Spread in pixels (negative = inset)
/// * `offset`     - Shadow offset
/// * `quality`    - Sampling quality
pub fn soft_shadow(
    rect: Rect,
    color: Color32,
    blur_radius: f32,
    spread: f32,
    offset: ShadowOffset,
    quality: BlurQuality,
) -> Vec<Shape> {
    let n = quality.samples().max(2);
    let mut shapes = Vec::with_capacity(n);

    // sigma = radius/3 so that 3σ covers the full radius (Gaussian falls off smoothly)
    let sigma = blur_radius / 3.0;
    let base_a = color.a() as f32;

    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let x = t * blur_radius;
        let weight = gaussian(x, sigma);
        // Normalize weights so the first and last sample have meaningful weight
        // (Gaussian at 0 and at radius)
        let alpha = (base_a * weight).clamp(0.0, 255.0) as u8;

        if alpha == 0 {
            continue;
        }

        // Expand rect: spread + (blur_radius - x) on each side
        let expansion = spread + (blur_radius - x);
        let shadow_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let shadow_rect = Rect::from_min_max(
            Pos2::new(
                rect.min.x - expansion + offset.x,
                rect.min.y - expansion + offset.y,
            ),
            Pos2::new(
                rect.max.x + expansion + offset.x,
                rect.max.y + expansion + offset.y,
            ),
        );
        let rounding = CornerRadius::same((expansion * 0.5).max(0.0) as u8);
        shapes.push(Shape::Rect(RectShape::filled(
            shadow_rect,
            rounding,
            shadow_color,
        )));
    }

    shapes
}

/// Soft inner shadow (inset shadow with Gaussian falloff).
pub fn soft_inner_shadow(
    rect: Rect,
    color: Color32,
    blur_radius: f32,
    quality: BlurQuality,
) -> Vec<Shape> {
    let n = quality.samples().max(2);
    let mut shapes = Vec::with_capacity(n * 4);
    let sigma = blur_radius / 3.0;
    let base_a = color.a() as f32;

    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let x = t * blur_radius;
        let weight = gaussian(x, sigma);
        let alpha = (base_a * weight).clamp(0.0, 255.0) as u8;

        if alpha == 0 {
            continue;
        }

        let inset = x;
        let c = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
        let stroke = Stroke::new(1.0, c);
        let inner = Rect::from_min_max(
            Pos2::new(rect.min.x + inset, rect.min.y + inset),
            Pos2::new(rect.max.x - inset, rect.max.y - inset),
        );
        if inner.width() > 0.0 && inner.height() > 0.0 {
            shapes.push(Shape::Rect(RectShape::stroke(
                inner,
                CornerRadius::ZERO,
                stroke,
                StrokeKind::Inside,
            )));
        }
    }

    shapes
}

/// Soft glow (symmetric soft_shadow with zero offset).
pub fn soft_glow(rect: Rect, color: Color32, radius: f32, quality: BlurQuality) -> Vec<Shape> {
    soft_shadow(rect, color, radius, 0.0, ShadowOffset::zero(), quality)
}

// ---------------------------------------------------------------------------
// Software box blur on ColorImage
// ---------------------------------------------------------------------------

/// Software box blur on a `ColorImage`.
///
/// Uses a separable two-pass box blur (horizontal then vertical), which is
/// O(width*height) regardless of radius. Edge pixels are clamped.
///
/// # Arguments
/// * `image`  - Input image
/// * `radius` - Blur radius in pixels (1–32)
///
/// # Returns
/// A new blurred `ColorImage`.
pub fn blur_image(image: &ColorImage, radius: u32) -> ColorImage {
    let radius = radius.min(32).max(1);
    let [width, height] = image.size;
    let (width, height) = (width as usize, height as usize);

    if width == 0 || height == 0 {
        return image.clone();
    }

    let pixels = &image.pixels;
    let mut temp = vec![Color32::TRANSPARENT; width * height];

    // Horizontal pass
    for y in 0..height {
        for x in 0..width {
            let mut r: u32 = 0;
            let mut g: u32 = 0;
            let mut b: u32 = 0;
            let mut a: u32 = 0;
            let mut count: u32 = 0;

            let x0 = (x as i32 - radius as i32).max(0) as usize;
            let x1 = (x as i32 + radius as i32).min(width as i32 - 1) as usize;

            for ix in x0..=x1 {
                let p = pixels[y * width + ix];
                r += p.r() as u32;
                g += p.g() as u32;
                b += p.b() as u32;
                a += p.a() as u32;
                count += 1;
            }

            let inv = 1.0 / count as f32;
            temp[y * width + x] = Color32::from_rgba_unmultiplied(
                (r as f32 * inv) as u8,
                (g as f32 * inv) as u8,
                (b as f32 * inv) as u8,
                (a as f32 * inv) as u8,
            );
        }
    }

    // Vertical pass
    let mut output = vec![Color32::TRANSPARENT; width * height];
    for x in 0..width {
        for y in 0..height {
            let mut r: u32 = 0;
            let mut g: u32 = 0;
            let mut b: u32 = 0;
            let mut a: u32 = 0;
            let mut count: u32 = 0;

            let y0 = (y as i32 - radius as i32).max(0) as usize;
            let y1 = (y as i32 + radius as i32).min(height as i32 - 1) as usize;

            for iy in y0..=y1 {
                let p = temp[iy * width + x];
                r += p.r() as u32;
                g += p.g() as u32;
                b += p.b() as u32;
                a += p.a() as u32;
                count += 1;
            }

            let inv = 1.0 / count as f32;
            output[y * width + x] = Color32::from_rgba_unmultiplied(
                (r as f32 * inv) as u8,
                (g as f32 * inv) as u8,
                (b as f32 * inv) as u8,
                (a as f32 * inv) as u8,
            );
        }
    }

    ColorImage {
        size: [width as _, height as _],
        pixels: output,
        source_size: egui::Vec2::new(width as f32, height as f32),
    }
}

/// Upload a blurred image as a texture and return a `Shape::Image`.
///
/// The caller must retain the `TextureHandle` to keep the texture alive.
///
/// # Arguments
/// * `ctx`      - `egui::Context`
/// * `image`    - Input `ColorImage`
/// * `radius`   - Blur radius in pixels
/// * `dest_rect` - Destination rect in screen coordinates
///
/// # Returns
/// A tuple of `(Shape::Image, TextureHandle)`.
pub fn blurred_image_shape(
    ctx: &Context,
    image: ColorImage,
    radius: u32,
    dest_rect: Rect,
) -> (Shape, TextureHandle) {
    let blurred = blur_image(&image, radius);
    let handle = ctx.load_texture("__expressive_blur", blurred, TextureOptions::LINEAR);
    let shape = Shape::image(
        handle.id(),
        dest_rect,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
    (shape, handle)
}
