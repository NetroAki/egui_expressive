//! Border and radius utility methods for `Tw`.

use egui::{Color32, Rect, Stroke, Ui};

use crate::tailwind::builder::Tw;
use crate::tailwind::types::RadiusCorners;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct BorderSide {
    pub width: f32,
    pub color: Option<Color32>,
}

impl BorderSide {
    pub fn width(width: f32) -> Self {
        Self { width, color: None }
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct BorderEdges {
    pub top: BorderSide,
    pub right: BorderSide,
    pub bottom: BorderSide,
    pub left: BorderSide,
}

impl BorderEdges {
    pub fn is_empty(self) -> bool {
        self.top.width <= 0.0
            && self.right.width <= 0.0
            && self.bottom.width <= 0.0
            && self.left.width <= 0.0
    }

    pub fn paint(self, ui: &Ui, rect: Rect, fallback: Color32) {
        let painter = ui.painter();
        if self.top.width > 0.0 {
            painter.line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(self.top.width, self.top.color.unwrap_or(fallback)),
            );
        }
        if self.right.width > 0.0 {
            painter.line_segment(
                [rect.right_top(), rect.right_bottom()],
                Stroke::new(self.right.width, self.right.color.unwrap_or(fallback)),
            );
        }
        if self.bottom.width > 0.0 {
            painter.line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(self.bottom.width, self.bottom.color.unwrap_or(fallback)),
            );
        }
        if self.left.width > 0.0 {
            painter.line_segment(
                [rect.left_top(), rect.left_bottom()],
                Stroke::new(self.left.width, self.left.color.unwrap_or(fallback)),
            );
        }
    }
}

impl Tw {
    pub fn rounded(mut self, r: f32) -> Self {
        self.border_radius = r;
        self.radius_corners = RadiusCorners::same(r);
        self
    }

    pub fn rounded_none(mut self) -> Self {
        self.border_radius = 0.0;
        self.radius_corners = RadiusCorners::default();
        self
    }

    pub fn rounded_sm(mut self) -> Self {
        self.border_radius = 2.0;
        self
    }

    pub fn rounded_md(mut self) -> Self {
        self.border_radius = 4.0;
        self
    }

    pub fn rounded_lg(mut self) -> Self {
        self.border_radius = 8.0;
        self
    }

    pub fn rounded_xl(mut self) -> Self {
        self.border_radius = 12.0;
        self
    }

    pub fn rounded_2xl(mut self) -> Self {
        self.border_radius = 16.0;
        self
    }

    pub fn rounded_full(mut self) -> Self {
        self.border_radius = 9999.0;
        self.radius_corners = RadiusCorners::same(9999.0);
        self
    }

    pub fn rounded_t(mut self, r: f32) -> Self {
        self.radius_corners.nw = r;
        self.radius_corners.ne = r;
        self
    }

    pub fn rounded_b(mut self, r: f32) -> Self {
        self.radius_corners.sw = r;
        self.radius_corners.se = r;
        self
    }

    pub fn rounded_l(mut self, r: f32) -> Self {
        self.radius_corners.nw = r;
        self.radius_corners.sw = r;
        self
    }

    pub fn rounded_r(mut self, r: f32) -> Self {
        self.radius_corners.ne = r;
        self.radius_corners.se = r;
        self
    }

    pub fn border(mut self, w: f32) -> Self {
        self.border_width = w;
        self
    }

    pub fn border_1(mut self) -> Self {
        self.border_width = 1.0;
        self
    }

    pub fn border_2(mut self) -> Self {
        self.border_width = 2.0;
        self
    }

    pub fn border_t(mut self, w: f32) -> Self {
        self.border_edges.top = BorderSide::width(w);
        self
    }

    pub fn border_r(mut self, w: f32) -> Self {
        self.border_edges.right = BorderSide::width(w);
        self
    }

    pub fn border_b(mut self, w: f32) -> Self {
        self.border_edges.bottom = BorderSide::width(w);
        self
    }

    pub fn border_l(mut self, w: f32) -> Self {
        self.border_edges.left = BorderSide::width(w);
        self
    }

    pub fn border_x(mut self, w: f32) -> Self {
        self.border_edges.left = BorderSide::width(w);
        self.border_edges.right = BorderSide::width(w);
        self
    }

    pub fn border_y(mut self, w: f32) -> Self {
        self.border_edges.top = BorderSide::width(w);
        self.border_edges.bottom = BorderSide::width(w);
        self
    }

    pub fn border_l_color(mut self, color: Color32) -> Self {
        self.border_edges.left = self.border_edges.left.color(color);
        self
    }

    pub fn border_r_color(mut self, color: Color32) -> Self {
        self.border_edges.right = self.border_edges.right.color(color);
        self
    }

    pub fn border_t_color(mut self, color: Color32) -> Self {
        self.border_edges.top = self.border_edges.top.color(color);
        self
    }

    pub fn border_b_color(mut self, color: Color32) -> Self {
        self.border_edges.bottom = self.border_edges.bottom.color(color);
        self
    }
}
