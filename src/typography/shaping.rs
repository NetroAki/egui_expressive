use super::core::{ShapedGlyph, ShapedGlyphRun, TypeSpec};

fn rustybuzz_feature(tag: &[u8; 4], value: bool) -> rustybuzz::Feature {
    rustybuzz::Feature::new(
        rustybuzz::ttf_parser::Tag::from_bytes(tag),
        if value { 1 } else { 0 },
        ..,
    )
}

fn rustybuzz_features_for_type_spec(spec: &TypeSpec) -> Vec<rustybuzz::Feature> {
    let features = spec.open_type_features;
    vec![
        rustybuzz_feature(b"liga", features.ligatures),
        rustybuzz_feature(b"clig", features.contextual_ligatures),
        rustybuzz_feature(b"dlig", features.discretionary_ligatures),
        rustybuzz_feature(b"frac", features.fractions),
        rustybuzz_feature(b"ordn", features.ordinals),
        rustybuzz_feature(b"swsh", features.swash),
        rustybuzz_feature(b"titl", features.titling_alternates),
        rustybuzz_feature(b"salt", features.stylistic_alternates),
        rustybuzz_feature(b"kern", features.kerning),
    ]
}

fn shape_text_with_font_bytes_with_shaper<F>(
    font_data: &[u8],
    text: &str,
    spec: &TypeSpec,
    shaper: F,
) -> Option<ShapedGlyphRun>
where
    F: FnOnce(
        &rustybuzz::Face,
        &[rustybuzz::Feature],
        rustybuzz::UnicodeBuffer,
    ) -> rustybuzz::GlyphBuffer,
{
    let display_text = super::text::transformed_text(text, spec.text_transform);
    let face = rustybuzz::Face::from_slice(font_data, 0)?;
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(&display_text);
    buffer.guess_segment_properties();
    let features = rustybuzz_features_for_type_spec(spec);
    let shaped = shaper(&face, &features, buffer);
    let infos = shaped.glyph_infos();
    let positions = shaped.glyph_positions();
    let glyphs = infos
        .iter()
        .zip(positions.iter())
        .map(|(info, pos)| ShapedGlyph {
            glyph_id: info.glyph_id,
            cluster: info.cluster,
            advance_x: pos.x_advance as f32 / 64.0,
            advance_y: pos.y_advance as f32 / 64.0,
            offset_x: pos.x_offset as f32 / 64.0,
            offset_y: pos.y_offset as f32 / 64.0,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Some(ShapedGlyphRun {
        text: display_text,
        glyphs,
    })
}

pub fn shape_text_with_font_bytes(
    font_data: &[u8],
    text: &str,
    spec: &TypeSpec,
) -> Option<ShapedGlyphRun> {
    shape_text_with_font_bytes_with_shaper(font_data, text, spec, rustybuzz::shape)
}
