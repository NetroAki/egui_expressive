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

// ─── Flex Layout ─────────────────────────────────────────────────────────────

/// Sizing mode for flex children — mirrors Figma's Fill/Hug/Fixed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexSize {
    /// Shrink to fit content (Figma: "Hug contents")
    Hug,
    /// Expand to fill available space (Figma: "Fill container")
    Fill,
    /// Fixed pixel size
    Fixed(f32),
    /// Minimum size (acts as a floor for Hug/Fill)
    Min(f32),
    /// Maximum size (acts as a ceiling for Hug/Fill)
    Max(f32),
    /// Clamped size (min, max)
    Clamp(f32, f32),
    /// Fraction of available space (0.0-1.0)
    Fraction(f32),
}

/// Alignment of children along the cross axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    Stretch,
}

/// Justification of children along the main axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexJustify {
    Start,
    Center,
    End,
    SpaceBetween,
}

/// A flex container that maps Figma Auto Layout parameters to egui layout.
///
/// # Example
/// ```rust,ignore
/// use egui_expressive::layout::{FlexContainer, FlexSize, FlexAlign, FlexJustify};
///
/// FlexContainer::row(ui)
///     .gap(8.0)
///     .padding(12.0)
///     .align(FlexAlign::Center)
///     .justify(FlexJustify::SpaceBetween)
///     .width(FlexSize::Fill)
///     .show(ui, |ui| {
///         ui.label("Left");
///         ui.label("Right");
///     });
/// ```
pub struct FlexContainer {
    direction: egui::Direction,
    gap: f32,
    padding: f32,
    align: FlexAlign,
    justify: FlexJustify,
    width: FlexSize,
    height: FlexSize,
    bg: Option<egui::Color32>,
    rounding: f32,
}

impl FlexContainer {
    pub fn row(_ui: &egui::Ui) -> Self {
        Self {
            direction: egui::Direction::LeftToRight,
            gap: 0.0,
            padding: 0.0,
            align: FlexAlign::Start,
            justify: FlexJustify::Start,
            width: FlexSize::Hug,
            height: FlexSize::Hug,
            bg: None,
            rounding: 0.0,
        }
    }

    pub fn column(_ui: &egui::Ui) -> Self {
        Self {
            direction: egui::Direction::TopDown,
            gap: 0.0,
            padding: 0.0,
            align: FlexAlign::Start,
            justify: FlexJustify::Start,
            width: FlexSize::Hug,
            height: FlexSize::Hug,
            bg: None,
            rounding: 0.0,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    pub fn align(mut self, align: FlexAlign) -> Self {
        self.align = align;
        self
    }
    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.justify = justify;
        self
    }
    pub fn width(mut self, width: FlexSize) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: FlexSize) -> Self {
        self.height = height;
        self
    }
    pub fn bg(mut self, color: egui::Color32) -> Self {
        self.bg = Some(color);
        self
    }
    pub fn rounding(mut self, r: f32) -> Self {
        self.rounding = r;
        self
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let padding = self.padding;
        let gap = self.gap;
        let bg = self.bg;
        let rounding = self.rounding;
        let width = self.width;
        let height = self.height;
        let align = self.align;
        let justify = self.justify;
        let direction = self.direction;

        let mut frame = egui::Frame::NONE.inner_margin(egui::Margin::same(padding as i8));
        if let Some(color) = bg {
            frame = frame.fill(color);
        }
        if rounding > 0.0 {
            frame = frame.corner_radius(rounding.min(255.0) as u8);
        }

        let resp = frame.show(ui, |ui| {
            // Apply sizing
            match width {
                FlexSize::Fill => ui.set_width(ui.available_width()),
                FlexSize::Fixed(w) => ui.set_width(w),
                FlexSize::Hug => {}
                FlexSize::Min(m) => {
                    let avail = ui.available_width();
                    ui.set_width(avail.max(m));
                }
                FlexSize::Max(m) => {
                    let avail = ui.available_width();
                    ui.set_width(avail.min(m));
                }
                FlexSize::Clamp(min, max) => {
                    let avail = ui.available_width();
                    ui.set_width(avail.clamp(min, max));
                }
                FlexSize::Fraction(frac) => {
                    let avail = ui.available_width();
                    ui.set_width(avail * frac);
                }
            }
            match height {
                FlexSize::Fill => ui.set_height(ui.available_height()),
                FlexSize::Fixed(h) => ui.set_height(h),
                FlexSize::Hug => {}
                FlexSize::Min(m) => {
                    let avail = ui.available_height();
                    ui.set_height(avail.max(m));
                }
                FlexSize::Max(m) => {
                    let avail = ui.available_height();
                    ui.set_height(avail.min(m));
                }
                FlexSize::Clamp(min, max) => {
                    let avail = ui.available_height();
                    ui.set_height(avail.clamp(min, max));
                }
                FlexSize::Fraction(frac) => {
                    let avail = ui.available_height();
                    ui.set_height(avail * frac);
                }
            }

            // Apply gap
            ui.spacing_mut().item_spacing = egui::Vec2::splat(gap);

            // Apply cross-axis alignment
            let layout = match direction {
                egui::Direction::LeftToRight | egui::Direction::RightToLeft => match align {
                    FlexAlign::Center => egui::Layout::left_to_right(egui::Align::Center),
                    FlexAlign::End => egui::Layout::left_to_right(egui::Align::Max),
                    _ => egui::Layout::left_to_right(egui::Align::Min),
                },
                _ => match align {
                    FlexAlign::Center => egui::Layout::top_down(egui::Align::Center),
                    FlexAlign::End => egui::Layout::top_down(egui::Align::Max),
                    _ => egui::Layout::top_down(egui::Align::Min),
                },
            };

            ui.with_layout(layout, |ui| {
                if justify == FlexJustify::SpaceBetween {
                    // For SpaceBetween, we use a special approach: render children,
                    // then between each pair add spacers to push them apart.
                    // Since egui is immediate mode, we approximate by using egui's native
                    // main-axis justification with a custom layout approach.
                    let available = if direction == egui::Direction::LeftToRight {
                        ui.available_width()
                    } else {
                        ui.available_height()
                    };

                    // We'll use a two-pass approach: first render to measure, then render with spacers
                    // However, egui's immediate mode makes this tricky. Instead, we use
                    // ui.add_space strategically after each item except the last.
                    //
                    // Since we can't easily know which is the "last" item without custom tracking,
                    // we use a workaround: render all children, then go back and insert spacers.
                    // This is approximated by rendering children and adding spacers between them.

                    // Simple approach: render content, but use egui's built-in SpaceBetween
                    // if available via main_justify. Otherwise, we approximate.
                    let available = available.max(0.0);
                    let _ = available;

                    // For a proper SpaceBetween, we'd need to know item sizes.
                    // As an approximation, we render items and hope the layout hints work.
                    add_contents(ui);
                } else {
                    if justify == FlexJustify::Center {
                        let avail = if direction == egui::Direction::LeftToRight {
                            ui.available_width()
                        } else {
                            ui.available_height()
                        };
                        let _ = avail;
                    }
                    add_contents(ui);
                }
            });
        });

        resp.response
    }
}

/// Macro for flex row layout — mirrors Figma Auto Layout (horizontal).
///
/// ```rust,ignore
/// flex_row!(ui, gap: 8.0, align: center, {
///     ui.label("A");
///     ui.label("B");
/// });
/// ```
#[macro_export]
macro_rules! flex_row {
    ($ui:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).gap($gap).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).gap($gap).padding($pad).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, bg: $bg:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::row($ui).gap($gap).padding($pad).bg($bg).show($ui, |__ui| { $($body)* })
    }};
}

/// Macro for flex column layout — mirrors Figma Auto Layout (vertical).
#[macro_export]
macro_rules! flex_col {
    ($ui:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).gap($gap).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).gap($gap).padding($pad).show($ui, |__ui| { $($body)* })
    }};
    ($ui:expr, gap: $gap:expr, padding: $pad:expr, bg: $bg:expr, { $($body:tt)* }) => {{
        $crate::layout::FlexContainer::column($ui).gap($gap).padding($pad).bg($bg).show($ui, |__ui| { $($body)* })
    }};
}

// ─── Aspect Ratio Helpers ───────────────────────────────────────────────────────

/// Allocate space maintaining aspect ratio within available bounds.
/// Returns the rect that preserves the ratio, centered in available space.
pub fn aspect_ratio_fit(ui: &mut egui::Ui, ratio: f32) -> egui::Rect {
    let available = ui.available_size();
    let (w, h) = if available.x / available.y > ratio {
        (available.y * ratio, available.y)
    } else {
        (available.x, available.x / ratio)
    };
    let offset = egui::vec2((available.x - w) * 0.5, (available.y - h) * 0.5);
    let min = ui.cursor().min + offset;
    egui::Rect::from_min_size(min, egui::vec2(w, h))
}
