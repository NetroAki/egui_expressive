//! Declarative layout DSL for Figma-style Auto Layout in egui.
//!
//! # use egui_expressive::*;
//!
//! This module provides a set of macros that reduce the verbosity of egui layout code
//! to match Figma's Auto Layout readability.
//!
//! ## Basic Usage
//!
//! ```rust,ignore
//! # use egui_expressive::*;
//! # use egui::Color32;
//! fn example(ui: &mut egui::Ui) {
//!     // Vertical stack with 8px gap
//!     vstack!(ui, gap: 8.0, {
//!         ui.label("Item 1");
//!         ui.label("Item 2");
//!     });
//!
//!     // Horizontal stack with background
//!     hstack!(ui, gap: 8.0, padding: 12.0, bg: Color32::from_rgb(30, 30, 30), rounding: 4.0, {
//!         ui.label("Hello");
//!         ui.button("World");
//!     });
//! }
//! ```

use egui::{Color32, Frame, Margin, Stroke, Vec2};

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Apply Figma-style Auto Layout settings to a Ui.
///
/// This sets the item spacing and optionally configures padding via the ui's
/// layout system.
///
/// # Arguments
/// * `ui` - The Ui to configure
/// * `gap` - Item spacing in pixels
/// * `_padding` - Inner padding (reserved for future use, currently unused)
///
/// # Example
///
/// ```rust,no_run
/// # use egui_expressive::auto_layout;
/// fn example(ui: &mut egui::Ui) {
///     auto_layout(ui, 8.0, 12.0);
/// }
/// ```
pub fn auto_layout(ui: &mut egui::Ui, gap: f32, _padding: f32) {
    ui.spacing_mut().item_spacing = Vec2::splat(gap);
    let _ = _padding; // Reserved for future use
}

/// Create a Frame with design-token-friendly parameters.
///
/// # Arguments
/// * `bg` - Background fill color
/// * `rounding` - Corner rounding radius
/// * `padding` - Inner padding (uniform on all sides)
/// * `stroke` - Optional border stroke
///
/// # Example
///
/// ```rust,no_run
/// # use egui_expressive::styled_frame;
/// # use egui::Color32;
/// fn example() {
///     let frame = styled_frame(Color32::from_rgb(30, 30, 30), 4.0, 12.0, None);
/// }
/// ```
pub fn styled_frame(bg: Color32, rounding: f32, padding: f32, stroke: Option<Stroke>) -> Frame {
    let padding_i8 = padding.round() as i8;
    let mut frame = Frame::NONE
        .inner_margin(Margin::same(padding_i8))
        .fill(bg)
        .corner_radius(rounding);

    if let Some(s) = stroke {
        frame = frame.stroke(s);
    }

    frame
}

/// Horizontal rule (divider line).
///
/// # Arguments
/// * `ui` - The Ui to paint onto
/// * `color` - Line color
/// * `thickness` - Line thickness in pixels
///
/// # Example
///
/// ```rust,no_run
/// # use egui_expressive::hrule;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     hrule(ui, Color32::from_rgb(60, 60, 60), 1.0);
/// }
/// ```
pub fn hrule(ui: &mut egui::Ui, color: Color32, thickness: f32) {
    let available = ui.available_size();
    let mut clip = ui.clip_rect();
    clip.set_height(available.y);
    let y_center = clip.center().y;
    ui.painter()
        .hline(clip.x_range(), y_center, Stroke::new(thickness, color));
}

/// Vertical rule.
///
/// # Arguments
/// * `ui` - The Ui to paint onto
/// * `color` - Line color
/// * `thickness` - Line thickness in pixels
///
/// # Example
///
/// ```rust,no_run
/// # use egui_expressive::vrule;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     vrule(ui, Color32::from_rgb(60, 60, 60), 1.0);
/// }
/// ```
pub fn vrule(ui: &mut egui::Ui, color: Color32, thickness: f32) {
    let available = ui.available_size();
    let mut clip = ui.clip_rect();
    clip.set_width(available.x);
    let x_center = clip.center().x;
    ui.painter()
        .vline(x_center, clip.y_range(), Stroke::new(thickness, color));
}

// ---------------------------------------------------------------------------
// vstack! Macro
// ---------------------------------------------------------------------------

/// Vertical stack layout (Figma-style).
///
/// Lays out children vertically with consistent spacing.
///
/// # Parameters (all optional):
/// - `gap: f32` — Item spacing (defaults to 0)
/// - `padding: f32` — Uniform padding (or `[h, v]` for horizontal/vertical)
/// - `bg: Color32` — Background fill
/// - `rounding: f32` — Corner rounding
/// - `width: f32` — Fixed width (uses ui.set_width)
/// - `height: f32` — Fixed height
/// - `align: Align` — Horizontal alignment (Left, Center, Right)
///
/// # Variants
///
/// Basic usage:
/// ```rust,ignore
/// # use egui_expressive::vstack;
/// fn example(ui: &mut egui::Ui) {
///     vstack!(ui, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap:
/// ```rust,ignore
/// # use egui_expressive::vstack;
/// fn example(ui: &mut egui::Ui) {
///     vstack!(ui, gap: 8.0, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap and padding:
/// ```rust,ignore
/// # use egui_expressive::vstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     vstack!(ui, gap: 8.0, padding: 12.0, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap, padding, and background:
/// ```rust,ignore
/// # use egui_expressive::vstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     vstack!(ui, gap: 8.0, padding: 12.0, bg: Color32::from_rgb(30, 30, 30), {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap, padding, bg, and rounding:
/// ```rust,ignore
/// # use egui_expressive::vstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     vstack!(ui, gap: 8.0, padding: 12.0, bg: Color32::from_rgb(30, 30, 30), rounding: 4.0, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
#[macro_export]
macro_rules! vstack {
    // vstack!(ui, { body })
    ($ui:expr, { $($body:tt)* }) => {
        $crate::vstack!($ui, gap: 0.0, { $($body)* })
    };

    // vstack!(ui, gap: X, { body })
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {
        $crate::vstack!($ui, gap: $gap, padding: 0.0, { $($body)* })
    };

    // vstack!(ui, gap: X, padding: Y, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, );
    };

    // vstack!(ui, gap: X, padding: [H, V], { body })
    ($ui:expr, gap: $gap:expr, padding: [$($padding:expr),+], { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, ($($padding),+), { $($body)* }, );
    };

    // vstack!(ui, gap: X, padding: Y, bg: Z, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg);
    };

    // vstack!(ui, gap: X, padding: Y, bg: Z, rounding: R, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, rounding: $rounding:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg, rounding: $rounding);
    };

    // vstack!(ui, gap: X, padding: Y, bg: Z, rounding: R, width: W, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, rounding: $rounding:expr, width: $width:expr, { $($body:tt)* }) => {
        $crate::vstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg, rounding: $rounding, width: $width);
    };
}

// Helper macro for vstack implementation
#[macro_export]
macro_rules! vstack_impl {
    // No bg, no rounding
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* },) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;

        let __resp = $ui.vertical(|__ui| { $($body)* });

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};

    // With bg, no rounding
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;

        let __frame = $crate::layout::styled_frame($bg, 0.0, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.vertical(|__ui| { $($body)* }));

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};

    // With bg and rounding
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr, rounding: $rounding:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;

        let __frame = $crate::layout::styled_frame($bg, $rounding, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.vertical(|__ui| { $($body)* }));

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};

    // With bg, rounding, and width
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr, rounding: $rounding:expr, width: $width:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.y = $gap;
        $ui.set_width($width);

        let __frame = $crate::layout::styled_frame($bg, $rounding, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.vertical(|__ui| { $($body)* }));

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
}

// ---------------------------------------------------------------------------
// hstack! Macro
// ---------------------------------------------------------------------------

/// Horizontal stack layout (Figma-style).
///
/// Lays out children horizontally with consistent spacing.
///
/// # Parameters (all optional):
/// - `gap: f32` — Item spacing (defaults to 0)
/// - `padding: f32` — Uniform padding (or `[h, v]` for horizontal/vertical)
/// - `bg: Color32` — Background fill
/// - `rounding: f32` — Corner rounding
/// - `align: Align` — Vertical alignment (Top, Center, Bottom)
///
/// # Variants
///
/// Basic usage:
/// ```rust,ignore
/// # use egui_expressive::hstack;
/// fn example(ui: &mut egui::Ui) {
///     hstack!(ui, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap:
/// ```rust,ignore
/// # use egui_expressive::hstack;
/// fn example(ui: &mut egui::Ui) {
///     hstack!(ui, gap: 8.0, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap and padding:
/// ```rust,ignore
/// # use egui_expressive::hstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     hstack!(ui, gap: 8.0, padding: 12.0, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap, padding, and background:
/// ```rust,ignore
/// # use egui_expressive::hstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     hstack!(ui, gap: 8.0, padding: 12.0, bg: Color32::from_rgb(30, 30, 30), {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
///
/// With gap, padding, bg, and rounding:
/// ```rust,ignore
/// # use egui_expressive::hstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     hstack!(ui, gap: 8.0, padding: 12.0, bg: Color32::from_rgb(30, 30, 30), rounding: 4.0, {
///         ui.label("Item 1");
///         ui.label("Item 2");
///     });
/// }
/// ```
#[macro_export]
macro_rules! hstack {
    // hstack!(ui, { body })
    ($ui:expr, { $($body:tt)* }) => {
        $crate::hstack!($ui, gap: 0.0, { $($body)* })
    };

    // hstack!(ui, gap: X, { body })
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {
        $crate::hstack!($ui, gap: $gap, padding: 0.0, { $($body)* })
    };

    // hstack!(ui, gap: X, padding: Y, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, $padding, { $($body)* }, );
    };

    // hstack!(ui, gap: X, padding: [H, V], { body })
    ($ui:expr, gap: $gap:expr, padding: [$($padding:expr),+], { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, ($($padding),+), { $($body)* }, );
    };

    // hstack!(ui, gap: X, padding: Y, bg: Z, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg);
    };

    // hstack!(ui, gap: X, padding: Y, bg: Z, rounding: R, { body })
    ($ui:expr, gap: $gap:expr, padding: $padding:expr, bg: $bg:expr, rounding: $rounding:expr, { $($body:tt)* }) => {
        $crate::hstack_impl!($ui, $gap, $padding, { $($body)* }, bg: $bg, rounding: $rounding);
    };
}

// Helper macro for hstack implementation
#[macro_export]
macro_rules! hstack_impl {
    // No bg, no rounding
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* },) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.x = $gap;

        let __resp = $ui.horizontal(|__ui| { $($body)* });

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};

    // With bg, no rounding
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.x = $gap;

        let __frame = $crate::layout::styled_frame($bg, 0.0, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.horizontal(|__ui| { $($body)* }));

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};

    // With bg and rounding
    ($ui:expr, $gap:expr, $padding:expr, { $($body:tt)* }, bg: $bg:expr, rounding: $rounding:expr) => {{
        let __prev_spacing = $ui.spacing().item_spacing;
        $ui.spacing_mut().item_spacing.x = $gap;

        let __frame = $crate::layout::styled_frame($bg, $rounding, $padding, None);
        let __resp = __frame.show($ui, |__ui| __ui.horizontal(|__ui| { $($body)* }));

        $ui.spacing_mut().item_spacing = __prev_spacing;
        __resp
    }};
}

// ---------------------------------------------------------------------------
// zstack! Macro
// ---------------------------------------------------------------------------

/// Z-stack: children are painted at the same position (overlapping).
///
/// Uses `ui.allocate_ui` to create a fixed-size area where all children
/// are positioned absolutely at the same origin, allowing for overlapping
/// layers.
///
/// # Parameters:
/// - `size: Vec2` — Required, the size of the stack
/// - `bg: Color32` — Optional background fill
/// - `rounding: f32` — Optional corner rounding
///
/// # Example
///
/// ```rust,ignore
/// # use egui_expressive::zstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     zstack!(ui, size: Vec2::new(100.0, 100.0), {
///         ui.label("Layer 1");
///         ui.label("Layer 2");
///     });
/// }
/// ```
///
/// With background:
/// ```rust,ignore
/// # use egui_expressive::zstack;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     zstack!(ui, size: Vec2::new(100.0, 100.0), bg: Color32::from_rgb(30, 30, 30), {
///         ui.label("Content");
///     });
/// }
/// ```
#[macro_export]
macro_rules! zstack {
    // zstack!(ui, size: S, { body })
    ($ui:expr, size: $size:expr, { $($body:tt)* }) => {{
        let __size = $size;
        let (__rect, __resp) = $ui.allocate_ui(__size, |__ui| { $($body)* });
        __resp
    }};

    // zstack!(ui, size: S, bg: C, { body })
    ($ui:expr, size: $size:expr, bg: $bg:expr, { $($body:tt)* }) => {{
        let __size = $size;
        let __bg = $bg;

        $ui.painter()
            .rect_filled($ui.available_rect_before_wrap(), 0.0, __bg);

        let (__rect, __resp) = $ui.allocate_ui(__size, |__ui| { $($body)* });
        __resp
    }};

    // zstack!(ui, size: S, bg: C, rounding: R, { body })
    ($ui:expr, size: $size:expr, bg: $bg:expr, rounding: $rounding:expr, { $($body:tt)* }) => {{
        let __size = $size;
        let __bg = $bg;
        let __rounding = $rounding;

        let __available = $ui.available_rect_before_wrap();
        $ui.painter()
            .rounded_rect_filled(__available, __rounding, __bg);

        let (__rect, __resp) = $ui.allocate_ui(__size, |__ui| { $($body)* });
        __resp
    }};
}

// ---------------------------------------------------------------------------
// spacer! Macro
// ---------------------------------------------------------------------------

/// Flexible spacer that fills available space.
///
/// # Variants
///
/// Fills all remaining space:
/// ```rust,no_run
/// # use egui_expressive::spacer;
/// fn example(ui: &mut egui::Ui) {
///     spacer!(ui);
/// }
/// ```
///
/// Fixed-size gap:
/// ```rust,no_run
/// # use egui_expressive::spacer;
/// fn example(ui: &mut egui::Ui) {
///     spacer!(ui, 16.0);
/// }
/// ```
#[macro_export]
macro_rules! spacer {
    // spacer!(ui) — fills remaining space
    ($ui:expr) => {{
        let __size = $ui.available_size();
        $ui.allocate_space(__size);
    }};

    // spacer!(ui, X) — fixed size
    ($ui:expr, $size:expr) => {{
        $ui.allocate_space(egui::Vec2::splat($size));
    }};
}

// ---------------------------------------------------------------------------
// divider! Macro
// ---------------------------------------------------------------------------

/// Horizontal or vertical divider line.
///
/// # Variants
///
/// Horizontal divider (default):
/// ```rust,no_run
/// # use egui_expressive::divider;
/// fn example(ui: &mut egui::Ui) {
///     divider!(ui);
/// }
/// ```
///
/// Vertical divider:
/// ```rust,no_run
/// # use egui_expressive::divider;
/// fn example(ui: &mut egui::Ui) {
///     divider!(ui, vertical);
/// }
/// ```
///
/// Custom color:
/// ```rust,no_run
/// # use egui_expressive::divider;
/// # use egui::Color32;
/// fn example(ui: &mut egui::Ui) {
///     divider!(ui, Color32::RED);
/// }
/// ```
///
/// Vertical with custom thickness:
/// ```rust,no_run
/// # use egui_expressive::divider;
/// fn example(ui: &mut egui::Ui) {
///     divider!(ui, vertical, 2.0);
/// }
/// ```
#[macro_export]
macro_rules! divider {
    // divider!(ui) — horizontal, default color
    ($ui:expr) => {{
        $ui.separator();
    }};

    // divider!(ui, vertical) — vertical, default color
    ($ui:expr, vertical) => {{
        $crate::layout::vrule($ui, egui::Color32::from_rgb(60, 60, 60), 1.0);
    }};

    // divider!(ui, vertical, THICKNESS) — vertical with thickness
    ($ui:expr, vertical, $thickness:expr) => {{
        $crate::layout::vrule($ui, egui::Color32::from_rgb(60, 60, 60), $thickness);
    }};

    // divider!(ui, COLOR) — horizontal with custom color
    ($ui:expr, $color:expr) => {{
        $crate::layout::hrule($ui, $color, 1.0);
    }};

    // divider!(ui, vertical, COLOR) — vertical with custom color
    ($ui:expr, vertical, $color:expr) => {{
        $crate::layout::vrule($ui, $color, 1.0);
    }};

    // divider!(ui, vertical, COLOR, THICKNESS) — vertical with color and thickness
    ($ui:expr, vertical, $color:expr, $thickness:expr) => {{
        $crate::layout::vrule($ui, $color, $thickness);
    }};

    // divider!(ui, COLOR, THICKNESS) — horizontal with color and thickness
    ($ui:expr, $color:expr, $thickness:expr) => {{
        $crate::layout::hrule($ui, $color, $thickness);
    }};
}
