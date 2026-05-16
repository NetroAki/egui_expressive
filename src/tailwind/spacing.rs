//! Box-model spacing helpers and Tailwind-style spacing constants.

use egui::Margin;

/// Four-edge spacing used for CSS-like margin and padding.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    /// Same value on all edges.
    pub fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    /// Horizontal (`left`/`right`) and vertical (`top`/`bottom`) values.
    pub fn symmetric(h: f32, v: f32) -> Self {
        Self {
            top: v,
            right: h,
            bottom: v,
            left: h,
        }
    }

    /// Alias for [`Self::symmetric`].
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

/// Tailwind spacing scale, using a 4 px base (`TW_1 == 4.0`).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_convert_to_egui_margin() {
        let margin: Margin = Edges::symmetric(8.0, 12.0).into();
        assert_eq!(margin.left, 8);
        assert_eq!(margin.right, 8);
        assert_eq!(margin.top, 12);
        assert_eq!(margin.bottom, 12);
    }
}
