//! Margin, padding, and compatibility aliases for `Tw`.

use crate::tailwind::{spacing::Edges, types::Size, Tw};

impl Tw {
    pub fn m(mut self, v: f32) -> Self {
        self.margin = Edges::all(v);
        self
    }
    pub fn mx(mut self, v: f32) -> Self {
        self.margin.left = v;
        self.margin.right = v;
        self
    }
    pub fn my(mut self, v: f32) -> Self {
        self.margin.top = v;
        self.margin.bottom = v;
        self
    }
    pub fn mt(mut self, v: f32) -> Self {
        self.margin.top = v;
        self
    }
    pub fn mr(mut self, v: f32) -> Self {
        self.margin.right = v;
        self
    }
    pub fn mb(mut self, v: f32) -> Self {
        self.margin.bottom = v;
        self
    }
    pub fn ml(mut self, v: f32) -> Self {
        self.margin.left = v;
        self
    }

    pub fn p(mut self, v: f32) -> Self {
        self.padding = Edges::all(v);
        self
    }
    pub fn px(mut self, v: f32) -> Self {
        self.padding.left = v;
        self.padding.right = v;
        self
    }
    pub fn py(mut self, v: f32) -> Self {
        self.padding.top = v;
        self.padding.bottom = v;
        self
    }
    pub fn pt(mut self, v: f32) -> Self {
        self.padding.top = v;
        self
    }
    pub fn pb(mut self, v: f32) -> Self {
        self.padding.bottom = v;
        self
    }
    pub fn pl(mut self, v: f32) -> Self {
        self.padding.left = v;
        self
    }
    pub fn pr(mut self, v: f32) -> Self {
        self.padding.right = v;
        self
    }

    pub fn padding(self, v: f32) -> Self {
        self.p(v)
    }
    pub fn corner_radius(self, r: f32) -> Self {
        self.rounded(r)
    }
    pub fn frame_size(mut self, width: Option<f32>, height: Option<f32>) -> Self {
        if let Some(w) = width {
            self.width = Size::Px(w);
        }
        if let Some(h) = height {
            self.height = Size::Px(h);
        }
        self
    }
    pub fn frame_constraints(
        mut self,
        min_w: Option<f32>,
        max_w: Option<f32>,
        min_h: Option<f32>,
        max_h: Option<f32>,
    ) -> Self {
        self.min_width = min_w;
        self.max_width = max_w.map(Size::Px);
        self.min_height = min_h;
        self.max_height = max_h.map(Size::Px);
        self
    }
}
