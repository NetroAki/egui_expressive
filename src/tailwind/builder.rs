//! Public `Tw` utility-style builder.

use egui::{Color32, CursorIcon, Vec2};

use crate::layout::{GridLayout, PositionStyle};
use crate::tailwind::border::BorderEdges;
use crate::tailwind::spacing::Edges;
use crate::tailwind::types::{
    Display, FlexDirection, FontWeight, Items, Justify, Overflow, RadiusCorners, SelectionStyle,
    Size, TwBackdropSource, TwDropShadow, TwGradient, TwRing, TwTransition,
};
use crate::theme::Elevation;

/// Tailwind/CSS-recognizable style builder for egui frames.
#[derive(Clone, Default, Debug)]
pub struct Tw {
    /// CSS margin (`m-*`, `mx-*`, `my-*`, etc.).
    pub margin: Edges,
    /// CSS padding (`p-*`, `px-*`, `py-*`, etc.).
    pub padding: Edges,

    pub bg: Option<Color32>,
    pub fg: Option<Color32>,
    pub bg_token: Option<crate::tailwind::theme_tokens::ColorToken>,
    pub fg_token: Option<crate::tailwind::theme_tokens::ColorToken>,
    pub border_color: Option<Color32>,

    /// Note: `font_size` must be applied per-widget using `RichText::new(text).size(size)`.
    pub font_size: Option<f32>,
    pub font_weight: FontWeight,
    /// Built-in egui font family alias (`sans`/`mono`) for exact Phase 8 family selection.
    pub font_family: Option<&'static str>,
    /// Note: `letter_spacing` must be applied per-widget using `RichText::new(text).letter_spacing(spacing)`.
    pub letter_spacing: Option<f32>,

    pub width: Size,
    pub height: Size,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<Size>,
    pub max_height: Option<Size>,

    pub border_width: f32,
    pub border_radius: f32,
    pub radius_corners: RadiusCorners,
    pub border_edges: BorderEdges,

    pub opacity: f32,
    pub elevation: Option<Elevation>,

    pub display: Display,
    pub flex_direction: FlexDirection,
    pub justify: Justify,
    pub items: Items,
    pub overflow: Overflow,
    pub cursor: Option<CursorIcon>,
    pub pointer_events: bool,

    pub gap: Option<Vec2>,
    pub space: Option<Vec2>,
    pub divide: Option<Vec2>,
    pub flex_wrap: bool,
    pub grid: Option<GridLayout>,
    pub col_span: Option<usize>,
    pub row_span: Option<usize>,

    pub position: PositionStyle,
    pub id: Option<egui::Id>,

    pub gradient: Option<TwGradient>,
    pub backdrop_blur: Option<f32>,
    pub backdrop_source: TwBackdropSource,
    pub drop_shadow: Option<TwDropShadow>,
    pub aspect_ratio: Option<f32>,
    pub ring: Option<TwRing>,
    pub transition: Option<TwTransition>,
    pub selection: Option<SelectionStyle>,
}

impl Tw {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            pointer_events: true,
            ..Default::default()
        }
    }

    /// Set a stable id for positioned areas or grid containers.
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Some(egui::Id::new(id));
        self
    }
}
