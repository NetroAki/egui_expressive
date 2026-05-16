//! Render contract for `Tw`: frame conversion, flow layout, positioned layout.

use egui::{
    Align, Color32, Direction, FontId, Frame, Layout, Margin, Response, Sense, Stroke, Ui, Vec2,
};

use crate::layout::{GridLayout, PositionMode};
use crate::tailwind::{
    shadow::elevation_shadow,
    types::{
        Display, FlexDirection, FontWeight, GradientDirection, Items, Justify, Overflow,
        RadiusCorners, Size,
    },
    Tw,
};
use crate::{
    blur::{soft_shadow, BlurQuality},
    draw::{linear_gradient_rect, GradientDir, ShadowOffset},
};

impl Tw {
    pub fn to_frame(&self) -> Frame {
        let mut f = Frame::NONE;
        if let Some(bg) = self.bg {
            f = f.fill(apply_opacity(bg, self.opacity));
        }
        let radius = if self.radius_corners != RadiusCorners::default() {
            self.radius_corners.to_corner_radius()
        } else if self.border_radius > 0.0 {
            RadiusCorners::same(self.border_radius).to_corner_radius()
        } else {
            egui::CornerRadius::ZERO
        };
        if radius != egui::CornerRadius::ZERO {
            f = f.corner_radius(radius);
        }
        if self.border_width > 0.0 {
            f = f.stroke(Stroke::new(
                self.border_width,
                apply_opacity(
                    self.border_color.unwrap_or(Color32::from_gray(100)),
                    self.opacity,
                ),
            ));
        }
        let inner: Margin = self.padding.into();
        let outer: Margin = self.margin.into();
        f = f.inner_margin(inner).outer_margin(outer);
        if let Some(elevation) = self.elevation {
            let mut shadow = elevation_shadow(elevation);
            shadow.color = apply_opacity(shadow.color, self.opacity);
            f = f.shadow(shadow);
        }
        f
    }

    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let mut style = self;
        style.resolve_theme_tokens(ui.ctx());
        if style.position.is_positioned() {
            return style.show_positioned(ui, content);
        }
        if style.display == Display::Hidden {
            return ui.allocate_response(Vec2::ZERO, Sense::hover());
        }

        let min_w = style.min_width;
        let min_h = style.min_height;
        let max_w = style.max_width;
        let max_h = style.max_height;
        let width = style.width;
        let height = style.height;
        let display = style.display;
        let flex_direction = style.flex_direction;
        let justify = style.justify;
        let items = style.items;
        let overflow = style.overflow;
        let gap = style.gap;
        let space = style.space;
        let divide = style.divide;
        let grid = style.grid;
        let flex_wrap = style.flex_wrap;
        let cursor = style.cursor;
        let pointer_events = style.pointer_events;
        let border_edges = style.border_edges;
        let opacity = style.opacity;
        #[cfg(feature = "wgpu")]
        let exact_shadow_has_rounded_corners =
            style.border_radius > 0.0 || style.radius_corners != RadiusCorners::default();
        #[cfg(feature = "wgpu")]
        let exact_shadow_fill = style.bg.map(|color| apply_opacity(color, opacity));
        #[cfg(feature = "wgpu")]
        let exact_shadow_has_outer_border = style.border_width > 0.0;
        #[cfg(feature = "wgpu")]
        let exact_shadow_has_directional_border = !style.border_edges.is_empty();
        let border_color = style
            .border_color
            .map(|color| apply_opacity(color, opacity));
        let gradient = style
            .gradient
            .clone()
            .map(|gradient| gradient.with_opacity(opacity));
        let backdrop_blur = style.backdrop_blur;
        #[cfg(feature = "wgpu")]
        let backdrop_source = style.backdrop_source;
        let drop_shadow = style.drop_shadow.map(|shadow| shadow.with_opacity(opacity));
        let aspect_ratio = style.aspect_ratio;
        let ring = style.ring.map(|ring| ring.with_opacity(opacity));
        let selection = style.selection;
        let font_size = style.font_size;
        let font_weight = style.font_weight;
        let letter_spacing = style.letter_spacing;
        let id = style.id;
        #[cfg(feature = "wgpu")]
        let exact_drop_shadow_idx = drop_shadow
            .as_ref()
            .map(|_| ui.painter().add(egui::Shape::Noop));
        #[cfg(feature = "wgpu")]
        let exact_backdrop_blur_idx = backdrop_blur.and_then(|_| {
            matches!(
                backdrop_source,
                crate::tailwind::types::TwBackdropSource::AppProvidedSnapshot
            )
            .then(|| ui.painter().add(egui::Shape::Noop))
        });
        let frame = style.to_frame();
        let fg = style.fg.map(|color| apply_opacity(color, opacity));
        let mut prepared = frame.begin(ui);
        let gradient_idx = gradient
            .as_ref()
            .map(|_| prepared.content_ui.painter().add(egui::Shape::Noop));
        if gradient.is_some() {
            prepared.frame.fill = Color32::TRANSPARENT;
        }

        {
            let ui = &mut prepared.content_ui;
            if let Some(fg) = fg {
                ui.visuals_mut().override_text_color = Some(fg);
            }
            apply_typography(ui, font_size, font_weight, letter_spacing);
            if let Some(selection) = selection {
                ui.visuals_mut().selection.bg_fill = selection.bg;
                ui.visuals_mut().selection.stroke.color = selection.fg;
            }
            apply_size(ui, width, true);
            apply_size(ui, height, false);
            if let Some(ratio) = aspect_ratio {
                let width = ui.available_width();
                if width.is_finite() && width > 0.0 {
                    ui.set_min_height(width / ratio);
                }
            }
            if let Some(w) = min_w {
                ui.set_min_width(resolve_special_min(ui, w, true));
            }
            if let Some(h) = min_h {
                ui.set_min_height(resolve_special_min(ui, h, false));
            }
            if let Some(w) = max_w.and_then(|w| resolve_size(ui, w, true)) {
                ui.set_max_width(w);
            }
            if let Some(h) = max_h.and_then(|h| resolve_size(ui, h, false)) {
                ui.set_max_height(h);
            }
            if let Some(gap) = gap {
                ui.spacing_mut().item_spacing = gap;
            } else if let Some(space) = space {
                ui.spacing_mut().item_spacing = space;
            }
            if matches!(overflow, Overflow::Hidden | Overflow::Clip) {
                ui.set_clip_rect(ui.max_rect());
            }
            let show_content = |ui: &mut Ui| {
                if pointer_events {
                    content(ui);
                } else {
                    ui.add_enabled_ui(false, content);
                }
            };
            match display {
                Display::Flex if flex_wrap => {
                    ui.horizontal_wrapped(show_content);
                }
                Display::Flex => {
                    let layout = flex_layout(flex_direction, justify, items);
                    ui.with_layout(layout, show_content);
                }
                Display::Grid => {
                    let grid = grid.unwrap_or_else(|| GridLayout::columns(1));
                    let spacing = gap.unwrap_or_else(|| egui::vec2(grid.gap_x, grid.gap_y));
                    egui::Grid::new(id.unwrap_or_else(|| ui.id().with("tw_grid")))
                        .num_columns(grid.columns)
                        .spacing(spacing)
                        .show(ui, show_content);
                }
                _ => show_content(ui),
            }
        }

        if let (Some(idx), Some(gradient)) = (gradient_idx, gradient.as_ref()) {
            let fill_rect = prepared.frame.fill_rect(prepared.content_ui.min_rect());
            prepared
                .content_ui
                .painter()
                .with_clip_rect(fill_rect)
                .set(idx, gradient_shape(fill_rect, gradient));
        }

        #[cfg(feature = "wgpu")]
        let exact_drop_shadow_painted =
            if let (Some(idx), Some(shadow)) = (exact_drop_shadow_idx, drop_shadow) {
                let fill_rect = prepared.frame.fill_rect(prepared.content_ui.min_rect());
                let input = crate::tailwind::exact_effects::TwExactDropShadowInput {
                    rect: fill_rect,
                    fill: exact_shadow_fill,
                    shadow,
                    has_rounded_corners: exact_shadow_has_rounded_corners,
                    has_border: exact_shadow_has_outer_border,
                    has_ring: ring.is_some(),
                    has_gradient: gradient.is_some(),
                    has_directional_border: exact_shadow_has_directional_border,
                    has_divide: divide.is_some(),
                };
                if let Some(shape) = crate::tailwind::exact_effects::exact_drop_shadow_shape(
                    prepared.content_ui.ctx(),
                    input,
                ) {
                    ui.painter().set(idx, shape);
                    true
                } else {
                    false
                }
            } else {
                false
            };
        #[cfg(not(feature = "wgpu"))]
        let exact_drop_shadow_painted = false;

        #[cfg(feature = "wgpu")]
        let exact_backdrop_blur_painted =
            if let (Some(idx), Some(radius)) = (exact_backdrop_blur_idx, backdrop_blur) {
                let fill_rect = prepared.frame.fill_rect(prepared.content_ui.min_rect());
                let (shape, report) = crate::backdrop::app_provided_backdrop_blur_shape(
                    &prepared.content_ui,
                    fill_rect,
                    radius,
                );
                if report.is_exact() {
                    if let Some(shape) = shape {
                        ui.painter().set(idx, shape);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
        #[cfg(not(feature = "wgpu"))]
        let exact_backdrop_blur_painted = false;

        let mut response = prepared.end(ui);
        if let Some(shadow) = drop_shadow {
            if !exact_drop_shadow_painted {
                for shape in drop_shadow_shapes(response.rect, shadow) {
                    ui.painter().add(shape);
                }
            }
        }
        if let Some(radius) = backdrop_blur {
            if !exact_backdrop_blur_painted {
                let alpha = backdrop_overlay_alpha(radius, opacity);
                ui.painter()
                    .rect_filled(response.rect, 0.0, Color32::from_white_alpha(alpha));
            }
        }
        if !border_edges.is_empty() {
            border_edges.paint(
                ui,
                response.rect,
                border_color.unwrap_or(Color32::from_gray(100)),
            );
        }
        if let Some(cursor) = cursor {
            response = response.on_hover_cursor(cursor);
        }
        if let Some(ring) = ring {
            ui.painter().rect_stroke(
                response.rect.expand(ring.width),
                4.0,
                Stroke::new(ring.width, ring.color),
                egui::StrokeKind::Outside,
            );
        }
        if let Some(divide) = divide {
            paint_divide_hint(
                ui,
                response.rect,
                divide,
                border_color.unwrap_or(Color32::from_gray(80)),
            );
        }
        response
    }

    fn show_positioned(mut self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let screen = ui.ctx().input(|input| input.content_rect());
        let inset = self.position.inset;
        let base = match self.position.mode {
            PositionMode::Fixed | PositionMode::Absolute | PositionMode::Sticky => screen.min,
            PositionMode::Relative | PositionMode::Static => ui.cursor().min,
        };
        let pos = egui::pos2(
            base.x + inset.left.unwrap_or(0.0),
            base.y + inset.top.unwrap_or(0.0),
        ) + self.position.translate;
        let order = match self.position.z_index.unwrap_or(0) {
            z if z < 0 => egui::Order::Background,
            z if z >= 100 => egui::Order::Tooltip,
            _ => egui::Order::Foreground,
        };
        self.position.mode = PositionMode::Static;
        egui::Area::new(
            self.id
                .unwrap_or_else(|| egui::Id::new("egui_expressive_tw_area")),
        )
        .order(order)
        .fixed_pos(pos)
        .show(ui.ctx(), |ui| self.show(ui, content))
        .inner
    }

    fn resolve_theme_tokens(&mut self, ctx: &egui::Context) {
        let theme = crate::theme::Theme::load(ctx);
        if let Some(token) = self.bg_token {
            self.bg = Some(token.resolve(&theme.colors));
        }
        if let Some(token) = self.fg_token {
            self.fg = Some(token.resolve(&theme.colors));
        }
    }
}

fn apply_typography(
    ui: &mut Ui,
    font_size: Option<f32>,
    font_weight: FontWeight,
    letter_spacing: Option<f32>,
) {
    if let Some(size) = font_size {
        ui.style_mut().override_font_id = Some(FontId::proportional(size));
    }
    // egui 0.34 exposes weight and letter-spacing per `RichText`, not as a global `Ui` style.
    // Store the Tailwind text intent for `Tw::rich_text`/custom widgets, and apply size globally.
    ui.data_mut(|data| {
        data.insert_temp(egui::Id::new("egui_expressive.tw.font_weight"), font_weight);
        if let Some(spacing) = letter_spacing {
            data.insert_temp(egui::Id::new("egui_expressive.tw.letter_spacing"), spacing);
        }
    });
}

fn flex_layout(direction: FlexDirection, justify: Justify, items: Items) -> Layout {
    let cross = match items {
        Items::Start => Align::Min,
        Items::Center => Align::Center,
        Items::End => Align::Max,
        Items::Stretch => Align::Min,
    };
    let main = match direction {
        FlexDirection::Row => Direction::LeftToRight,
        FlexDirection::Column => Direction::TopDown,
    };
    let mut layout = Layout::from_main_dir_and_cross_align(main, cross);
    if matches!(justify, Justify::Center | Justify::End | Justify::Between) {
        layout = layout.with_main_justify(true);
    }
    if matches!(items, Items::Stretch) {
        layout = layout.with_cross_justify(true);
    }
    layout
}

fn gradient_shape(rect: egui::Rect, gradient: &crate::tailwind::types::TwGradient) -> egui::Shape {
    let direction = match gradient.direction {
        GradientDirection::ToRight => GradientDir::LeftToRight,
        GradientDirection::ToBottom => GradientDir::TopToBottom,
        GradientDirection::ToBottomRight => GradientDir::Angle(45.0),
        GradientDirection::Angle(deg) => GradientDir::Angle(deg),
    };
    linear_gradient_rect(rect, &gradient.stops, direction)
}

fn paint_divide_hint(ui: &Ui, rect: egui::Rect, divide: Vec2, color: Color32) {
    let (x, y) = divide_hint_centers(rect, divide);
    if let Some(x) = x {
        ui.painter()
            .vline(x, rect.y_range(), Stroke::new(divide.x, color));
    }
    if let Some(y) = y {
        ui.painter()
            .hline(rect.x_range(), y, Stroke::new(divide.y, color));
    }
}

fn divide_hint_centers(rect: egui::Rect, divide: Vec2) -> (Option<f32>, Option<f32>) {
    if divide.x > 0.0 {
        if divide.y > 0.0 {
            (Some(rect.center().x), Some(rect.center().y))
        } else {
            (Some(rect.center().x), None)
        }
    } else if divide.y > 0.0 {
        (None, Some(rect.center().y))
    } else {
        (None, None)
    }
}

fn backdrop_overlay_alpha(radius: f32, opacity: f32) -> u8 {
    let alpha = (radius / 32.0).clamp(0.05, 0.35) * opacity.clamp(0.0, 1.0);
    (alpha * 255.0).round() as u8
}

fn drop_shadow_shapes(
    rect: egui::Rect,
    shadow: crate::tailwind::types::TwDropShadow,
) -> Vec<egui::Shape> {
    let blur = shadow.blur as f32;
    if blur <= 0.0 {
        return vec![egui::Shape::rect_filled(
            rect.translate(shadow.offset),
            egui::CornerRadius::same(4),
            shadow.color,
        )];
    }

    soft_shadow(
        rect,
        shadow.color,
        blur,
        (blur * 0.08).min(2.0),
        ShadowOffset::new(shadow.offset.x, shadow.offset.y),
        BlurQuality::Medium,
    )
}

fn apply_opacity(color: Color32, opacity: f32) -> Color32 {
    let alpha = (color.a() as f32 * opacity.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn apply_size(ui: &mut Ui, size: Size, horizontal: bool) {
    if let Some(value) = resolve_size(ui, size, horizontal) {
        if horizontal {
            ui.set_width(value);
        } else {
            ui.set_height(value);
        }
    }
}

fn resolve_size(ui: &Ui, size: Size, horizontal: bool) -> Option<f32> {
    let available = ui.available_size();
    let viewport = ui.ctx().input(|input| input.content_rect().size());
    resolve_size_against(size, available, viewport, horizontal)
}

fn resolve_size_against(
    size: Size,
    available: Vec2,
    viewport: Vec2,
    horizontal: bool,
) -> Option<f32> {
    match size {
        Size::Full => Some(if horizontal { available.x } else { available.y }),
        Size::Px(value) => Some(value),
        Size::Percent(percent) => {
            Some((if horizontal { available.x } else { available.y }) * percent / 100.0)
        }
        Size::ViewportWidth(percent) => Some(viewport.x * percent / 100.0),
        Size::ViewportHeight(percent) => Some(viewport.y * percent / 100.0),
        Size::Auto => None,
    }
}

fn resolve_special_min(ui: &Ui, value: f32, horizontal: bool) -> f32 {
    if value.is_infinite() {
        ui.ctx().input(|input| {
            let rect = input.content_rect();
            if horizontal {
                rect.width()
            } else {
                rect.height()
            }
        })
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "wgpu")]
    use crate::platform::{
        install_backdrop_snapshot_provider, BackdropCaptureError, BackdropCaptureRequest,
        BackdropSnapshot, BackdropSnapshotProvider,
    };
    use crate::tailwind::types::{Display, TwBackdropSource};
    use crate::theme::Elevation;
    #[cfg(feature = "wgpu")]
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[cfg(feature = "wgpu")]
    struct TestBackdropProvider {
        calls: Arc<AtomicUsize>,
    }

    #[cfg(feature = "wgpu")]
    impl BackdropSnapshotProvider for TestBackdropProvider {
        fn capture_backdrop_snapshot(
            &self,
            request: &BackdropCaptureRequest,
        ) -> Result<BackdropSnapshot, BackdropCaptureError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let mut pixels = vec![0; request.expected_len()?];
            for (idx, pixel) in pixels.chunks_exact_mut(4).enumerate() {
                let x = (idx as u32 % request.requested_width) as u8;
                let y = (idx as u32 / request.requested_width) as u8;
                pixel.copy_from_slice(&[x.wrapping_mul(7), y.wrapping_mul(11), 160, 255]);
            }
            BackdropSnapshot::new(request.requested_width, request.requested_height, pixels)
        }
    }

    #[cfg(feature = "wgpu")]
    fn mark_context_ready(ctx: &egui::Context) {
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new("egui_expressive.gpu_effects.context_ready"),
                true,
            );
        });
    }

    #[cfg(feature = "wgpu")]
    fn install_test_backdrop_provider(ctx: &egui::Context) -> Arc<AtomicUsize> {
        let calls = Arc::new(AtomicUsize::new(0));
        install_backdrop_snapshot_provider(
            ctx,
            Arc::new(TestBackdropProvider {
                calls: calls.clone(),
            }),
        );
        calls
    }

    #[cfg(feature = "wgpu")]
    fn output_contains_callback(output: &egui::FullOutput) -> bool {
        output
            .shapes
            .iter()
            .any(|shape| matches!(shape.shape, egui::Shape::Callback(_)))
    }

    #[cfg(feature = "wgpu")]
    fn first_callback_index(output: &egui::FullOutput) -> usize {
        output
            .shapes
            .iter()
            .position(|shape| matches!(shape.shape, egui::Shape::Callback(_)))
            .expect("exact callback shape is present")
    }

    #[cfg(feature = "wgpu")]
    fn first_filled_rect_index(output: &egui::FullOutput, fill: Color32) -> usize {
        output
            .shapes
            .iter()
            .position(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.fill == fill))
            .unwrap_or_else(|| panic!("filled rect with {fill:?} is present"))
    }

    fn mesh_shape(shape: egui::Shape) -> egui::epaint::Mesh {
        match shape {
            egui::Shape::Mesh(mesh) => (*mesh).clone(),
            other => panic!("expected mesh shape, got {other:?}"),
        }
    }

    #[test]
    fn tw_margin_maps_to_outer_margin() {
        let frame = Tw::new().mx(8.0).mt(4.0).mb(12.0).to_frame();
        assert_eq!(frame.outer_margin.left, 8);
        assert_eq!(frame.outer_margin.right, 8);
        assert_eq!(frame.outer_margin.top, 4);
        assert_eq!(frame.outer_margin.bottom, 12);
    }

    #[test]
    fn tw_shadow_maps_elevation_to_frame_shadow() {
        let frame = Tw::new().shadow(Elevation::Level2).to_frame();
        assert!(frame.shadow.blur > 0);
        assert_ne!(frame.shadow.color, Color32::TRANSPARENT);
    }

    #[test]
    fn tw_opacity_affects_frame_colors() {
        let frame = Tw::new()
            .bg(Color32::from_rgba_unmultiplied(10, 20, 30, 200))
            .border_1()
            .border_color(Color32::from_rgba_unmultiplied(40, 50, 60, 180))
            .opacity(0.5)
            .to_frame();
        assert_eq!(frame.fill.a(), 100);
        assert_eq!(frame.stroke.color.a(), 90);
    }

    #[test]
    fn tw_grid_and_position_utilities_store_css_intent() {
        let style = Tw::new()
            .grid_cols(3)
            .grid_rows(2)
            .col_span(2)
            .absolute()
            .inset(8.0)
            .translate_x(-12.0)
            .z(20);
        assert_eq!(style.display, Display::Grid);
        assert_eq!(style.grid.unwrap().columns, 3);
        assert_eq!(style.grid.unwrap().rows, Some(2));
        assert_eq!(style.col_span, Some(2));
        assert!(style.position.is_positioned());
        assert_eq!(style.position.inset.top, Some(8.0));
        assert_eq!(style.position.translate.x, -12.0);
        assert_eq!(style.position.z_index, Some(20));
    }

    #[test]
    fn tw_utility_gaps_store_tailwind_values() {
        let style = Tw::new()
            .w_pct(75.0)
            .max_w_vw(92.0)
            .bg_alpha(Color32::BLACK, 0.55)
            .flex()
            .flex_wrap()
            .gap_x(8.0)
            .gap_y(12.0)
            .pointer_events_none()
            .cursor_pointer()
            .border_l(2.0);
        assert_eq!(style.width, Size::Percent(75.0));
        assert_eq!(style.max_width, Some(Size::ViewportWidth(92.0)));
        assert_eq!(style.display, Display::Flex);
        assert!(style.flex_wrap);
        assert!(!style.pointer_events);
        assert_eq!(style.gap, Some(Vec2::new(8.0, 12.0)));
        assert_eq!(style.border_edges.left.width, 2.0);
    }

    #[test]
    fn tw_stage12_size_resolution_is_egui_bounded_not_css_tree_layout() {
        let available = Vec2::new(320.0, 180.0);
        let viewport = Vec2::new(1280.0, 720.0);

        assert_eq!(
            resolve_size_against(Size::Percent(50.0), available, viewport, true),
            Some(160.0)
        );
        assert_eq!(
            resolve_size_against(Size::Percent(25.0), available, viewport, false),
            Some(45.0)
        );
        assert_eq!(
            resolve_size_against(Size::ViewportWidth(10.0), available, viewport, true),
            Some(128.0)
        );
        assert_eq!(
            resolve_size_against(Size::ViewportHeight(10.0), available, viewport, false),
            Some(72.0)
        );
        assert_eq!(
            resolve_size_against(Size::Full, available, viewport, true),
            Some(320.0)
        );
    }

    #[test]
    fn tw_stage12_divide_hint_uses_container_centerline() {
        let rect = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(90.0, 120.0));

        assert_eq!(
            divide_hint_centers(rect, Vec2::new(1.0, 0.0)),
            (Some(50.0), None)
        );
        assert_eq!(
            divide_hint_centers(rect, Vec2::new(0.0, 2.0)),
            (None, Some(70.0))
        );
        assert_eq!(
            divide_hint_centers(rect, Vec2::new(1.0, 2.0)),
            (Some(50.0), Some(70.0))
        );
    }

    #[test]
    fn tw_stage12_backdrop_overlay_alpha_remains_bounded_tint() {
        assert_eq!(backdrop_overlay_alpha(0.0, 1.0), 13);
        assert_eq!(backdrop_overlay_alpha(12.0, 0.5), 45);
        assert_eq!(backdrop_overlay_alpha(64.0, 1.0), 89);
    }

    #[test]
    fn tw_phase5_default_backdrop_blur_stays_bounded_overlay() {
        let style = Tw::new().backdrop_blur(24.0);
        assert_eq!(style.backdrop_blur, Some(24.0));
        assert_eq!(style.backdrop_source, TwBackdropSource::BoundedOverlay);
        assert_eq!(backdrop_overlay_alpha(24.0, 1.0), 89);

        let contract = include_str!("../../docs/ui-framework/tw-render-contract.md");
        assert!(contract.contains("default `backdrop_blur` row in the bounded corpus"));
        assert!(contract.contains("GpuSourceLayerEffectCallback"));
    }

    #[test]
    fn tw_r100_002_app_provided_backdrop_selector_stores_source_intent() {
        let style = Tw::new().backdrop_blur_app_provided(-4.0);

        assert_eq!(style.backdrop_blur, Some(0.0));
        assert_eq!(style.backdrop_source, TwBackdropSource::AppProvidedSnapshot);

        let bounded = style.backdrop_blur(12.0);
        assert_eq!(bounded.backdrop_blur, Some(12.0));
        assert_eq!(bounded.backdrop_source, TwBackdropSource::BoundedOverlay);
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn tw_app_provided_backdrop_blur_uses_exact_callback_when_ready() {
        let ctx = egui::Context::default();
        mark_context_ready(&ctx);
        let calls = install_test_backdrop_provider(&ctx);

        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                Tw::new()
                    .w(64.0)
                    .h(32.0)
                    .backdrop_blur_app_provided(4.0)
                    .show(ui, |_| {});
            });
        });

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(output_contains_callback(&output));
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn tw_app_provided_backdrop_callback_stays_behind_frame_and_content() {
        let ctx = egui::Context::default();
        mark_context_ready(&ctx);
        let calls = install_test_backdrop_provider(&ctx);
        let frame_fill = Color32::from_rgb(24, 64, 160);
        let child_fill = Color32::from_rgb(240, 200, 80);

        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                Tw::new()
                    .w(64.0)
                    .h(32.0)
                    .bg(frame_fill)
                    .backdrop_blur_app_provided(4.0)
                    .show(ui, |ui| {
                        let (_id, rect) = ui.allocate_space(egui::vec2(8.0, 8.0));
                        ui.painter().rect_filled(rect, 0.0, child_fill);
                    });
            });
        });

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let callback = first_callback_index(&output);
        let frame = first_filled_rect_index(&output, frame_fill);
        let child = first_filled_rect_index(&output, child_fill);
        assert!(callback < frame, "callback must paint behind frame fill");
        assert!(callback < child, "callback must paint behind child content");
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn tw_default_backdrop_blur_keeps_bounded_overlay_without_provider_call() {
        let ctx = egui::Context::default();
        mark_context_ready(&ctx);
        let calls = install_test_backdrop_provider(&ctx);

        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                Tw::new()
                    .w(64.0)
                    .h(32.0)
                    .backdrop_blur(4.0)
                    .show(ui, |_| {});
            });
        });

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(!output_contains_callback(&output));
    }

    #[cfg(feature = "wgpu")]
    #[test]
    fn tw_exact_drop_shadow_callback_stays_behind_frame_and_content() {
        let ctx = egui::Context::default();
        mark_context_ready(&ctx);
        let frame_fill = Color32::from_rgb(32, 96, 180);
        let child_fill = Color32::from_rgb(255, 210, 90);

        let output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show_inside(ctx, |ui| {
                Tw::new()
                    .w(64.0)
                    .h(32.0)
                    .bg(frame_fill)
                    .drop_shadow(egui::vec2(2.0, 4.0), 8, Color32::from_black_alpha(120))
                    .show(ui, |ui| {
                        let (_id, rect) = ui.allocate_space(egui::vec2(8.0, 8.0));
                        ui.painter().rect_filled(rect, 0.0, child_fill);
                    });
            });
        });

        let callback = first_callback_index(&output);
        let frame = first_filled_rect_index(&output, frame_fill);
        let child = first_filled_rect_index(&output, child_fill);
        assert!(callback < frame, "callback must paint behind frame fill");
        assert!(callback < child, "callback must paint behind child content");
    }

    #[test]
    fn tw_stage12_drop_shadow_uses_gaussian_soft_layers() {
        let rect = egui::Rect::from_min_max(egui::pos2(20.0, 30.0), egui::pos2(80.0, 70.0));
        let shadow = crate::tailwind::types::TwDropShadow {
            offset: Vec2::new(4.0, 6.0),
            blur: 12,
            color: Color32::from_black_alpha(144),
        };
        let shapes = drop_shadow_shapes(rect, shadow);

        assert!(
            shapes.len() >= 10,
            "medium gaussian shadow should use multiple layers"
        );
        let first_alpha = rect_fill_alpha(&shapes[0]);
        let last_alpha = rect_fill_alpha(shapes.last().expect("last shadow layer"));
        assert!(first_alpha > last_alpha);
    }

    #[test]
    fn tw_typography_and_effect_utilities_store_rendered_intent() {
        let style = Tw::new()
            .text_2xl()
            .font_bold()
            .tracking_wide()
            .flex_col()
            .items_stretch()
            .justify_between()
            .space_x(6.0)
            .divide_y(1.0)
            .bg_gradient_to_r(Color32::BLACK, Color32::WHITE)
            .backdrop_blur(12.0)
            .drop_shadow(Vec2::new(0.0, 4.0), 8, Color32::from_black_alpha(80))
            .aspect_ratio(16.0 / 9.0)
            .ring(2.0, Color32::LIGHT_BLUE)
            .transition(0.15)
            .selection(Color32::BLUE, Color32::WHITE);

        assert_eq!(style.font_size, Some(24.0));
        assert_eq!(style.font_weight, FontWeight::Bold);
        assert_eq!(style.letter_spacing, Some(0.5));
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.items, Items::Stretch);
        assert_eq!(style.justify, Justify::Between);
        assert_eq!(style.space, Some(Vec2::new(6.0, 0.0)));
        assert_eq!(style.divide, Some(Vec2::new(0.0, 1.0)));
        assert!(style.gradient.is_some());
        assert_eq!(style.backdrop_blur, Some(12.0));
        assert_eq!(style.backdrop_source, TwBackdropSource::BoundedOverlay);
        assert!(style.drop_shadow.is_some());
        assert_eq!(style.aspect_ratio, Some(16.0 / 9.0));
        assert!(style.ring.is_some());
        assert_eq!(style.transition.unwrap().duration_secs, 0.15);
        assert!(style.selection.is_some());
    }

    #[test]
    fn tw_font_weight_supports_full_tailwind_scale() {
        let style = Tw::new().font_semibold();
        assert_eq!(style.font_weight, FontWeight::SemiBold);

        let rich = Tw::new().font_weight(820).rich_text("weight");
        let debug = format!("{rich:?}");
        assert_eq!(
            Tw::new().font_weight(820).font_weight,
            FontWeight::ExtraBold
        );
        assert!(debug.contains("strong: true"));

        let light_debug = format!("{:?}", Tw::new().font_weight(120).rich_text("light"));
        assert_eq!(Tw::new().font_weight(120).font_weight, FontWeight::Thin);
        assert!(light_debug.contains("weak: true"));
    }

    #[test]
    fn tw_gradient_supports_angle_and_multi_stop_linear_meshes() {
        let style = Tw::new().bg_gradient_angle_stops(
            33.0,
            [
                (0.0, Color32::BLACK),
                (0.4, Color32::RED),
                (1.0, Color32::WHITE),
            ],
        );

        let gradient = style.gradient.as_ref().expect("gradient stored");
        assert_eq!(gradient.direction, GradientDirection::Angle(33.0));
        assert_eq!(gradient.stops.len(), 3);

        let mesh = mesh_shape(gradient_shape(
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 40.0)),
            gradient,
        ));
        assert_eq!(
            mesh.vertices.len(),
            6,
            "three stops should produce a quad strip"
        );
        assert_eq!(
            mesh.indices.len(),
            12,
            "three stops should produce two segments"
        );
    }

    #[test]
    fn tw_phase6_exact_rect_frame_subset_is_explicit() {
        let style = Tw::new()
            .p(12.0)
            .w(96.0)
            .h(48.0)
            .bg(Color32::from_rgb(24, 64, 160))
            .text_color(Color32::WHITE)
            .rounded_lg()
            .border_1()
            .ring(2.0, Color32::from_rgb(72, 132, 255))
            .bg_gradient_angle(
                35.0,
                Color32::from_rgb(12, 32, 96),
                Color32::from_rgb(96, 160, 255),
            );

        let frame = style.to_frame();
        assert_eq!(frame.inner_margin.left, 12);
        assert_eq!(frame.inner_margin.right, 12);
        assert_eq!(frame.fill, Color32::from_rgb(24, 64, 160));
        assert!(style.gradient.is_some());
        assert!(style.ring.is_some());

        let contract = include_str!("../../docs/ui-framework/tw-render-contract.md");
        assert!(contract.contains("Phase 6 exact rect-frame subset"));
        assert!(contract.contains("tailwind-supported-gradient-card"));
        assert!(contract.contains("layout/flex/grid parity remains bounded"));
    }

    fn rect_fill_alpha(shape: &egui::Shape) -> u8 {
        match shape {
            egui::Shape::Rect(rect) => rect.fill.a(),
            other => panic!("expected rect shape, got {other:?}"),
        }
    }

    #[test]
    fn tw_render_contract_names_supported_and_bounded_methods() {
        let contract = include_str!("../../docs/ui-framework/tw-render-contract.md");
        for method in [
            "opacity",
            "transition",
            "bg_gradient",
            "bg_gradient_angle",
            "bg_gradient_stops",
            "backdrop_blur",
            "drop_shadow",
            "divide",
            "ring",
            "filter",
            "font_weight",
            "pointer_events_none",
            "ResponsiveTw",
        ] {
            assert!(
                contract.contains(method),
                "missing {method} in render contract"
            );
        }
    }
}
