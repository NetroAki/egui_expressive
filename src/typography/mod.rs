//! Typography primitives and rendering helpers.
//!
//! This module is intentionally a thin facade over focused implementation files:
//! [`core`] owns value types, [`text`] owns egui-native text/block rendering,
//! [`shaping`] owns optional font-byte shaping, and [`render`] owns shaped glyph
//! painting. Keep new implementation out of this file.

pub mod core;
pub mod render;
pub mod shaping;
pub mod text;

pub use core::{
    shaped_glyph_run_advance_width, OpenTypeFeatures, ShapedGlyph, ShapedGlyphRun, TextDecoration,
    TextOverflow, TextTransform, TypeScale, TypeSpec,
};
pub use render::{render_shaped_glyph_run, render_text_with_font_bytes};
pub use shaping::shape_text_with_font_bytes;
pub use text::{render_text, render_text_block, TextBlock, TextBlockAlign, TextSpan, TypeLabel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typography_facade_reexports_canonical_typespec_features() {
        let spec = TypeSpec::new(12.0)
            .text_transform(TextTransform::SmallCaps)
            .baseline_shift(2.0)
            .horizontal_scale(0.9)
            .vertical_scale(1.1)
            .ligatures(false);

        assert_eq!(spec.text_transform, TextTransform::SmallCaps);
        assert_eq!(spec.baseline_shift, 2.0);
        assert_eq!(spec.horizontal_scale, 0.9);
        assert_eq!(spec.vertical_scale, 1.1);
        assert!(!spec.open_type_features.ligatures);
        assert!(spec.requires_full_shaper());
    }

    #[test]
    fn typography_type_scale_uses_canonical_typespec() {
        let scale = TypeScale::default();

        assert_eq!(scale.title_md.weight, 500);
        assert_eq!(scale.label_sm.weight, 500);
        assert_eq!(scale.mono.font_family.as_deref(), Some("mono"));
    }

    #[test]
    fn typography_opentype_fast_path_only_accepts_default_features() {
        assert!(OpenTypeFeatures::default().can_use_fast_path());
        assert!(!OpenTypeFeatures::all_off().can_use_fast_path());

        let fractions = OpenTypeFeatures {
            fractions: true,
            ..OpenTypeFeatures::default()
        };
        assert!(!fractions.can_use_fast_path());

        let no_kerning = OpenTypeFeatures {
            kerning: false,
            ..OpenTypeFeatures::default()
        };
        assert!(!no_kerning.can_use_fast_path());
    }
}
