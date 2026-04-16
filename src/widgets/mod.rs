#![allow(dead_code)]

//! Reusable controls: Knob, Fader, Meter, StepGrid, and more.
//! DAW-specific widgets are also accessible via the `daw` feature module.

use crate::interaction::DragAxis;
use egui::{
    emath::Vec2 as EMathVec2,
    epaint::{PathShape, PathStroke},
    Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
};
use std::f32::consts::PI;
use std::ops::RangeInclusive;

// ---------------------------------------------------------------------------
// Orientation
// ---------------------------------------------------------------------------

/// Control orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

// ---------------------------------------------------------------------------
// KnobStyle & KnobSize
// ---------------------------------------------------------------------------

/// Visual style for a Knob widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KnobStyle {
    /// Arc track with filled value arc and indicator line (default).
    #[default]
    Default,
    /// Filled circle with indicator line, no arc track.
    Flat,
    /// Thin ring outline with indicator line.
    Ring,
    /// Tick marks around the perimeter.
    Notched,
}

/// Preset size for a Knob widget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum KnobSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

impl KnobSize {
    pub fn to_px(self) -> f32 {
        match self {
            KnobSize::Xs => 24.0,
            KnobSize::Sm => 32.0,
            KnobSize::Md => 48.0,
            KnobSize::Lg => 64.0,
        }
    }
}

// ---------------------------------------------------------------------------
// ContinuousControl — drag-to-value primitive
// ---------------------------------------------------------------------------

/// The core interaction primitive for any continuous-value control.
/// Handles drag, normalization, shift-for-fine, and double-click-reset.
/// Use this to build knobs, faders, sliders, or any custom control.
///
/// # Example
/// ```rust,ignore
/// let mut ctrl = ContinuousControl::new(&mut value, 0.0..=1.0);
/// let response = ui.allocate_rect(rect, Sense::drag());
/// let t = ctrl.handle(ui, &response); // returns normalized 0.0..=1.0
/// ```
pub struct ContinuousControl<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    /// Pixels of drag to traverse the full range. Default: 200.0
    pub sensitivity: f32,
    /// Multiplier applied when Shift is held. Default: 0.1
    pub fine_multiplier: f32,
    /// Value to reset to on double-click. None = no reset.
    pub default_value: Option<f64>,
    /// Which axis drives the value. Default: Y (drag up = increase).
    pub axis: DragAxis,
}

impl<'a> ContinuousControl<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            sensitivity: 200.0,
            fine_multiplier: 0.1,
            default_value: None,
            axis: DragAxis::Y,
        }
    }

    pub fn sensitivity(mut self, s: f32) -> Self {
        self.sensitivity = s;
        self
    }
    pub fn fine_multiplier(mut self, m: f32) -> Self {
        self.fine_multiplier = m;
        self
    }
    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }
    pub fn axis(mut self, a: DragAxis) -> Self {
        self.axis = a;
        self
    }

    /// Process interaction for the given response. Call after `ui.allocate_rect()`.
    /// Returns the normalized value (0.0..=1.0).
    pub fn handle(&mut self, ui: &Ui, response: &Response) -> f32 {
        let min = *self.range.start();
        let max = *self.range.end();
        let range_span = (max - min) as f32;

        if response.dragged() {
            let raw_delta = match self.axis {
                DragAxis::Y => -response.drag_delta().y, // up = increase
                DragAxis::X => response.drag_delta().x,
                DragAxis::Free => {
                    let d = response.drag_delta();
                    if d.x.abs() > d.y.abs() {
                        d.x
                    } else {
                        -d.y
                    }
                }
            };
            let multiplier = if ui.input(|i| i.modifiers.shift) {
                self.fine_multiplier
            } else {
                1.0
            };
            let speed = range_span / self.sensitivity;
            let delta = (raw_delta as f64) * speed as f64 * multiplier as f64;
            *self.value = (*self.value + delta).clamp(min, max);
        }

        if response.double_clicked() {
            if let Some(default) = self.default_value {
                *self.value = default;
            }
        }

        self.normalized()
    }

    /// Returns the current value normalized to 0.0..=1.0.
    pub fn normalized(&self) -> f32 {
        let min = *self.range.start();
        let max = *self.range.end();
        ((*self.value - min) / (max - min)).clamp(0.0, 1.0) as f32
    }
}

// ---------------------------------------------------------------------------
// Knob
// ---------------------------------------------------------------------------

/// Rotary knob widget for continuous values.
pub struct Knob<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    size: f32,
    label: Option<String>,
    default_value: Option<f64>,
    knob_style: KnobStyle,
    knob_size_preset: Option<KnobSize>,
}

impl<'a> Knob<'a> {
    /// Create a new Knob.
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            size: 48.0,
            label: None,
            default_value: None,
            knob_style: KnobStyle::Default,
            knob_size_preset: None,
        }
    }

    /// Set the knob size (square dimension).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the label shown below the knob.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the default value (reset on double-click).
    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }

    pub fn style(mut self, style: KnobStyle) -> Self {
        self.knob_style = style;
        self
    }

    pub fn preset_size(mut self, size: KnobSize) -> Self {
        self.knob_size_preset = Some(size);
        self
    }
}

impl<'a> egui::Widget for Knob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut diameter = self.size;
        if let Some(preset) = self.knob_size_preset {
            diameter = preset.to_px();
        }
        let size = Vec2::splat(diameter);
        let response = ui.allocate_rect(
            Rect::from_center_size(ui.cursor().center(), size),
            Sense::click_and_drag(),
        );

        let mut ctrl = ContinuousControl::new(self.value, self.range.clone()).axis(DragAxis::Y);
        if let Some(dv) = self.default_value {
            ctrl = ctrl.default_value(dv);
        }
        let t = ctrl.handle(ui, &response);

        // Draw
        let rect = response.rect;
        let center = rect.center();
        let radius = diameter / 2.0 - 2.0;

        let painter = ui.painter();

        // Arc parameters: 225° to 135° going clockwise (270° sweep)
        let min_angle = 225f32.to_radians();
        let sweep = 270f32.to_radians();

        // Colors from visuals
        let visuals = ui.visuals();
        let track_color = visuals.widgets.inactive.bg_stroke.color;
        let value_color = visuals.selection.stroke.color;
        let bg_color = visuals.widgets.inactive.bg_fill;
        let indicator_color = visuals.widgets.active.fg_stroke.color;
        let label_color = visuals.text_color();

        // Paint based on style
        match self.knob_style {
            KnobStyle::Default => {
                // Background circle
                painter.circle_filled(center, radius, bg_color);

                let value_angle = min_angle + t * sweep;

                // Draw arc track
                let track_points: Vec<Pos2> = (0..=64)
                    .map(|i| {
                        let angle = min_angle + (sweep * i as f32) / 64.0;
                        center + EMathVec2::angled(angle) * radius
                    })
                    .collect();
                painter.add(Shape::Path(PathShape {
                    points: track_points,
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: PathStroke::new(2.0, track_color),
                }));

                // Draw value arc
                if t > 0.0 {
                    let value_arc_points: Vec<Pos2> = (0..=32)
                        .map(|i| {
                            let angle = min_angle + (value_angle - min_angle) * (i as f32) / 32.0;
                            center + EMathVec2::angled(angle) * radius
                        })
                        .collect();
                    painter.add(Shape::Path(PathShape {
                        points: value_arc_points,
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: PathStroke::new(3.0, value_color),
                    }));
                }

                // Indicator line
                let indicator_inner = radius * 0.3;
                let indicator_outer = radius * 0.75;
                let line_start = center + EMathVec2::angled(value_angle) * indicator_inner;
                let line_end = center + EMathVec2::angled(value_angle) * indicator_outer;
                painter.add(Shape::LineSegment {
                    points: [line_start, line_end],
                    stroke: Stroke::new(2.5, indicator_color),
                });
            }
            KnobStyle::Flat => {
                paint_knob_flat(&painter, center, radius, t, track_color, value_color);
            }
            KnobStyle::Ring => {
                paint_knob_ring(&painter, center, radius, t, track_color, value_color);
            }
            KnobStyle::Notched => {
                paint_knob_notched(&painter, center, radius, t, track_color, value_color, 13);
            }
        }

        // Label
        if let Some(label) = &self.label {
            let label_pos = Pos2::new(center.x, rect.max.y - 2.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.0),
                label_color,
            );
        }

        response
    }
}

// ---------------------------------------------------------------------------
// Knob paint helpers
// ---------------------------------------------------------------------------

fn paint_knob_flat(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    _track_color: egui::Color32,
    value_color: egui::Color32,
) {
    painter.circle_filled(center, radius, value_color);
    // Indicator line
    let angle = PI * 0.75 + normalized * PI * 1.5;
    let inner = egui::Pos2::new(
        center.x + angle.cos() * radius * 0.3,
        center.y + angle.sin() * radius * 0.3,
    );
    let outer = egui::Pos2::new(
        center.x + angle.cos() * radius * 0.85,
        center.y + angle.sin() * radius * 0.85,
    );
    painter.line_segment([inner, outer], egui::Stroke::new(2.0, egui::Color32::WHITE));
}

fn paint_knob_ring(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    track_color: egui::Color32,
    value_color: egui::Color32,
) {
    painter.circle_stroke(center, radius * 0.9, egui::Stroke::new(2.0, track_color));
    // Value arc
    let start_angle = PI * 0.75;
    let sweep = normalized * PI * 1.5;
    let points: Vec<egui::Pos2> = (0..=20)
        .map(|i| {
            let a = start_angle + sweep * (i as f32 / 20.0);
            egui::Pos2::new(
                center.x + a.cos() * radius * 0.9,
                center.y + a.sin() * radius * 0.9,
            )
        })
        .collect();
    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.5, value_color),
        ));
    }
    // Indicator dot
    let angle = start_angle + sweep;
    let dot = egui::Pos2::new(
        center.x + angle.cos() * radius * 0.9,
        center.y + angle.sin() * radius * 0.9,
    );
    painter.circle_filled(dot, 3.0, value_color);
}

fn paint_knob_notched(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    normalized: f32,
    track_color: egui::Color32,
    value_color: egui::Color32,
    ticks: usize,
) {
    let start_angle = PI * 0.75;
    let total_sweep = PI * 1.5;
    let active_angle = start_angle + normalized * total_sweep;

    for i in 0..ticks {
        let t = i as f32 / (ticks - 1) as f32;
        let angle = start_angle + t * total_sweep;
        let color = if angle <= active_angle {
            value_color
        } else {
            track_color
        };
        let inner = egui::Pos2::new(
            center.x + angle.cos() * radius * 0.65,
            center.y + angle.sin() * radius * 0.65,
        );
        let outer = egui::Pos2::new(
            center.x + angle.cos() * radius * 0.9,
            center.y + angle.sin() * radius * 0.9,
        );
        painter.line_segment([inner, outer], egui::Stroke::new(2.0, color));
    }
    // Center dot
    painter.circle_filled(center, radius * 0.15, value_color);
}

// ---------------------------------------------------------------------------
// Fader
// ---------------------------------------------------------------------------

/// Linear slider / fader widget.
pub struct Fader<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    size: Vec2,
    orientation: Orientation,
    label: Option<String>,
    meter_value: Option<f32>,
    meter_value_r: Option<f32>,
    meter_segmented: bool,
}

impl<'a> Fader<'a> {
    /// Create a new Fader with default size 40x120.
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            size: Vec2::new(40.0, 120.0),
            orientation: Orientation::Vertical,
            label: None,
            meter_value: None,
            meter_value_r: None,
            meter_segmented: false,
        }
    }

    /// Set the fader size.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Set the fader orientation.
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }

    /// Set the label shown beside the fader.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn meter_value(mut self, v: f32) -> Self {
        self.meter_value = Some(v.clamp(0.0, 1.0));
        self
    }

    pub fn stereo_meter(mut self, l: f32, r: f32) -> Self {
        self.meter_value = Some(l.clamp(0.0, 1.0));
        self.meter_value_r = Some(r.clamp(0.0, 1.0));
        self
    }

    pub fn meter_segmented(mut self, segmented: bool) -> Self {
        self.meter_segmented = segmented;
        self
    }
}

impl<'a> egui::Widget for Fader<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(
            Rect::from_min_size(ui.cursor().min, self.size),
            Sense::click_and_drag(),
        );

        let rect = response.rect;
        let painter = ui.painter();
        let hovered = response.hovered();

        // Get visuals for theming
        let visuals = ui.visuals();
        let track_bg = visuals.widgets.inactive.bg_fill;
        let thumb_normal = visuals.widgets.inactive.bg_fill.gamma_multiply(1.3);
        let thumb_hovered = visuals.widgets.hovered.bg_fill;
        let label_color = visuals.text_color();

        let min = *self.range.start();
        let max = *self.range.end();
        let t = ((*self.value - min) / (max - min)).clamp(0.0, 1.0) as f32;

        // Track dimensions
        let track_width = 6.0;
        let thumb_width = if self.orientation == Orientation::Vertical {
            self.size.x - 4.0
        } else {
            self.size.y - 4.0
        };
        let thumb_height = if self.orientation == Orientation::Vertical {
            track_width + 4.0
        } else {
            self.size.x - 4.0
        };

        // Calculate track_rect based on orientation
        let track_rect = match self.orientation {
            Orientation::Vertical => Rect::from_min_max(
                Pos2::new(rect.center().x - track_width / 2.0, rect.min.y + 4.0),
                Pos2::new(rect.center().x + track_width / 2.0, rect.max.y - 4.0),
            ),
            Orientation::Horizontal => Rect::from_min_max(
                Pos2::new(rect.min.x + 4.0, rect.center().y - track_width / 2.0),
                Pos2::new(rect.max.x - 4.0, rect.center().y + track_width / 2.0),
            ),
        };

        // Draw track
        painter.rect_filled(track_rect, 2.0, track_bg);

        // Draw integrated VU meter in track
        if let Some(level) = self.meter_value {
            if self.meter_value_r.is_some() {
                // Stereo: render two side-by-side meters
                let level_r = self.meter_value_r.unwrap();
                let left_track = Rect::from_min_max(
                    track_rect.min,
                    Pos2::new(track_rect.center().x, track_rect.max.y),
                );
                let right_track = Rect::from_min_max(
                    Pos2::new(track_rect.center().x, track_rect.min.y),
                    track_rect.max,
                );

                // Draw left (L) meter
                draw_meter_in_track(
                    painter,
                    left_track,
                    level,
                    self.meter_segmented,
                    self.orientation,
                );
                // Draw right (R) meter
                draw_meter_in_track(
                    painter,
                    right_track,
                    level_r,
                    self.meter_segmented,
                    self.orientation,
                );
            } else {
                // Mono: render single meter across full track width
                draw_meter_in_track(
                    painter,
                    track_rect,
                    level,
                    self.meter_segmented,
                    self.orientation,
                );
            }
        }

        // Draw thumb
        match self.orientation {
            Orientation::Vertical => {
                // Thumb position (from bottom)
                let thumb_y = rect.max.y - 4.0 - t * (rect.height() - 8.0 - thumb_height);
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 2.0, thumb_y),
                    Vec2::new(thumb_width, thumb_height),
                );
                let thumb_color = if hovered { thumb_hovered } else { thumb_normal };
                painter.rect_filled(thumb_rect, 2.0, thumb_color);
            }
            Orientation::Horizontal => {
                // Thumb position (from left)
                let thumb_x = rect.min.x + 4.0 + t * (rect.width() - 8.0 - thumb_width);
                let thumb_rect = Rect::from_min_size(
                    Pos2::new(thumb_x, rect.min.y + 2.0),
                    Vec2::new(thumb_width, thumb_height),
                );
                let thumb_color = if hovered { thumb_hovered } else { thumb_normal };
                painter.rect_filled(thumb_rect, 2.0, thumb_color);
            }
        }

        let axis = match self.orientation {
            Orientation::Vertical => DragAxis::Y,
            Orientation::Horizontal => DragAxis::X,
        };
        let mut ctrl = ContinuousControl::new(self.value, self.range.clone()).axis(axis);
        let _t = ctrl.handle(ui, &response);

        // Label
        if let Some(label) = &self.label {
            let label_pos = match self.orientation {
                Orientation::Vertical => Pos2::new(rect.max.x + 4.0, rect.center().y),
                Orientation::Horizontal => Pos2::new(rect.center().x, rect.min.y - 4.0),
            };
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(10.0),
                label_color,
            );
        }

        response
    }
}

fn meter_level_color(level: f32) -> egui::Color32 {
    if level > 0.85 {
        egui::Color32::from_rgb(220, 70, 70) // red
    } else if level > 0.65 {
        egui::Color32::from_rgb(220, 180, 60) // yellow
    } else {
        egui::Color32::from_rgb(80, 180, 120) // green
    }
}

/// Draw a single meter bar into the given track rect.
fn draw_meter_in_track(
    painter: &egui::Painter,
    track_rect: Rect,
    level: f32,
    segmented: bool,
    orientation: Orientation,
) {
    let meter_color = meter_level_color(level);
    if segmented {
        // Draw segmented meter
        let segments = 20usize;
        let active = (level * segments as f32) as usize;
        for seg in 0..segments {
            let t_seg = seg as f32 / segments as f32;
            let seg_rect = if orientation == Orientation::Vertical {
                let y_top =
                    track_rect.max.y - track_rect.height() * ((seg + 1) as f32 / segments as f32);
                let y_bot = track_rect.max.y - track_rect.height() * (seg as f32 / segments as f32);
                egui::Rect::from_min_max(
                    egui::Pos2::new(track_rect.min.x + 1.0, y_top + 1.0),
                    egui::Pos2::new(track_rect.max.x - 1.0, y_bot - 1.0),
                )
            } else {
                let x_left = track_rect.min.x + track_rect.width() * (seg as f32 / segments as f32);
                let x_right =
                    track_rect.min.x + track_rect.width() * ((seg + 1) as f32 / segments as f32);
                egui::Rect::from_min_max(
                    egui::Pos2::new(x_left + 1.0, track_rect.min.y + 1.0),
                    egui::Pos2::new(x_right - 1.0, track_rect.max.y - 1.0),
                )
            };
            let seg_color = if seg < active {
                meter_level_color(t_seg)
            } else {
                egui::Color32::from_gray(40)
            };
            painter.rect_filled(seg_rect, egui::CornerRadius::ZERO, seg_color);
        }
    } else {
        // Continuous meter fill
        let fill_rect = if orientation == Orientation::Vertical {
            let fill_h = track_rect.height() * level;
            egui::Rect::from_min_max(
                egui::Pos2::new(track_rect.min.x + 1.0, track_rect.max.y - fill_h),
                egui::Pos2::new(track_rect.max.x - 1.0, track_rect.max.y),
            )
        } else {
            let fill_w = track_rect.width() * level;
            egui::Rect::from_min_max(
                egui::Pos2::new(track_rect.min.x, track_rect.min.y + 1.0),
                egui::Pos2::new(track_rect.min.x + fill_w, track_rect.max.y - 1.0),
            )
        };
        painter.rect_filled(fill_rect, egui::CornerRadius::ZERO, meter_color);
    }
}

// ---------------------------------------------------------------------------
// ToggleDot
// ---------------------------------------------------------------------------

/// The 4 visual states of a mute/solo dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DotState {
    /// Normal active state (green/lit).
    #[default]
    On,
    /// Muted (orange).
    Muted,
    /// Solo (yellow).
    Solo,
    /// Solo-muted (dimmed).
    SoloMuted,
    /// Off/inactive (dark).
    Off,
}

impl DotState {
    pub fn color(self) -> egui::Color32 {
        match self {
            DotState::On => egui::Color32::from_rgb(80, 180, 120),
            DotState::Muted => egui::Color32::from_rgb(220, 140, 60),
            DotState::Solo => egui::Color32::from_rgb(220, 200, 60),
            DotState::SoloMuted => egui::Color32::from_rgb(80, 80, 90),
            DotState::Off => egui::Color32::from_rgb(45, 45, 52),
        }
    }

    /// Toggle between On and Off (simple 2-state use).
    pub fn toggle(self) -> Self {
        match self {
            DotState::On => DotState::Off,
            _ => DotState::On,
        }
    }
}

/// A small colored dot toggle button (mute/solo indicator).
/// Supports 2-state (on/off) and 4-state (on/muted/solo/solo-muted) modes.
pub struct ToggleDot<'a> {
    state: &'a mut DotState,
    size: f32,
    id: egui::Id,
}

impl<'a> ToggleDot<'a> {
    pub fn new(id: impl std::hash::Hash, state: &'a mut DotState) -> Self {
        Self {
            state,
            size: 8.0,
            id: egui::Id::new(id),
        }
    }

    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl<'a> egui::Widget for ToggleDot<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(self.size + 4.0), egui::Sense::click());
        if response.clicked() {
            *self.state = self.state.toggle();
        }
        let painter = ui.painter();
        let center = rect.center();
        let color = self.state.color();
        let border = if response.hovered() {
            ui.visuals().widgets.hovered.bg_stroke.color
        } else {
            ui.visuals().widgets.inactive.bg_stroke.color
        };
        painter.circle_filled(center, self.size * 0.5, color);
        painter.circle_stroke(center, self.size * 0.5, egui::Stroke::new(1.0, border));
        response
    }
}

// ---------------------------------------------------------------------------
// TransportButton
// ---------------------------------------------------------------------------

/// Transport control button kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportKind {
    Play,
    Stop,
    Record,
    Metronome,
    Loop,
}

/// A transport control button (play/stop/record/metronome/loop).
pub struct TransportButton<'a> {
    kind: TransportKind,
    active: &'a mut bool,
    size: f32,
}

impl<'a> TransportButton<'a> {
    pub fn new(kind: TransportKind, active: &'a mut bool) -> Self {
        Self {
            kind,
            active,
            size: 28.0,
        }
    }

    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl<'a> egui::Widget for TransportButton<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(self.size), egui::Sense::click());
        if response.clicked() {
            *self.active = !*self.active;
        }

        let painter = ui.painter();
        let center = rect.center();
        let r = self.size * 0.5;

        // Background
        let visuals = ui.visuals();
        let bg = if *self.active {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };
        painter.rect_filled(rect, egui::CornerRadius::same(4), bg);

        // Icon
        let icon_color = if *self.active {
            visuals.widgets.active.fg_stroke.color
        } else {
            visuals.widgets.inactive.fg_stroke.color
        };

        match self.kind {
            TransportKind::Play => {
                let pts = vec![
                    egui::Pos2::new(center.x - r * 0.3, center.y - r * 0.45),
                    egui::Pos2::new(center.x + r * 0.45, center.y),
                    egui::Pos2::new(center.x - r * 0.3, center.y + r * 0.45),
                ];
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    icon_color,
                    egui::Stroke::NONE,
                ));
            }
            TransportKind::Stop => {
                let s = r * 0.55;
                painter.rect_filled(
                    egui::Rect::from_center_size(center, egui::Vec2::splat(s * 2.0)),
                    egui::CornerRadius::ZERO,
                    icon_color,
                );
            }
            TransportKind::Record => {
                painter.circle_filled(center, r * 0.4, egui::Color32::from_rgb(220, 70, 70));
            }
            TransportKind::Metronome => {
                // Simplified: vertical line with tick
                let top = egui::Pos2::new(center.x, center.y - r * 0.5);
                let bot = egui::Pos2::new(center.x, center.y + r * 0.5);
                painter.line_segment([top, bot], egui::Stroke::new(2.0, icon_color));
                let tick = egui::Pos2::new(center.x + r * 0.3, center.y - r * 0.1);
                painter.line_segment([center, tick], egui::Stroke::new(2.0, icon_color));
            }
            TransportKind::Loop => {
                painter.circle_stroke(center, r * 0.4, egui::Stroke::new(2.0, icon_color));
                // Arrow tip
                let tip = egui::Pos2::new(center.x + r * 0.4, center.y);
                let a1 = egui::Pos2::new(tip.x - r * 0.15, tip.y - r * 0.15);
                let a2 = egui::Pos2::new(tip.x + r * 0.15, tip.y - r * 0.15);
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, a1, a2],
                    icon_color,
                    egui::Stroke::NONE,
                ));
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// Meter
// ---------------------------------------------------------------------------

/// Audio level meter widget.
#[derive(Debug, Clone)]
pub struct Meter {
    value: f32,
    peak: Option<f32>,
    orientation: Orientation,
    size: Vec2,
    segments: u32,
    clip_threshold: f32,
}

impl Meter {
    /// Create a new Meter.
    pub fn new(value: f32) -> Self {
        Self {
            value,
            peak: None,
            orientation: Orientation::Vertical,
            size: Vec2::new(16.0, 80.0),
            segments: 0,
            clip_threshold: 0.9,
        }
    }

    /// Set the peak hold value.
    pub fn peak(mut self, peak: f32) -> Self {
        self.peak = Some(peak);
        self
    }

    /// Set the meter size.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Set the meter orientation.
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }

    /// Set the number of discrete segments (0 = continuous).
    pub fn segments(mut self, n: u32) -> Self {
        self.segments = n;
        self
    }

    /// Set the clip threshold (yellow→red transition).
    pub fn clip_threshold(mut self, t: f32) -> Self {
        self.clip_threshold = t;
        self
    }

    fn level_color(&self, level: f32) -> Color32 {
        if level < 0.7 {
            Color32::from_rgb(60, 180, 80)
        } else if level < self.clip_threshold {
            Color32::from_rgb(220, 180, 0)
        } else {
            Color32::from_rgb(220, 50, 50)
        }
    }
}

impl egui::Widget for Meter {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(
            Rect::from_min_size(ui.cursor().min, self.size),
            Sense::hover(),
        );
        let rect = response.rect;
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, 1.0, ui.visuals().extreme_bg_color);

        let t = self.value.clamp(0.0, 1.0);

        if self.segments == 0 {
            // Continuous fill
            let fill_rect = match self.orientation {
                Orientation::Vertical => {
                    let height = rect.height() * t;
                    Rect::from_min_max(
                        Pos2::new(rect.min.x + 1.0, rect.max.y - height),
                        Pos2::new(rect.max.x - 1.0, rect.max.y - 1.0),
                    )
                }
                Orientation::Horizontal => {
                    let width = rect.width() * t;
                    Rect::from_min_max(
                        Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0),
                        Pos2::new(rect.min.x + width, rect.max.y - 1.0),
                    )
                }
            };
            painter.rect_filled(fill_rect, 0.0, self.level_color(t));
        } else {
            // Segmented fill
            let seg_count = self.segments as f32;
            let seg_gap = 2.0;
            match self.orientation {
                Orientation::Vertical => {
                    let seg_height = (rect.height() - (seg_count - 1.0) * seg_gap) / seg_count;
                    let filled_segs = (t * seg_count).floor() as i32;
                    for i in 0..self.segments as i32 {
                        let seg_idx = self.segments as i32 - 1 - i; // bottom to top
                        if seg_idx < filled_segs {
                            let y_top =
                                rect.max.y - (i as f32) * (seg_height + seg_gap) - seg_height;
                            let seg_rect = Rect::from_min_size(
                                Pos2::new(rect.min.x + 1.0, y_top),
                                Vec2::new(rect.width() - 2.0, seg_height),
                            );
                            let seg_t = (i as f32 + 1.0) / seg_count;
                            painter.rect_filled(seg_rect, 0.0, self.level_color(seg_t));
                        }
                    }
                }
                Orientation::Horizontal => {
                    let seg_width = (rect.width() - (seg_count - 1.0) * seg_gap) / seg_count;
                    let filled_segs = (t * seg_count).floor() as i32;
                    for i in 0..self.segments as i32 {
                        if i < filled_segs {
                            let x_left = rect.min.x + (i as f32) * (seg_width + seg_gap);
                            let seg_rect = Rect::from_min_size(
                                Pos2::new(x_left, rect.min.y + 1.0),
                                Vec2::new(seg_width, rect.height() - 2.0),
                            );
                            let seg_t = (i as f32 + 1.0) / seg_count;
                            painter.rect_filled(seg_rect, 0.0, self.level_color(seg_t));
                        }
                    }
                }
            }
        }

        // Peak hold line
        if let Some(peak) = self.peak {
            let peak_t = peak.clamp(0.0, 1.0);
            let peak_color = Color32::from_rgb(255, 80, 80);
            match self.orientation {
                Orientation::Vertical => {
                    let peak_y = rect.max.y - peak_t * rect.height();
                    if peak_y > rect.min.y {
                        painter.line_segment(
                            [Pos2::new(rect.min.x, peak_y), Pos2::new(rect.max.x, peak_y)],
                            Stroke::new(1.5, peak_color),
                        );
                    }
                }
                Orientation::Horizontal => {
                    let peak_x = rect.min.x + peak_t * rect.width();
                    if peak_x < rect.max.x {
                        painter.line_segment(
                            [Pos2::new(peak_x, rect.min.y), Pos2::new(peak_x, rect.max.y)],
                            Stroke::new(1.5, peak_color),
                        );
                    }
                }
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// StepGrid
// ---------------------------------------------------------------------------

/// Boolean step sequencer grid widget.
pub struct StepGrid<'a> {
    steps: &'a mut Vec<Vec<bool>>,
    rows: usize,
    cols: usize,
    cell_size: Vec2,
    active_col: Option<usize>,
    row_colors: Option<Vec<Color32>>,
}

impl<'a> StepGrid<'a> {
    /// Create a new StepGrid.
    pub fn new(steps: &'a mut Vec<Vec<bool>>, rows: usize, cols: usize) -> Self {
        // Ensure steps is properly sized
        while steps.len() < rows {
            steps.push(vec![false; cols]);
        }
        for row in steps.iter_mut() {
            while row.len() < cols {
                row.push(false);
            }
        }

        Self {
            steps,
            rows,
            cols,
            cell_size: Vec2::splat(28.0),
            active_col: None,
            row_colors: None,
        }
    }

    /// Set the cell size.
    pub fn cell_size(mut self, size: Vec2) -> Self {
        self.cell_size = size;
        self
    }

    /// Set the active column (highlighted).
    pub fn active_col(mut self, col: usize) -> Self {
        self.active_col = Some(col);
        self
    }

    /// Set per-row colors for active cells.
    pub fn row_colors(mut self, colors: Vec<Color32>) -> Self {
        self.row_colors = Some(colors);
        self
    }
}

impl<'a> egui::Widget for StepGrid<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let total_size = Vec2::new(
            (self.cols as f32) * self.cell_size.x,
            (self.rows as f32) * self.cell_size.y,
        );
        let response = ui.allocate_rect(
            Rect::from_min_size(ui.cursor().min, total_size),
            Sense::click_and_drag(),
        );
        let rect = response.rect;
        let painter = ui.painter();
        let visuals = ui.visuals();

        // Ensure steps is correctly sized
        while self.steps.len() < self.rows {
            self.steps.push(vec![false; self.cols]);
        }
        for row in self.steps.iter_mut() {
            while row.len() < self.cols {
                row.push(false);
            }
        }

        // Active column highlight
        if let Some(col) = self.active_col {
            if col < self.cols {
                let col_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + (col as f32) * self.cell_size.x, rect.min.y),
                    self.cell_size,
                );
                painter.rect_filled(
                    col_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 20),
                );
            }
        }

        // Draw cells
        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + (col as f32) * self.cell_size.x,
                        rect.min.y + (row as f32) * self.cell_size.y,
                    ),
                    self.cell_size,
                );
                let inner_rect = cell_rect.shrink(2.0);
                let is_active = self.steps[row][col];

                let color = if is_active {
                    if let Some(ref colors) = self.row_colors {
                        colors
                            .get(row)
                            .copied()
                            .unwrap_or(Color32::from_rgb(80, 140, 255))
                    } else {
                        Color32::from_rgb(80, 140, 255)
                    }
                } else {
                    visuals.widgets.inactive.bg_fill
                };

                painter.rect_filled(inner_rect, 2.0, color);
            }
        }

        // Handle interaction
        let pointer_pos = response.interact_pointer_pos();

        if let Some(pos) = pointer_pos {
            if rect.contains(pos) {
                let col = ((pos.x - rect.min.x) / self.cell_size.x).floor() as usize;
                let row = ((pos.y - rect.min.y) / self.cell_size.y).floor() as usize;

                if col < self.cols && row < self.rows {
                    if response.dragged() {
                        // Drag-to-toggle: use memory to track the target state
                        let drag_id = response.id.with("drag_target");
                        let target_state = ui
                            .ctx()
                            .memory(|m| m.data.get_temp::<bool>(drag_id))
                            .unwrap_or_else(|| {
                                // First cell in drag: toggle and store
                                let new_state = !self.steps[row][col];
                                ui.ctx()
                                    .memory_mut(|m| m.data.insert_temp(drag_id, new_state));
                                new_state
                            });

                        if self.steps[row][col] != target_state {
                            self.steps[row][col] = target_state;
                        }
                    } else if response.clicked() {
                        // Single click: toggle
                        self.steps[row][col] = !self.steps[row][col];
                    }
                }
            }
        }

        response
    }
}

// ─── ContextMenuBuilder ───────────────────────────────────────────────────────

/// An item in a context menu.
enum ContextMenuItem {
    Action {
        label: String,
        shortcut: Option<String>,
        enabled: bool,
        callback: Box<dyn FnOnce()>,
    },
    Checked {
        label: String,
        checked: bool,
        callback: Box<dyn FnOnce(bool)>,
    },
    Separator,
}

/// Builder for a structured context menu with separators, shortcuts, and disabled items.
///
/// # Example
/// ```ignore
/// response.context_menu(|ui| {
///     ContextMenuBuilder::new()
///         .action("Cut", Some("Ctrl+X"), || { /* ... */ })
///         .action("Copy", Some("Ctrl+C"), || { /* ... */ })
///         .separator()
///         .disabled("Paste")
///         .show(ui);
/// });
/// ```
pub struct ContextMenuBuilder {
    items: Vec<ContextMenuItem>,
}

impl ContextMenuBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a clickable action item.
    pub fn action(
        mut self,
        label: impl Into<String>,
        shortcut: Option<&str>,
        callback: impl FnOnce() + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Action {
            label: label.into(),
            shortcut: shortcut.map(|s| s.to_string()),
            enabled: true,
            callback: Box::new(callback),
        });
        self
    }

    /// Add a disabled (grayed out) action item.
    pub fn disabled(mut self, label: impl Into<String>) -> Self {
        self.items.push(ContextMenuItem::Action {
            label: label.into(),
            shortcut: None,
            enabled: false,
            callback: Box::new(|| {}),
        });
        self
    }

    /// Add a checkable item.
    pub fn checked(
        mut self,
        label: impl Into<String>,
        checked: bool,
        callback: impl FnOnce(bool) + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Checked {
            label: label.into(),
            checked,
            callback: Box::new(callback),
        });
        self
    }

    /// Add a visual separator.
    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::Separator);
        self
    }

    /// Render the menu into the given `Ui` (call inside `response.context_menu(|ui| { ... })`).
    pub fn show(self, ui: &mut egui::Ui) {
        let hint_color = ui.visuals().widgets.noninteractive.bg_stroke.color;

        for item in self.items {
            match item {
                ContextMenuItem::Separator => {
                    ui.separator();
                }
                ContextMenuItem::Action {
                    label,
                    shortcut,
                    enabled,
                    callback,
                } => {
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.horizontal(|ui| {
                            let resp = ui.selectable_label(false, &label);
                            if let Some(sc) = shortcut {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(sc).color(hint_color).size(10.0),
                                        );
                                    },
                                );
                            }
                            if resp.clicked() {
                                (callback)();
                                ui.close_menu();
                            }
                        });
                    });
                }
                ContextMenuItem::Checked {
                    label,
                    checked,
                    callback,
                } => {
                    ui.horizontal(|ui| {
                        let check_str = if checked { "✓ " } else { "  " };
                        let full_label = format!("{}{}", check_str, label);
                        if ui.selectable_label(checked, &full_label).clicked() {
                            (callback)(!checked);
                            ui.close_menu();
                        }
                    });
                }
            }
        }
    }
}

impl Default for ContextMenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ─── FloatingPanel ────────────────────────────────────────────────────────────

/// A floating panel wrapping `egui::Window` with design-token styling.
///
/// # Example
/// ```ignore
/// FloatingPanel::new("my_panel", "Panel Title")
///     .default_pos([100.0, 100.0])
///     .default_size([300.0, 400.0])
///     .show(ctx, |ui| {
///         ui.label("Panel content");
///     });
/// ```
pub struct FloatingPanel<'a> {
    id: egui::Id,
    title: String,
    default_pos: Option<egui::Pos2>,
    default_size: Option<egui::Vec2>,
    resizable: bool,
    open: Option<&'a mut bool>,
}

impl<'a> FloatingPanel<'a> {
    pub fn new(id: impl std::hash::Hash, title: impl Into<String>) -> Self {
        Self {
            id: egui::Id::new(id),
            title: title.into(),
            default_pos: None,
            default_size: None,
            resizable: true,
            open: None,
        }
    }

    pub fn default_pos(mut self, pos: impl Into<egui::Pos2>) -> Self {
        self.default_pos = Some(pos.into());
        self
    }

    pub fn default_size(mut self, size: impl Into<egui::Vec2>) -> Self {
        self.default_size = Some(size.into());
        self
    }

    pub fn resizable(self, r: bool) -> Self {
        Self {
            resizable: r,
            ..self
        }
    }

    /// Pass a mutable bool reference to make the panel's open state observable by the caller.
    pub fn open(mut self, open: &'a mut bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Show the panel. Returns `Some(inner_response)` if visible.
    pub fn show<R>(
        self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<egui::InnerResponse<Option<R>>> {
        // Build a Frame from current style, then override fill and stroke
        let frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgb(22, 22, 27))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 47)));

        let mut window = egui::Window::new(&self.title)
            .id(self.id)
            .resizable(self.resizable)
            .collapsible(false)
            .frame(frame);

        if let Some(pos) = self.default_pos {
            window = window.default_pos(pos);
        }
        if let Some(size) = self.default_size {
            window = window.default_size(size);
        }

        // Wire open state directly to caller's mutable reference
        if let Some(open_ref) = self.open {
            window = window.open(open_ref);
        }

        window.show(ctx, add_contents)
    }
}

// ─── TabBar ───────────────────────────────────────────────────────────────────

/// A horizontal tab bar widget.
///
/// # Example
/// ```ignore
/// let mut active = 0usize;
/// TabBar::new("my_tabs", &mut active)
///     .tab("Files")
///     .tab("Plugins")
///     .tab("Samples")
///     .show(ui);
/// ```
pub struct TabBar<'a> {
    id: egui::Id,
    active: &'a mut usize,
    tabs: Vec<String>,
    height: f32,
}

impl<'a> TabBar<'a> {
    pub fn new(id: impl std::hash::Hash, active: &'a mut usize) -> Self {
        Self {
            id: egui::Id::new(id),
            active,
            tabs: Vec::new(),
            height: 28.0,
        }
    }

    pub fn tab(mut self, label: impl Into<String>) -> Self {
        self.tabs.push(label.into());
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Show the tab bar. Returns the index of the active tab.
    pub fn show(self, ui: &mut egui::Ui) -> usize {
        let active = *self.active;
        let tab_count = self.tabs.len();
        if tab_count == 0 {
            return active;
        }

        let tab_width = (ui.available_width() / tab_count as f32).min(120.0);
        let total_width = tab_width * tab_count as f32;
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(total_width, self.height),
            egui::Sense::hover(),
        );

        let painter = ui.painter();
        let visuals = ui.visuals();
        let bg = visuals.widgets.inactive.bg_fill;
        let active_bg = bg;
        let active_line = visuals.selection.stroke.color;
        let text_active = visuals.strong_text_color();
        let text_inactive = visuals.text_color();

        // Background
        painter.rect_filled(rect, egui::CornerRadius::ZERO, bg);

        for (i, label) in self.tabs.iter().enumerate() {
            let tab_rect = egui::Rect::from_min_size(
                egui::Pos2::new(rect.min.x + i as f32 * tab_width, rect.min.y),
                egui::Vec2::new(tab_width, self.height),
            );

            let is_active = i == active;
            let tab_bg = if is_active { active_bg } else { bg };
            painter.rect_filled(tab_rect, egui::CornerRadius::ZERO, tab_bg);

            if is_active {
                // Active indicator line at bottom
                painter.line_segment(
                    [
                        egui::Pos2::new(tab_rect.min.x + 2.0, tab_rect.max.y - 1.0),
                        egui::Pos2::new(tab_rect.max.x - 2.0, tab_rect.max.y - 1.0),
                    ],
                    egui::Stroke::new(2.0, active_line),
                );
            }

            let text_color = if is_active {
                text_active
            } else {
                text_inactive
            };
            let font_id = egui::FontId::proportional(11.0);
            painter.text(
                tab_rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font_id,
                text_color,
            );

            // Click detection
            let tab_response = ui.interact(tab_rect, self.id.with(i), egui::Sense::click());
            if tab_response.clicked() {
                *self.active = i;
            }
        }

        *self.active
    }
}

// ─── TreeView ─────────────────────────────────────────────────────────────────

/// A node in a tree view.
pub struct TreeNode {
    pub label: String,
    pub icon: Option<char>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn child(mut self, node: TreeNode) -> Self {
        self.children.push(node);
        self
    }
}

/// A collapsible tree view widget.
///
/// # Example
/// ```ignore
/// TreeView::new("browser_tree")
///     .node(TreeNode::new("Samples").icon('📁')
///         .child(TreeNode::new("Kick.wav").icon('🎵'))
///         .child(TreeNode::new("Snare.wav").icon('🎵')))
///     .show(ui);
/// ```
pub struct TreeView {
    id: egui::Id,
    nodes: Vec<TreeNode>,
    row_height: f32,
}

impl TreeView {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            nodes: Vec::new(),
            row_height: 22.0,
        }
    }

    pub fn node(mut self, node: TreeNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let row_height = self.row_height;
        let id = self.id;
        for (i, node) in self.nodes.iter().enumerate() {
            Self::show_node(ui, node, 0, id.with(i), row_height);
        }
    }

    fn show_node(ui: &mut egui::Ui, node: &TreeNode, depth: usize, id: egui::Id, row_height: f32) {
        let indent = depth as f32 * 14.0;
        let has_children = !node.children.is_empty();

        // Load expanded state
        let expanded = ui.memory(|mem| mem.data.get_temp::<bool>(id).unwrap_or(false));

        let row_width = ui.available_width();
        let (row_rect, row_response) =
            ui.allocate_exact_size(egui::Vec2::new(row_width, row_height), egui::Sense::click());

        let painter = ui.painter();

        // Hover background
        if row_response.hovered() {
            painter.rect_filled(
                row_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgb(40, 40, 47),
            );
        }

        let text_x = row_rect.min.x + indent + 4.0;
        let center_y = row_rect.center().y;

        // Expand/collapse arrow
        if has_children {
            let arrow_x = text_x;
            let arrow_center = egui::Pos2::new(arrow_x + 6.0, center_y);
            let arrow_color = egui::Color32::from_rgb(130, 130, 142);
            if expanded {
                // Down arrow
                let pts = vec![
                    egui::Pos2::new(arrow_center.x - 4.0, arrow_center.y - 2.0),
                    egui::Pos2::new(arrow_center.x + 4.0, arrow_center.y - 2.0),
                    egui::Pos2::new(arrow_center.x, arrow_center.y + 3.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    arrow_color,
                    egui::Stroke::NONE,
                ));
            } else {
                // Right arrow
                let pts = vec![
                    egui::Pos2::new(arrow_center.x - 2.0, arrow_center.y - 4.0),
                    egui::Pos2::new(arrow_center.x + 3.0, arrow_center.y),
                    egui::Pos2::new(arrow_center.x - 2.0, arrow_center.y + 4.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    arrow_color,
                    egui::Stroke::NONE,
                ));
            }
        }

        // Icon
        let icon_x = text_x + if has_children { 16.0 } else { 4.0 };
        if let Some(icon_char) = node.icon {
            painter.text(
                egui::Pos2::new(icon_x, center_y),
                egui::Align2::LEFT_CENTER,
                icon_char.to_string(),
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(130, 130, 142),
            );
        }

        // Label
        let label_x = icon_x + if node.icon.is_some() { 16.0 } else { 0.0 };
        painter.text(
            egui::Pos2::new(label_x, center_y),
            egui::Align2::LEFT_CENTER,
            &node.label,
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(200, 200, 208),
        );

        // Toggle expand on click
        if row_response.clicked() && has_children {
            ui.memory_mut(|mem| mem.data.insert_temp(id, !expanded));
        }

        // Render children if expanded
        if expanded && has_children {
            for (i, child) in node.children.iter().enumerate() {
                Self::show_node(ui, child, depth + 1, id.with(i), row_height);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ResizableSplit
// ---------------------------------------------------------------------------

/// Axis along which a split divides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// A resizable two-pane split layout.
///
/// Stores the split position (0.0–1.0 fraction) in egui memory.
/// The user drags the divider to resize.
///
/// # Example
/// ```rust,ignore
/// ResizableSplit::new("my_split", SplitAxis::Horizontal)
///     .initial_fraction(0.25)
///     .min_size(80.0)
///     .show(ui, |left_ui| {
///         left_ui.label("Left pane");
///     }, |right_ui| {
///         right_ui.label("Right pane");
///     });
/// ```
pub struct ResizableSplit {
    id: egui::Id,
    axis: SplitAxis,
    initial_fraction: f32,
    min_size: f32,
    divider_width: f32,
    divider_color: Option<egui::Color32>,
}

impl ResizableSplit {
    pub fn new(id: impl std::hash::Hash, axis: SplitAxis) -> Self {
        Self {
            id: egui::Id::new(id),
            axis,
            initial_fraction: 0.5,
            min_size: 40.0,
            divider_width: 4.0,
            divider_color: None,
        }
    }

    pub fn initial_fraction(mut self, f: f32) -> Self {
        self.initial_fraction = f;
        self
    }

    pub fn min_size(mut self, px: f32) -> Self {
        self.min_size = px;
        self
    }

    pub fn divider_width(mut self, w: f32) -> Self {
        self.divider_width = w;
        self
    }

    pub fn divider_color(mut self, c: egui::Color32) -> Self {
        self.divider_color = Some(c);
        self
    }

    pub fn show<L, R>(self, ui: &mut egui::Ui, left_or_top: L, right_or_bottom: R)
    where
        L: FnOnce(&mut egui::Ui),
        R: FnOnce(&mut egui::Ui),
    {
        let rect = ui.available_rect_before_wrap();
        let frac_id = self.id.with("__split_frac");

        // Load or initialize fraction from memory - get ctx in a sub-scope
        let fraction: f32 = {
            let ctx = ui.ctx();
            ctx.memory(|m| m.data.get_temp(frac_id))
                .unwrap_or(self.initial_fraction)
        };

        let (left_rect, divider_rect, right_rect) = match self.axis {
            SplitAxis::Horizontal => {
                let left_w = rect.width() * fraction - self.divider_width / 2.0;
                let left = Rect::from_min_size(rect.min, Vec2::new(left_w, rect.height()));
                let divider = Rect::from_min_size(
                    Pos2::new(rect.min.x + left_w, rect.min.y),
                    Vec2::new(self.divider_width, rect.height()),
                );
                let right = Rect::from_min_size(
                    Pos2::new(divider.right(), rect.min.y),
                    Vec2::new(rect.right() - divider.right(), rect.height()),
                );
                (left, divider, right)
            }
            SplitAxis::Vertical => {
                let top_h = rect.height() * fraction - self.divider_width / 2.0;
                let top = Rect::from_min_size(rect.min, Vec2::new(rect.width(), top_h));
                let divider = Rect::from_min_size(
                    Pos2::new(rect.min.x, rect.min.y + top_h),
                    Vec2::new(rect.width(), self.divider_width),
                );
                let bottom = Rect::from_min_size(
                    Pos2::new(rect.min.x, divider.bottom()),
                    Vec2::new(rect.width(), rect.bottom() - divider.bottom()),
                );
                (top, divider, bottom)
            }
        };

        // Allocate divider for drag interaction
        let divider_response = ui.allocate_rect(divider_rect, Sense::drag());

        // Handle divider drag - store new fraction if needed
        let new_fraction = if divider_response.dragged() {
            let delta = divider_response.drag_delta();
            let total_size = match self.axis {
                SplitAxis::Horizontal => rect.width(),
                SplitAxis::Vertical => rect.height(),
            };
            let delta_frac = match self.axis {
                SplitAxis::Horizontal => delta.x / total_size,
                SplitAxis::Vertical => delta.y / total_size,
            };

            // Calculate new fraction with min_size constraint
            let min_frac = self.min_size / total_size;
            let max_frac = 1.0 - min_frac;
            Some((fraction + delta_frac).clamp(min_frac, max_frac))
        } else {
            None
        };

        // Paint divider
        let painter = ui.painter();
        let div_color = self
            .divider_color
            .unwrap_or_else(|| ui.visuals().widgets.noninteractive.bg_stroke.color);
        painter.rect_filled(divider_rect, 0.0, div_color);

        // Render left/top pane
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
            left_or_top(ui);
        });

        // Render right/bottom pane
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |ui| {
            right_or_bottom(ui);
        });

        // Now update memory with new fraction (after all ui operations are done)
        if let Some(frac) = new_fraction {
            let ctx = ui.ctx();
            ctx.memory_mut(|m| m.data.insert_temp(frac_id, frac));
        }
    }
}

// ---------------------------------------------------------------------------
// Ruler
// ---------------------------------------------------------------------------

/// A horizontal timeline ruler showing bar/beat markers with optional playhead and loop region.
///
/// Designed to sit above a LargeCanvas timeline. Coordinates are in "beats" (logical units).
///
/// # Example
/// ```rust,ignore
/// Ruler::new(pan_zoom, beats_per_bar)
///     .playhead(current_beat)
///     .loop_region(loop_start, loop_end)
///     .height(24.0)
///     .show(ui);
/// ```
pub struct Ruler<'a> {
    pan_zoom: &'a crate::interaction::PanZoom,
    beats_per_bar: u32,
    playhead: Option<f32>,
    loop_start: Option<f32>,
    loop_end: Option<f32>,
    height: f32,
    beat_color: Option<egui::Color32>,
    bar_color: Option<egui::Color32>,
    playhead_color: Option<egui::Color32>,
    loop_color: Option<egui::Color32>,
}

impl<'a> Ruler<'a> {
    pub fn new(pan_zoom: &'a crate::interaction::PanZoom, beats_per_bar: u32) -> Self {
        Self {
            pan_zoom,
            beats_per_bar,
            playhead: None,
            loop_start: None,
            loop_end: None,
            height: 24.0,
            beat_color: None,
            bar_color: None,
            playhead_color: None,
            loop_color: None,
        }
    }

    pub fn playhead(mut self, beat: f32) -> Self {
        self.playhead = Some(beat);
        self
    }

    pub fn loop_region(mut self, start: f32, end: f32) -> Self {
        self.loop_start = Some(start);
        self.loop_end = Some(end);
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn beat_color(mut self, c: egui::Color32) -> Self {
        self.beat_color = Some(c);
        self
    }

    pub fn bar_color(mut self, c: egui::Color32) -> Self {
        self.bar_color = Some(c);
        self
    }

    pub fn playhead_color(mut self, c: egui::Color32) -> Self {
        self.playhead_color = Some(c);
        self
    }

    pub fn loop_color(mut self, c: egui::Color32) -> Self {
        self.loop_color = Some(c);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let available_rect = ui.available_rect_before_wrap();
        let rect = Rect::from_min_size(
            available_rect.min,
            Vec2::new(available_rect.width(), self.height),
        );

        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter();

        // Background
        let bg_color = ui.visuals().widgets.noninteractive.bg_fill;
        painter.rect_filled(rect, 0.0, bg_color);

        let pixels_per_beat = self.pan_zoom.scale * 80.0;
        let origin_x = rect.min.x;

        // Calculate visible beat range
        let start_beat = -self.pan_zoom.offset.x / pixels_per_beat;
        let end_beat = start_beat + rect.width() / pixels_per_beat;

        // Draw loop region if set
        if let (Some(loop_start), Some(loop_end)) = (self.loop_start, self.loop_end) {
            let loop_color = self
                .loop_color
                .unwrap_or(egui::Color32::from_rgba_unmultiplied(100, 100, 200, 80));
            let loop_x1 = origin_x + (loop_start - start_beat) * pixels_per_beat;
            let loop_x2 = origin_x + (loop_end - start_beat) * pixels_per_beat;
            let loop_rect = Rect::from_min_max(
                Pos2::new(loop_x1.max(rect.min.x), rect.min.y),
                Pos2::new(loop_x2.min(rect.max.x), rect.max.y),
            );
            painter.rect_filled(loop_rect, 0.0, loop_color);
        }

        // Draw beat lines
        let beat_color = self.beat_color.unwrap_or(egui::Color32::from_gray(120));
        let bar_color = self.bar_color.unwrap_or(egui::Color32::from_gray(200));

        let first_beat = start_beat.floor() as i32;
        let last_beat = end_beat.ceil() as i32;

        for beat in first_beat..=last_beat {
            let beat_f = beat as f32;
            let x = origin_x + (beat_f - start_beat) * pixels_per_beat;

            if x < rect.min.x || x > rect.max.x {
                continue;
            }

            let is_bar = beat_f >= 0.0 && beat_f as u32 % self.beats_per_bar == 0;
            let line_height = if is_bar {
                rect.height()
            } else {
                rect.height() * 0.4
            };
            let y_top = rect.max.y - line_height;

            let color = if is_bar { bar_color } else { beat_color };
            let stroke = Stroke::new(1.0, color);
            painter.line_segment([Pos2::new(x, y_top), Pos2::new(x, rect.max.y)], stroke);

            // Draw bar number
            if is_bar && beat_f >= 0.0 {
                let bar_num = (beat_f / self.beats_per_bar as f32).floor() + 1.0;
                let text = format!("{}", bar_num as i32);
                let font_id = egui::FontId::proportional(10.0);
                painter.text(
                    Pos2::new(x + 2.0, rect.min.y),
                    egui::Align2::LEFT_TOP,
                    text,
                    font_id,
                    bar_color,
                );
            }
        }

        // Draw playhead
        if let Some(playhead_beat) = self.playhead {
            let playhead_color = self.playhead_color.unwrap_or(egui::Color32::RED);
            let playhead_x = origin_x + (playhead_beat - start_beat) * pixels_per_beat;

            if playhead_x >= rect.min.x && playhead_x <= rect.max.x {
                let stroke = Stroke::new(1.0, playhead_color);
                painter.line_segment(
                    [
                        Pos2::new(playhead_x, rect.min.y),
                        Pos2::new(playhead_x, rect.max.y),
                    ],
                    stroke,
                );
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// TimelineClip
// ---------------------------------------------------------------------------

/// Type of timeline clip, determines visual style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipKind {
    Pattern,
    Audio,
    Automation,
}

impl ClipKind {
    fn default_color(self) -> egui::Color32 {
        match self {
            ClipKind::Pattern => egui::Color32::from_rgb(60, 180, 100),
            ClipKind::Audio => egui::Color32::from_rgb(150, 80, 200),
            ClipKind::Automation => egui::Color32::from_rgb(80, 140, 200),
        }
    }
}

/// A draggable, resizable clip on a timeline grid.
///
/// Renders a colored clip rectangle with a header bar, label, and resize handles.
/// Drag the body to move, drag the left/right edges to resize.
///
/// # Example
/// ```rust,ignore
/// let response = TimelineClip::new("clip_1", clip_start, clip_length)
///     .kind(ClipKind::Audio)
///     .label("Kick Loop")
///     .color(Color32::from_rgb(120, 60, 180))
///     .show(ui, &culler);
/// if response.drag_delta() != Vec2::ZERO { ... }
/// ```
pub struct TimelineClip<'a> {
    id: egui::Id,
    start: &'a mut f32,
    length: &'a mut f32,
    kind: ClipKind,
    label: Option<String>,
    color: Option<egui::Color32>,
    height: f32,
    pixels_per_unit: f32,
    pan_zoom: Option<&'a crate::interaction::PanZoom>,
    origin: egui::Pos2,
}

impl<'a> TimelineClip<'a> {
    pub fn new(id: impl std::hash::Hash, start: &'a mut f32, length: &'a mut f32) -> Self {
        Self {
            id: egui::Id::new(id),
            start,
            length,
            kind: ClipKind::Pattern,
            label: None,
            color: None,
            height: 60.0,
            pixels_per_unit: 80.0,
            pan_zoom: None,
            origin: egui::Pos2::ZERO,
        }
    }

    pub fn kind(mut self, kind: ClipKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn pixels_per_unit(mut self, ppu: f32) -> Self {
        self.pixels_per_unit = ppu;
        self
    }

    pub fn pan_zoom(mut self, pz: &'a crate::interaction::PanZoom, origin: egui::Pos2) -> Self {
        self.pan_zoom = Some(pz);
        self.origin = origin;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let scale = self.pan_zoom.map(|pz| pz.scale).unwrap_or(1.0);
        let offset = self.pan_zoom.map(|pz| pz.offset).unwrap_or(Vec2::ZERO);

        let screen_x = self.origin.x + (*self.start * self.pixels_per_unit * scale) + offset.x;
        let screen_width = *self.length * self.pixels_per_unit * scale;

        let clip_rect = Rect::from_min_size(
            Pos2::new(screen_x, self.origin.y),
            Vec2::new(screen_width, self.height),
        );

        // Allocate full clip rect for body drag
        let response = ui.allocate_rect(clip_rect, Sense::drag());

        // Handle body drag (move)
        if response.dragged() {
            let delta = response.drag_delta();
            *self.start += delta.x / (self.pixels_per_unit * scale);
            *self.start = self.start.max(0.0);
        }

        // Re-calculate positions after potential start change
        let screen_x = self.origin.x + (*self.start * self.pixels_per_unit * scale) + offset.x;
        let screen_width = *self.length * self.pixels_per_unit * scale;

        let handle_width = 8.0;

        // Left resize handle (allocated FIRST so it takes priority in hit-testing)
        let left_handle_rect = Rect::from_min_size(
            Pos2::new(screen_x, clip_rect.min.y),
            Vec2::new(handle_width, self.height),
        );
        let left_handle = ui.allocate_rect(left_handle_rect, Sense::drag());
        if left_handle.dragged() {
            let delta = left_handle.drag_delta();
            *self.start += delta.x / (self.pixels_per_unit * scale);
            *self.length -= delta.x / (self.pixels_per_unit * scale);
            *self.length = self.length.max(0.25);
        }

        // Right resize handle
        let right_handle_rect = Rect::from_min_size(
            Pos2::new(screen_x + screen_width - handle_width, clip_rect.min.y),
            Vec2::new(handle_width, self.height),
        );
        let right_handle = ui.allocate_rect(right_handle_rect, Sense::drag());
        if right_handle.dragged() {
            let delta = right_handle.drag_delta();
            *self.length += delta.x / (self.pixels_per_unit * scale);
            *self.length = self.length.max(0.25);
        }

        // Body rect excludes handle zones (shrink by handle_width on each side)
        let body_rect = Rect::from_min_size(
            Pos2::new(screen_x + handle_width, clip_rect.min.y),
            Vec2::new(screen_width - handle_width * 2.0, self.height),
        );

        // Allocate full clip rect for body drag
        let response = ui.allocate_rect(body_rect, Sense::drag());

        // Handle body drag (move)
        if response.dragged() {
            let delta = response.drag_delta();
            *self.start += delta.x / (self.pixels_per_unit * scale);
            *self.start = self.start.max(0.0);
        }

        // Re-calculate positions after potential start change
        let _screen_x = self.origin.x + (*self.start * self.pixels_per_unit * scale) + offset.x;
        let _screen_width = *self.length * self.pixels_per_unit * scale;

        let painter = ui.painter();

        // Get clip color
        let fill_color = self.color.unwrap_or_else(|| self.kind.default_color());
        let header_height = 12.0;
        let header_color = egui::Color32::from_rgba_unmultiplied(
            fill_color.r() / 2,
            fill_color.g() / 2,
            fill_color.b() / 2,
            fill_color.a(),
        );

        // Draw clip body
        painter.rect_filled(clip_rect, 2.0, fill_color);

        // Draw header bar
        let header_rect =
            Rect::from_min_size(clip_rect.min, Vec2::new(clip_rect.width(), header_height));
        painter.rect_filled(header_rect, egui::CornerRadius::ZERO, header_color);

        // Draw label
        if let Some(label_text) = &self.label {
            let font_id = egui::FontId::proportional(10.0);
            painter.text(
                Pos2::new(header_rect.min.x + 4.0, header_rect.min.y),
                egui::Align2::LEFT_TOP,
                label_text.clone(),
                font_id,
                egui::Color32::WHITE,
            );
        }

        // Draw resize handles (lighter strip on edges) - only on hover
        // For simplicity, we always draw subtle handle indicators
        let handle_strip_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40);
        painter.rect_filled(
            Rect::from_min_size(left_handle_rect.min, Vec2::new(2.0, self.height)),
            0.0,
            handle_strip_color,
        );
        painter.rect_filled(
            Rect::from_min_size(
                Pos2::new(right_handle_rect.right() - 2.0, right_handle_rect.min.y),
                Vec2::new(2.0, self.height),
            ),
            0.0,
            handle_strip_color,
        );

        // Draw border
        painter.rect_stroke(
            clip_rect,
            2.0,
            Stroke::new(1.0, fill_color),
            egui::StrokeKind::Outside,
        );

        response
    }
}

// ---------------------------------------------------------------------------
// ChannelStrip
// ---------------------------------------------------------------------------

/// A complete vertical mixer channel strip.
///
/// Composes: color bar → channel name → mute/solo dots → pan knob → fader with VU meter → level readout → channel index badge.
///
/// # Example
/// ```rust,ignore
/// ChannelStrip::new("ch1", &mut volume, &mut pan, &mut mute_state, &mut solo_state)
///     .name("Kick")
///     .color(Color32::from_rgb(60, 180, 120))
///     .meter_level(0.7)
///     .stereo_meter(0.7, 0.65)
///     .index(1)
///     .width(56.0)
///     .show(ui);
/// ```
pub struct ChannelStrip<'a> {
    id: egui::Id,
    volume: &'a mut f64,
    pan: &'a mut f64,
    mute_state: &'a mut DotState,
    solo_state: &'a mut DotState,
    name: Option<String>,
    color: egui::Color32,
    meter_l: f32,
    meter_r: Option<f32>,
    index: Option<u32>,
    width: f32,
}

impl<'a> ChannelStrip<'a> {
    /// Create a new ChannelStrip.
    pub fn new(
        id: impl std::hash::Hash,
        volume: &'a mut f64,
        pan: &'a mut f64,
        mute_state: &'a mut DotState,
        solo_state: &'a mut DotState,
    ) -> Self {
        Self {
            id: egui::Id::new(id),
            volume,
            pan,
            mute_state,
            solo_state,
            name: None,
            color: egui::Color32::from_rgb(60, 60, 80),
            meter_l: 0.0,
            meter_r: None,
            index: None,
            width: 56.0,
        }
    }

    /// Set the channel name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the channel color (top bar).
    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    /// Set mono meter level (0.0 to 1.0).
    pub fn meter_level(mut self, level: f32) -> Self {
        self.meter_l = level.clamp(0.0, 1.0);
        self
    }

    /// Set stereo meter levels (0.0 to 1.0).
    pub fn stereo_meter(mut self, l: f32, r: f32) -> Self {
        self.meter_l = l.clamp(0.0, 1.0);
        self.meter_r = Some(r.clamp(0.0, 1.0));
        self
    }

    /// Set the channel index badge.
    pub fn index(mut self, idx: u32) -> Self {
        self.index = Some(idx);
        self
    }

    /// Set the channel strip width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Render the channel strip.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let w = self.width;

        ui.vertical(|ui| {
            // Color bar at top
            let (bar_rect, _) =
                ui.allocate_exact_size(egui::Vec2::new(w, 4.0), egui::Sense::hover());
            ui.painter().rect_filled(
                egui::Rect::from_min_size(bar_rect.min, egui::Vec2::new(w, 4.0)),
                0.0,
                self.color,
            );

            // Channel name
            let name_text = self.name.as_deref().unwrap_or("");
            ui.add_sized([w, 20.0], egui::Label::new(name_text).truncate());

            // Mute/Solo row
            ui.horizontal(|ui| {
                ui.add_sized(
                    [w * 0.5, 12.0],
                    ToggleDot::new(self.id.with("mute"), self.mute_state).size(10.0),
                );
                ui.add_sized(
                    [w * 0.5, 12.0],
                    ToggleDot::new(self.id.with("solo"), self.solo_state).size(10.0),
                );
            });

            // Spacing
            ui.add_sized([w, 4.0], egui::Label::new(""));

            // Pan knob
            ui.add(Knob::new(self.pan, -1.0..=1.0).size(32.0).label("PAN"));
        });

        // Fader with meter
        let fader_height = 120.0;
        let fader_size = egui::Vec2::new(w - 8.0, fader_height);
        let fader_rect = egui::Rect::from_min_size(ui.cursor().min, fader_size);

        ui.painter().rect_filled(
            fader_rect.expand(4.0),
            0.0,
            egui::Color32::from_rgb(20, 20, 28),
        );

        let mut fader =
            Fader::new(self.volume, 0.0..=1.0).size(egui::Vec2::new(w - 8.0, fader_height));
        if let Some(r) = self.meter_r {
            fader = fader.stereo_meter(self.meter_l, r);
        } else {
            fader = fader.meter_value(self.meter_l);
        }
        ui.add(fader);

        // Level readout (dB)
        let level = (self.meter_l as f64).max(1e-6); // -120 dB floor
        let db = 20.0 * level.log10();
        let db_text = if db < -96.0 {
            "-∞ dB".to_string()
        } else {
            format!("{:.1} dB", db)
        };
        ui.add_sized(
            [w, 16.0],
            egui::Label::new(egui::RichText::new(db_text).size(10.0)),
        );

        // Index badge
        if let Some(idx) = self.index {
            let badge_text = format!("{}", idx);
            let text_size = 14.0_f32; // approximate badge width
            let badge_rect = egui::Rect::from_min_size(
                egui::Pos2::new(fader_rect.min.x, fader_rect.max.y + 4.0),
                egui::Vec2::new(text_size + 6.0, 14.0),
            );
            ui.painter()
                .rect_filled(badge_rect, 4.0, egui::Color32::from_rgb(50, 50, 60));
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                badge_text,
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(180, 180, 190),
            );
        }

        ui.allocate_rect(
            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(w, 200.0)),
            egui::Sense::hover(),
        )
    }
}

// ---------------------------------------------------------------------------
// Waveform
// ---------------------------------------------------------------------------

/// Renders audio waveform data as a filled shape.
///
/// Accepts a slice of normalized sample values (-1.0 to 1.0) and renders them
/// as a filled waveform within the given rect.
///
/// # Example
/// ```rust,ignore
/// Waveform::new(&samples)
///     .color(Color32::from_rgb(100, 200, 255))
///     .filled(true)
///     .show(ui, rect);
/// ```
pub struct Waveform<'a> {
    samples: &'a [f32],
    color: egui::Color32,
    filled: bool,
    line_width: f32,
    background: Option<egui::Color32>,
}

impl<'a> Waveform<'a> {
    /// Create a new Waveform widget.
    pub fn new(samples: &'a [f32]) -> Self {
        Self {
            samples,
            color: egui::Color32::from_rgb(100, 200, 255),
            filled: true,
            line_width: 1.5,
            background: None,
        }
    }

    /// Set the waveform color.
    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    /// Set whether to fill the waveform (true) or draw outline only (false).
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Set the line width for outline mode.
    pub fn line_width(mut self, w: f32) -> Self {
        self.line_width = w;
        self
    }

    /// Set a background color.
    pub fn background(mut self, color: egui::Color32) -> Self {
        self.background = Some(color);
        self
    }

    /// Render the waveform into the given rect. Does NOT allocate UI space — caller provides rect.
    pub fn paint(self, painter: &egui::Painter, rect: egui::Rect) {
        if self.samples.is_empty() {
            return;
        }

        let width = rect.width() as usize;
        let height = rect.height();
        let center_y = rect.center().y;

        // Background
        if let Some(bg) = self.background {
            painter.rect_filled(rect, 0.0, bg);
        }

        // Resample to match pixel width
        let step = self.samples.len() as f32 / width as f32;

        if self.filled {
            // Build filled polygon: top half left-to-right, bottom half right-to-left
            let mut top_points: Vec<egui::Pos2> = Vec::with_capacity(width);
            let mut bottom_points: Vec<egui::Pos2> = Vec::with_capacity(width);

            for i in 0..width {
                let sample_idx = (i as f32 * step) as usize;
                let sample_idx = sample_idx.min(self.samples.len() - 1);
                let sample = self.samples[sample_idx].clamp(-1.0, 1.0);

                let x = rect.min.x + i as f32;
                let top_y = center_y - sample.abs() * height * 0.5;
                let bottom_y = center_y + sample.abs() * height * 0.5;

                top_points.push(egui::Pos2::new(x, top_y));
                bottom_points.push(egui::Pos2::new(x, bottom_y));
            }

            // Build full polygon: top + reversed bottom
            let mut points = top_points.clone();
            points.extend(bottom_points.clone().into_iter().rev());

            painter.add(egui::Shape::Path(egui::epaint::PathShape {
                points,
                closed: true,
                fill: self.color.linear_multiply(0.8),
                stroke: egui::epaint::PathStroke::NONE,
            }));

            // Draw outline on top
            if !top_points.is_empty() {
                painter.add(egui::Shape::line(
                    top_points.clone(),
                    egui::Stroke::new(self.line_width, self.color),
                ));
                let reversed_bottom: Vec<_> = bottom_points.into_iter().rev().collect();
                painter.add(egui::Shape::line(
                    reversed_bottom,
                    egui::Stroke::new(self.line_width, self.color),
                ));
            }
        } else {
            // Draw as polyline
            let mut points: Vec<egui::Pos2> = Vec::with_capacity(width);
            for i in 0..width {
                let sample_idx = (i as f32 * step) as usize;
                let sample_idx = sample_idx.min(self.samples.len() - 1);
                let sample = self.samples[sample_idx].clamp(-1.0, 1.0);

                let x = rect.min.x + i as f32;
                let y = center_y - sample * height * 0.5;

                points.push(egui::Pos2::new(x, y));
            }

            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(self.line_width, self.color),
            ));
        }
    }

    /// Allocate space in the UI and render.
    pub fn show(self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
        self.paint(ui.painter(), rect);
        response
    }
}

// ---------------------------------------------------------------------------
// DragReorder
// ---------------------------------------------------------------------------

/// State for a drag-reorder list.
#[derive(Clone, Debug, Default)]
struct DragReorderState {
    dragging: Option<usize>,
    drag_offset: f32,
    hover_index: Option<usize>,
}

/// A drag-to-reorder list widget.
///
/// Renders a list of items with drag handles. Users can drag items to reorder them.
/// The `items` slice is reordered in-place when a drag completes.
///
/// # Example
/// ```rust,ignore
/// DragReorder::new(ui.id().with("tracks"), &mut self.tracks)
///     .item_height(32.0)
///     .show(ui, |ui, item, _dragging| {
///         ui.label(&item.name);
///     });
/// ```
pub struct DragReorder<'a, T> {
    id: egui::Id,
    items: &'a mut Vec<T>,
    item_height: f32,
}

impl<'a, T: Clone> DragReorder<'a, T> {
    pub fn new(id: impl std::hash::Hash, items: &'a mut Vec<T>) -> Self {
        Self {
            id: egui::Id::new(id),
            items,
            item_height: 32.0,
        }
    }

    pub fn item_height(mut self, h: f32) -> Self {
        self.item_height = h;
        self
    }

    /// Show the reorderable list.
    ///
    /// `render_item` receives `(ui, item, is_dragging)`.
    pub fn show(self, ui: &mut egui::Ui, mut render_item: impl FnMut(&mut egui::Ui, &T, bool)) {
        let n = self.items.len();
        let total_height = n as f32 * self.item_height;
        let (outer_rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(ui.available_width(), total_height),
            egui::Sense::hover(),
        );

        let mut state: DragReorderState = ui
            .ctx()
            .memory(|m| m.data.get_temp(self.id))
            .unwrap_or_default();

        let mut new_order: Option<(usize, usize)> = None;

        for i in 0..n {
            let item_rect = egui::Rect::from_min_size(
                egui::Pos2::new(
                    outer_rect.min.x,
                    outer_rect.min.y + i as f32 * self.item_height,
                ),
                egui::Vec2::new(outer_rect.width(), self.item_height - 1.0),
            );

            // Drag handle (left 20px)
            let handle_rect =
                egui::Rect::from_min_size(item_rect.min, egui::Vec2::new(20.0, self.item_height));
            let handle_resp = ui.interact(
                handle_rect,
                self.id.with(("handle", i)),
                egui::Sense::drag(),
            );

            if handle_resp.drag_started() {
                state.dragging = Some(i);
                state.drag_offset = 0.0;
            }

            if handle_resp.dragged() {
                if state.dragging == Some(i) {
                    state.drag_offset += handle_resp.drag_delta().y;
                    // Compute target index
                    let target_f = i as f32 + state.drag_offset / self.item_height;
                    let target = (target_f.round() as isize).clamp(0, n as isize - 1) as usize;
                    state.hover_index = Some(target);
                }
            }

            if handle_resp.drag_stopped() {
                if let (Some(from), Some(to)) = (state.dragging, state.hover_index) {
                    if from != to {
                        new_order = Some((from, to));
                    }
                }
                state.dragging = None;
                state.drag_offset = 0.0;
                state.hover_index = None;
            }

            let is_dragging = state.dragging == Some(i);

            // Draw item background
            let bg = if is_dragging {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 15)
            } else if handle_resp.hovered() {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8)
            } else {
                egui::Color32::TRANSPARENT
            };
            if bg != egui::Color32::TRANSPARENT {
                ui.painter().rect_filled(item_rect, 4.0, bg);
            }

            // Draw drag handle dots
            let dot_color = egui::Color32::from_gray(80);
            for row in 0..3 {
                for col in 0..2 {
                    let dot_pos = egui::Pos2::new(
                        handle_rect.center().x - 3.0 + col as f32 * 6.0,
                        handle_rect.center().y - 4.0 + row as f32 * 4.0,
                    );
                    ui.painter().circle_filled(dot_pos, 1.5, dot_color);
                }
            }

            // Drop indicator line
            if let Some(target) = state.hover_index {
                if target == i && state.dragging != Some(i) {
                    let line_y = item_rect.min.y;
                    ui.painter().line_segment(
                        [
                            egui::Pos2::new(outer_rect.min.x, line_y),
                            egui::Pos2::new(outer_rect.max.x, line_y),
                        ],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255)),
                    );
                }
            }

            // Render item content (offset 20px for handle)
            let content_rect = egui::Rect::from_min_max(
                egui::Pos2::new(item_rect.min.x + 20.0, item_rect.min.y),
                item_rect.max,
            );
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
            render_item(&mut child, &self.items[i], is_dragging);
        }

        // Apply reorder
        if let Some((from, to)) = new_order {
            let item = self.items.remove(from);
            self.items.insert(to, item);
        }

        ui.ctx().memory_mut(|m| m.data.insert_temp(self.id, state));
    }
}

// ---------------------------------------------------------------------------
// VerticalDrag
// ---------------------------------------------------------------------------

/// A vertical drag widget for continuous value editing.
///
/// Renders a thin vertical bar that the user drags up/down to change a value.
/// Similar to a fader but minimal — just the drag zone, no visual track.
///
/// # Example
/// ```rust,ignore
/// ui.add(VerticalDrag::new(&mut self.value, 0.0..=1.0).height(80.0).label("VOL"));
/// ```
pub struct VerticalDrag<'a> {
    value: &'a mut f64,
    range: std::ops::RangeInclusive<f64>,
    height: f32,
    width: f32,
    label: Option<String>,
    default_value: Option<f64>,
    sensitivity: f64,
}

impl<'a> VerticalDrag<'a> {
    pub fn new(value: &'a mut f64, range: std::ops::RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            height: 80.0,
            width: 16.0,
            label: None,
            default_value: None,
            sensitivity: 1.0,
        }
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }
    pub fn sensitivity(mut self, s: f64) -> Self {
        self.sensitivity = s;
        self
    }
}

impl egui::Widget for VerticalDrag<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired = egui::Vec2::new(self.width, self.height);
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());

        if response.dragged() {
            let range_size = *self.range.end() - *self.range.start();
            let pixels_per_unit = self.height as f64 / range_size;
            let fine = ui.input(|i| i.modifiers.shift);
            let factor = if fine { 0.1 } else { 1.0 };
            let delta =
                -response.drag_delta().y as f64 / pixels_per_unit * factor * self.sensitivity;
            *self.value = (*self.value + delta).clamp(*self.range.start(), *self.range.end());
        }

        if response.double_clicked() {
            if let Some(def) = self.default_value {
                *self.value = def;
            }
        }

        if ui.is_rect_visible(rect) {
            let t = ((*self.value - *self.range.start())
                / (*self.range.end() - *self.range.start())) as f32;
            let t = t.clamp(0.0, 1.0);

            // Track
            let track_color = egui::Color32::from_gray(35);
            ui.painter().rect_filled(rect, 3.0, track_color);

            // Fill
            let fill_height = t * rect.height();
            let fill_rect = egui::Rect::from_min_max(
                egui::Pos2::new(rect.min.x, rect.max.y - fill_height),
                rect.max,
            );
            let fill_color = if response.hovered() || response.dragged() {
                egui::Color32::from_rgb(100, 160, 255)
            } else {
                egui::Color32::from_rgb(70, 120, 200)
            };
            ui.painter().rect_filled(fill_rect, 3.0, fill_color);

            // Notch at current value
            let notch_y = rect.max.y - fill_height;
            ui.painter().line_segment(
                [
                    egui::Pos2::new(rect.min.x, notch_y),
                    egui::Pos2::new(rect.max.x, notch_y),
                ],
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );

            // Label
            if let Some(label) = &self.label {
                ui.painter().text(
                    egui::Pos2::new(rect.center().x, rect.max.y + 4.0),
                    egui::Align2::CENTER_TOP,
                    label,
                    egui::FontId::proportional(9.0),
                    egui::Color32::from_gray(140),
                );
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// DragNumber
// ---------------------------------------------------------------------------

/// A draggable numeric display. Click-drag horizontally to change value.
/// Shift = fine control. Double-click to type. Right-click to reset.
///
/// # Example
/// ```rust,ignore
/// ui.add(DragNumber::new(&mut self.bpm, 60.0..=300.0)
///     .label("BPM")
///     .default_value(120.0)
///     .speed(1.0)
///     .decimals(1));
/// ```
pub struct DragNumber<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    label: Option<String>,
    default_value: Option<f64>,
    speed: f64,
    decimals: usize,
    width: f32,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl<'a> DragNumber<'a> {
    pub fn new(value: &'a mut f64, range: RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            label: None,
            default_value: None,
            speed: 1.0,
            decimals: 0,
            width: 64.0,
            prefix: None,
            suffix: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn default_value(mut self, v: f64) -> Self {
        self.default_value = Some(v);
        self
    }

    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn prefix(mut self, p: impl Into<String>) -> Self {
        self.prefix = Some(p.into());
        self
    }

    pub fn suffix(mut self, s: impl Into<String>) -> Self {
        self.suffix = Some(s.into());
        self
    }
}

impl<'a> egui::Widget for DragNumber<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let height = 22.0_f32;
        let label_height = if self.label.is_some() { 14.0_f32 } else { 0.0 };
        let total_height = height + label_height;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(self.width, total_height), Sense::click_and_drag());

        let id = response.id;
        let is_editing: bool = ui
            .ctx()
            .data(|d| d.get_temp(id.with("editing")).unwrap_or(false));

        // --- Editing mode (text input) ---
        if is_editing {
            let edit_str: String = ui.ctx().data(|d| {
                d.get_temp(id.with("edit_str"))
                    .unwrap_or_else(|| format!("{:.prec$}", *self.value, prec = self.decimals))
            });

            let mut buf = edit_str.clone();
            let text_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, label_height),
                Vec2::new(self.width, height),
            );
            let te_resp = ui.put(
                text_rect,
                egui::TextEdit::singleline(&mut buf)
                    .desired_width(self.width)
                    .font(egui::TextStyle::Monospace),
            );

            ui.ctx()
                .data_mut(|d| d.insert_temp(id.with("edit_str"), buf.clone()));

            if te_resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(v) = buf.trim().parse::<f64>() {
                    *self.value = v.clamp(*self.range.start(), *self.range.end());
                }
                ui.ctx().data_mut(|d| {
                    d.insert_temp(id.with("editing"), false);
                    d.remove::<String>(id.with("edit_str"));
                });
            } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.ctx().data_mut(|d| {
                    d.insert_temp(id.with("editing"), false);
                    d.remove::<String>(id.with("edit_str"));
                });
            }

            return response;
        }

        // --- Drag interaction ---
        if response.double_clicked() {
            let s = format!("{:.prec$}", *self.value, prec = self.decimals);
            ui.ctx().data_mut(|d| {
                d.insert_temp(id.with("editing"), true);
                d.insert_temp(id.with("edit_str"), s);
            });
        } else if response.secondary_clicked() {
            if let Some(def) = self.default_value {
                *self.value = def;
            }
        } else if response.dragged() {
            let delta = response.drag_delta().x as f64;
            let fine = ui.input(|i| i.modifiers.shift);
            let multiplier = if fine { 0.1 } else { 1.0 };
            *self.value = (*self.value + delta * self.speed * multiplier)
                .clamp(*self.range.start(), *self.range.end());
        }

        // --- Paint ---
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let visuals = ui.visuals();

            let hovered = response.hovered();
            let dragging = response.dragged();

            let bg_color = if dragging {
                Color32::from_gray(55)
            } else if hovered {
                Color32::from_gray(45)
            } else {
                Color32::from_gray(35)
            };

            let value_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, label_height),
                Vec2::new(self.width, height),
            );

            // Background
            painter.rect_filled(value_rect, egui::CornerRadius::same(3u8), bg_color);

            // Subtle left accent line when hovered/dragging
            if hovered || dragging {
                painter.rect_filled(
                    Rect::from_min_size(value_rect.min, Vec2::new(2.0, height)),
                    egui::CornerRadius::ZERO,
                    Color32::from_rgb(100, 160, 255),
                );
            }

            // Value text
            let display = {
                let mut s = String::new();
                if let Some(ref p) = self.prefix {
                    s.push_str(p);
                }
                s.push_str(&format!("{:.prec$}", *self.value, prec = self.decimals));
                if let Some(ref sfx) = self.suffix {
                    s.push_str(sfx);
                }
                s
            };

            let text_color = if dragging {
                Color32::WHITE
            } else {
                visuals.text_color()
            };

            painter.text(
                value_rect.center(),
                egui::Align2::CENTER_CENTER,
                &display,
                egui::FontId::monospace(13.0),
                text_color,
            );

            // Label above
            if let Some(ref lbl) = self.label {
                painter.text(
                    Rect::from_min_size(rect.min, Vec2::new(self.width, label_height)).center(),
                    egui::Align2::CENTER_CENTER,
                    lbl.as_str(),
                    egui::FontId::proportional(10.0),
                    Color32::from_gray(140),
                );
            }

            // Drag cursor hint arrows (◀ ▶) when hovered
            if hovered && !dragging {
                let arrow_color = Color32::from_gray(120);
                let cy = value_rect.center().y;
                let lx = value_rect.min.x + 5.0;
                let rx = value_rect.max.x - 5.0;
                // left arrow ◀
                painter.text(
                    Pos2::new(lx, cy),
                    egui::Align2::LEFT_CENTER,
                    "◀",
                    egui::FontId::proportional(8.0),
                    arrow_color,
                );
                // right arrow ▶
                painter.text(
                    Pos2::new(rx, cy),
                    egui::Align2::RIGHT_CENTER,
                    "▶",
                    egui::FontId::proportional(8.0),
                    arrow_color,
                );
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// CollapsePanel
// ---------------------------------------------------------------------------

/// State for CollapsePanel animation.
#[derive(Clone, Debug)]
struct CollapsePanelState {
    open: bool,
    anim_t: f32, // 0.0 = fully closed, 1.0 = fully open
}

impl Default for CollapsePanelState {
    fn default() -> Self {
        Self {
            open: true,
            anim_t: 1.0,
        }
    }
}

/// A collapsible panel with smooth easing animation.
///
/// The panel animates open/close using an ease-in-out curve.
///
/// # Example
/// ```rust,ignore
/// CollapsePanel::new(ui.id().with("settings"), "Settings")
///     .default_open(true)
///     .show(ui, |ui| {
///         ui.label("Panel content");
///     });
/// ```
pub struct CollapsePanel<'a> {
    id: egui::Id,
    title: &'a str,
    default_open: bool,
    header_height: f32,
    animation_duration: f32,
}

impl<'a> CollapsePanel<'a> {
    pub fn new(id: impl std::hash::Hash, title: &'a str) -> Self {
        Self {
            id: egui::Id::new(id),
            title,
            default_open: true,
            header_height: 32.0,
            animation_duration: 0.2,
        }
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
    pub fn header_height(mut self, h: f32) -> Self {
        self.header_height = h;
        self
    }
    pub fn animation_duration(mut self, secs: f32) -> Self {
        self.animation_duration = secs;
        self
    }

    /// Show the collapsible panel.
    ///
    /// Returns `true` if the panel is currently open (or animating open).
    pub fn show(self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) -> bool {
        let ctx = ui.ctx().clone();

        let mut state: CollapsePanelState = ctx
            .memory(|m| m.data.get_temp(self.id))
            .unwrap_or_else(|| CollapsePanelState {
                open: self.default_open,
                anim_t: if self.default_open { 1.0 } else { 0.0 },
            });

        // Advance animation
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        let speed = 1.0 / self.animation_duration.max(0.05);
        if state.open {
            state.anim_t = (state.anim_t + dt * speed).min(1.0);
        } else {
            state.anim_t = (state.anim_t - dt * speed).max(0.0);
        }

        // Ease-in-out curve
        let t = state.anim_t;
        let eased = t * t * (3.0 - 2.0 * t); // smoothstep

        if state.anim_t > 0.0 && state.anim_t < 1.0 {
            ctx.request_repaint();
        }

        // Header
        let header_rect = ui
            .allocate_space(egui::Vec2::new(ui.available_width(), self.header_height))
            .1;
        let header_resp = ui.interact(header_rect, self.id.with("header"), egui::Sense::click());

        if header_resp.clicked() {
            state.open = !state.open;
        }

        if ui.is_rect_visible(header_rect) {
            // Background
            let bg = if header_resp.hovered() {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10)
            } else {
                egui::Color32::from_gray(28)
            };
            ui.painter().rect_filled(header_rect, 4.0, bg);

            // Arrow (rotates with animation)
            let arrow_angle = eased * std::f32::consts::FRAC_PI_2; // 0 = right, π/2 = down
            let arrow_center = egui::Pos2::new(header_rect.min.x + 16.0, header_rect.center().y);
            let arrow_size = 5.0_f32;
            // Draw a simple triangle arrow
            let cos_a = arrow_angle.cos();
            let sin_a = arrow_angle.sin();
            let pts = [
                egui::Pos2::new(
                    arrow_center.x + cos_a * arrow_size - sin_a * 0.0,
                    arrow_center.y + sin_a * arrow_size + cos_a * 0.0,
                ),
                egui::Pos2::new(
                    arrow_center.x + cos_a * (-arrow_size * 0.5) - sin_a * (-arrow_size * 0.8),
                    arrow_center.y + sin_a * (-arrow_size * 0.5) + cos_a * (-arrow_size * 0.8),
                ),
                egui::Pos2::new(
                    arrow_center.x + cos_a * (-arrow_size * 0.5) - sin_a * (arrow_size * 0.8),
                    arrow_center.y + sin_a * (-arrow_size * 0.5) + cos_a * (arrow_size * 0.8),
                ),
            ];
            let arrow_color = egui::Color32::from_gray(160);
            ui.painter().add(egui::Shape::convex_polygon(
                pts.to_vec(),
                arrow_color,
                egui::Stroke::NONE,
            ));

            // Title
            ui.painter().text(
                egui::Pos2::new(header_rect.min.x + 32.0, header_rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.title,
                egui::FontId::proportional(13.0),
                egui::Color32::from_gray(220),
            );
        }

        // Content (clipped to animated height)
        let is_visible = eased > 0.001;
        if is_visible {
            // Measure content height by rendering into a hidden UI first
            // For simplicity, use a clip rect that scales with eased
            let available_width = ui.available_width();
            // Allocate a region and clip it
            let content_start = ui.cursor().min;

            // Use a child UI with clipping
            let max_content_height = 2000.0; // generous max
            let clip_height = eased * max_content_height;

            let child_rect = egui::Rect::from_min_size(
                content_start,
                egui::Vec2::new(available_width, clip_height),
            );

            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(*ui.layout()),
            );
            add_contents(&mut child_ui);
            let content_height = child_ui.min_rect().height();

            // Advance the parent cursor by the actual (eased) height
            let actual_height = (eased * content_height).min(clip_height);
            ui.allocate_space(egui::Vec2::new(available_width, actual_height));
        }

        ctx.memory_mut(|m| m.data.insert_temp(self.id, state));
        is_visible
    }
}
