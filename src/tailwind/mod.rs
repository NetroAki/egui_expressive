use egui::{Color32, CornerRadius, Frame, Margin, Response, Stroke, Ui};

// ── Supporting types ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, Debug)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
    pub fn symmetric(h: f32, v: f32) -> Self {
        Self {
            top: v,
            right: h,
            bottom: v,
            left: h,
        }
    }
    pub fn axes(h: f32, v: f32) -> Self {
        Self::symmetric(h, v)
    }
}

impl From<f32> for Edges {
    fn from(v: f32) -> Self {
        Self::all(v)
    }
}

impl From<Edges> for Margin {
    fn from(e: Edges) -> Self {
        Margin {
            top: e.top.clamp(-128.0, 127.0).round() as i8,
            right: e.right.clamp(-128.0, 127.0).round() as i8,
            bottom: e.bottom.clamp(-128.0, 127.0).round() as i8,
            left: e.left.clamp(-128.0, 127.0).round() as i8,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub enum Size {
    #[default]
    Auto,
    Px(f32),
    Full,
}

#[derive(Clone, Copy, Default, Debug)]
pub enum FontWeight {
    Light,
    #[default]
    Normal,
    Medium,
    Bold,
}

// ── Spacing scale (Tailwind 4px base) ────────────────────────────────────────

pub const TW_0: f32 = 0.0;
pub const TW_1: f32 = 4.0;
pub const TW_2: f32 = 8.0;
pub const TW_3: f32 = 12.0;
pub const TW_4: f32 = 16.0;
pub const TW_5: f32 = 20.0;
pub const TW_6: f32 = 24.0;
pub const TW_8: f32 = 32.0;
pub const TW_10: f32 = 40.0;
pub const TW_12: f32 = 48.0;
pub const TW_16: f32 = 64.0;
pub const TW_20: f32 = 80.0;
pub const TW_24: f32 = 96.0;
pub const TW_32: f32 = 128.0;
pub const TW_40: f32 = 160.0;
pub const TW_48: f32 = 192.0;
pub const TW_64: f32 = 256.0;

// ── Main Tw builder ───────────────────────────────────────────────────────────

#[derive(Clone, Default, Debug)]
pub struct Tw {
    // Spacing
    pub padding: Edges,

    // Colors
    pub bg: Option<Color32>,
    pub fg: Option<Color32>,
    pub border_color: Option<Color32>,

    // Typography
    /// Note: `font_size` must be applied per-widget using `RichText::new(text).size(size)`.
    pub font_size: Option<f32>,
    pub font_weight: FontWeight,
    /// Note: `letter_spacing` must be applied per-widget using `RichText::new(text).letter_spacing(spacing)`.
    pub letter_spacing: Option<f32>,

    // Sizing
    pub width: Size,
    pub height: Size,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,

    // Borders
    pub border_width: f32,
    pub border_radius: f32,

    // Effects
    pub opacity: f32, // 1.0 = fully opaque
}

impl Tw {
    pub fn new() -> Self {
        Self {
            opacity: 1.0,
            ..Default::default()
        }
    }

    // ── Padding (Tailwind: p-*, px-*, py-*, pt-*, pb-*, pl-*, pr-*) ──────────
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

    // ── Colors ───────────────────────────────────────────────────────────────
    pub fn bg(mut self, c: Color32) -> Self {
        self.bg = Some(c);
        self
    }
    pub fn text_color(mut self, c: Color32) -> Self {
        self.fg = Some(c);
        self
    }
    pub fn border_color(mut self, c: Color32) -> Self {
        self.border_color = Some(c);
        self
    }

    // ── Typography ───────────────────────────────────────────────────────────
    pub fn text_xs(mut self) -> Self {
        self.font_size = Some(10.0);
        self
    }
    pub fn text_sm(mut self) -> Self {
        self.font_size = Some(12.0);
        self
    }
    pub fn text_base(mut self) -> Self {
        self.font_size = Some(14.0);
        self
    }
    pub fn text_lg(mut self) -> Self {
        self.font_size = Some(16.0);
        self
    }
    pub fn text_xl(mut self) -> Self {
        self.font_size = Some(20.0);
        self
    }
    pub fn text_2xl(mut self) -> Self {
        self.font_size = Some(24.0);
        self
    }
    pub fn text_3xl(mut self) -> Self {
        self.font_size = Some(30.0);
        self
    }
    pub fn font_light(mut self) -> Self {
        self.font_weight = FontWeight::Light;
        self
    }
    pub fn font_normal(mut self) -> Self {
        self.font_weight = FontWeight::Normal;
        self
    }
    pub fn font_medium(mut self) -> Self {
        self.font_weight = FontWeight::Medium;
        self
    }
    pub fn font_bold(mut self) -> Self {
        self.font_weight = FontWeight::Bold;
        self
    }
    pub fn tracking(mut self, v: f32) -> Self {
        self.letter_spacing = Some(v);
        self
    }
    pub fn tracking_tight(mut self) -> Self {
        self.letter_spacing = Some(-0.5);
        self
    }
    pub fn tracking_wide(mut self) -> Self {
        self.letter_spacing = Some(0.5);
        self
    }
    pub fn tracking_wider(mut self) -> Self {
        self.letter_spacing = Some(1.0);
        self
    }

    // ── Sizing ───────────────────────────────────────────────────────────────
    pub fn w(mut self, v: f32) -> Self {
        self.width = Size::Px(v);
        self
    }
    pub fn h(mut self, v: f32) -> Self {
        self.height = Size::Px(v);
        self
    }
    pub fn w_full(mut self) -> Self {
        self.width = Size::Full;
        self
    }
    pub fn h_full(mut self) -> Self {
        self.height = Size::Full;
        self
    }
    pub fn min_w(mut self, v: f32) -> Self {
        self.min_width = Some(v);
        self
    }
    pub fn min_h(mut self, v: f32) -> Self {
        self.min_height = Some(v);
        self
    }
    pub fn max_w(mut self, v: f32) -> Self {
        self.max_width = Some(v);
        self
    }
    pub fn max_h(mut self, v: f32) -> Self {
        self.max_height = Some(v);
        self
    }

    // ── Borders ──────────────────────────────────────────────────────────────
    pub fn rounded(mut self, r: f32) -> Self {
        self.border_radius = r;
        self
    }
    pub fn rounded_none(mut self) -> Self {
        self.border_radius = 0.0;
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

    // ── Effects ──────────────────────────────────────────────────────────────
    pub fn opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }

    // ── SwiftUI aliases ───────────────────────────────────────────────────────
    /// SwiftUI: `.padding(16.0)` — uniform padding
    pub fn padding(self, v: f32) -> Self {
        self.p(v)
    }
    /// SwiftUI: `.background(color)`
    pub fn background(self, c: Color32) -> Self {
        self.bg(c)
    }
    /// SwiftUI: `.foregroundColor(color)`
    pub fn foreground_color(self, c: Color32) -> Self {
        self.text_color(c)
    }
    /// SwiftUI: `.cornerRadius(8.0)`
    pub fn corner_radius(self, r: f32) -> Self {
        self.rounded(r)
    }
    /// SwiftUI: `.frame(width:height:)`
    pub fn frame_size(mut self, width: Option<f32>, height: Option<f32>) -> Self {
        if let Some(w) = width {
            self.width = Size::Px(w);
        }
        if let Some(h) = height {
            self.height = Size::Px(h);
        }
        self
    }
    /// SwiftUI: `.frame(minWidth:maxWidth:minHeight:maxHeight:)`
    pub fn frame_constraints(
        mut self,
        min_w: Option<f32>,
        max_w: Option<f32>,
        min_h: Option<f32>,
        max_h: Option<f32>,
    ) -> Self {
        self.min_width = min_w;
        self.max_width = max_w;
        self.min_height = min_h;
        self.max_height = max_h;
        self
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    /// Build the egui Frame from this style.
    pub fn to_frame(&self) -> Frame {
        let mut f = Frame::NONE;
        if let Some(bg) = self.bg {
            f = f.fill(bg);
        }
        if self.border_radius > 0.0 {
            let r = self.border_radius.min(255.0) as u8;
            f = f.corner_radius(CornerRadius::same(r));
        }
        if self.border_width > 0.0 {
            let color = self.border_color.unwrap_or(Color32::from_gray(100));
            f = f.stroke(Stroke::new(self.border_width, color));
        }
        let m: Margin = self.padding.into();
        f = f.inner_margin(m);
        f
    }

    /// Render content inside this styled container.
    /// Returns the outer Response.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
        let min_w = self.min_width;
        let min_h = self.min_height;
        let max_w = self.max_width;
        let max_h = self.max_height;
        let width = self.width;
        let height = self.height;
        let frame = self.to_frame();
        let fg = self.fg;

        let resp = frame.show(ui, |ui| {
            // Apply text color (fg)
            if let Some(fg) = fg {
                ui.visuals_mut().override_text_color = Some(fg);
            }

            // Apply sizing constraints
            match width {
                Size::Full => ui.set_width(ui.available_width()),
                Size::Px(w) => ui.set_width(w),
                Size::Auto => {}
            }
            match height {
                Size::Full => ui.set_height(ui.available_height()),
                Size::Px(h) => ui.set_height(h),
                Size::Auto => {}
            }
            if let Some(w) = min_w {
                ui.set_min_width(w);
            }
            if let Some(h) = min_h {
                ui.set_min_height(h);
            }
            if let Some(w) = max_w {
                ui.set_max_width(w);
            }
            if let Some(h) = max_h {
                ui.set_max_height(h);
            }

            content(ui);
        });

        resp.response
    }
}
