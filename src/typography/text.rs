use egui::{Color32, FontId, Pos2, Rect, Stroke, Ui, Widget};

use super::core::{TextDecoration, TextOverflow, TextTransform, TypeSpec};

pub(super) fn transformed_text(text: &str, transform: TextTransform) -> String {
    match transform {
        TextTransform::None | TextTransform::SmallCaps => text.to_string(),
        TextTransform::Uppercase => text.to_uppercase(),
        TextTransform::Lowercase => text.to_lowercase(),
        TextTransform::Capitalize => {
            let mut result = String::new();
            let mut capitalize_next = true;
            for ch in text.chars() {
                if ch.is_whitespace() {
                    capitalize_next = true;
                    result.push(ch);
                } else if capitalize_next {
                    result.extend(ch.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.extend(ch.to_lowercase());
                }
            }
            result
        }
    }
}

fn small_caps_glyph(ch: char, font_id: &FontId, spec: &TypeSpec) -> (String, FontId) {
    if spec.text_transform == TextTransform::SmallCaps && ch.is_lowercase() {
        let mut small_font_id = font_id.clone();
        small_font_id.size *= 0.72;
        (ch.to_uppercase().to_string(), small_font_id)
    } else {
        (ch.to_string(), font_id.clone())
    }
}

fn should_use_fast_path(spec: &TypeSpec, word_spacing: f32, is_small_caps: bool) -> bool {
    !is_small_caps && word_spacing == 0.0 && spec.can_use_fast_path()
}

fn measure_text_width(
    painter: &egui::Painter,
    text: &str,
    spec: &TypeSpec,
    word_spacing: f32,
) -> f32 {
    let font_id = spec.to_font_id();
    let color = spec.color.unwrap_or(Color32::BLACK);
    let final_text = transformed_text(text, spec.text_transform);
    let is_small_caps = spec.text_transform == TextTransform::SmallCaps;

    if should_use_fast_path(spec, word_spacing, is_small_caps) {
        return painter
            .layout(final_text, font_id, color, f32::INFINITY)
            .rect
            .width();
    }

    let mut width = 0.0;
    let mut count = 0usize;
    for ch in final_text.chars() {
        let (ch_str, ch_font_id) = small_caps_glyph(ch, &font_id, spec);
        width += painter
            .layout(ch_str, ch_font_id, color, f32::INFINITY)
            .rect
            .width()
            * spec.horizontal_scale;
        if ch == ' ' {
            width += word_spacing;
        }
        count += 1;
    }
    if count > 1 {
        width += spec.letter_spacing * (count - 1) as f32;
    }
    width
}

fn render_text_internal(
    painter: &egui::Painter,
    pos: Pos2,
    text: &str,
    spec: &TypeSpec,
    max_width: Option<f32>,
    word_spacing: f32,
) -> Rect {
    if text.is_empty() {
        return Rect::from_two_pos(pos, pos);
    }

    let font_id = spec.to_font_id();
    let color = spec.color.unwrap_or(Color32::BLACK);
    let display_text = transformed_text(text, spec.text_transform);

    let final_text = if let Some(max_w) = max_width {
        if spec.overflow == TextOverflow::Ellipsis {
            let galley = painter.layout(display_text.clone(), font_id.clone(), color, max_w);
            if galley.rect.width() > max_w {
                let ellipsis = "\u{2026}";
                let mut truncated = display_text.clone();
                while !truncated.is_empty() {
                    truncated.pop();
                    let test_text = truncated.clone() + ellipsis;
                    let test_galley = painter.layout(test_text, font_id.clone(), color, max_w);
                    if test_galley.rect.width() <= max_w {
                        break;
                    }
                }
                if truncated.is_empty() {
                    truncated = ellipsis.to_string();
                } else {
                    truncated.push_str(ellipsis);
                }
                truncated
            } else {
                display_text
            }
        } else {
            display_text
        }
    } else {
        display_text
    };

    let is_small_caps = spec.text_transform == TextTransform::SmallCaps;
    let origin_y = pos.y - spec.baseline_shift;

    if should_use_fast_path(spec, word_spacing, is_small_caps) {
        let galley = painter.layout(final_text.clone(), font_id.clone(), color, f32::INFINITY);
        let rect = galley.rect.translate(pos.to_vec2());
        painter.galley(pos, galley.clone(), color);

        if spec.decoration != TextDecoration::None {
            let underline_y = pos.y + spec.size * 0.15;
            let strikethrough_y = pos.y + spec.size * 0.5;
            let text_width = galley.rect.width();

            if spec.decoration == TextDecoration::Underline
                || spec.decoration == TextDecoration::Both
            {
                painter.add(egui::Shape::LineSegment {
                    points: [
                        Pos2::new(pos.x, underline_y),
                        Pos2::new(pos.x + text_width, underline_y),
                    ],
                    stroke: Stroke::new(spec.size * 0.08, color),
                });
            }
            if spec.decoration == TextDecoration::Strikethrough
                || spec.decoration == TextDecoration::Both
            {
                painter.add(egui::Shape::LineSegment {
                    points: [
                        Pos2::new(pos.x, strikethrough_y),
                        Pos2::new(pos.x + text_width, strikethrough_y),
                    ],
                    stroke: Stroke::new(spec.size * 0.08, color),
                });
            }
        }

        return rect;
    }

    let mut current_x = pos.x;
    let mut max_y = origin_y;
    let mut min_y = origin_y;

    for ch in final_text.chars() {
        let (ch_str, ch_font_id) = small_caps_glyph(ch, &font_id, spec);
        let y_offset = if is_small_caps && ch.is_lowercase() {
            spec.effective_size() * 0.28
        } else {
            0.0
        };
        let ch_galley = painter.layout(ch_str, ch_font_id, color, f32::INFINITY);
        let raw_width = ch_galley.rect.width();
        let ch_width = raw_width * spec.horizontal_scale;
        let ch_max_y = origin_y + y_offset + ch_galley.rect.max.y;
        let ch_min_y = origin_y + y_offset + ch_galley.rect.min.y;

        let ch_pos = Pos2::new(current_x, origin_y + y_offset);
        painter.galley(ch_pos, ch_galley, color);

        current_x += ch_width + spec.letter_spacing;
        if ch == ' ' {
            current_x += word_spacing;
        }
        max_y = max_y.max(ch_max_y);
        min_y = min_y.min(ch_min_y);
    }

    let final_rect = Rect::from_min_max(
        Pos2::new(pos.x, min_y),
        Pos2::new(current_x - spec.letter_spacing, max_y),
    );

    if spec.decoration != TextDecoration::None {
        let underline_y = origin_y + spec.effective_size() * 0.15;
        let strikethrough_y = origin_y + spec.effective_size() * 0.5;
        let text_width = current_x - pos.x - spec.letter_spacing;

        if spec.decoration == TextDecoration::Underline || spec.decoration == TextDecoration::Both {
            painter.add(egui::Shape::LineSegment {
                points: [
                    Pos2::new(pos.x, underline_y),
                    Pos2::new(pos.x + text_width, underline_y),
                ],
                stroke: Stroke::new(spec.effective_size() * 0.08, color),
            });
        }
        if spec.decoration == TextDecoration::Strikethrough
            || spec.decoration == TextDecoration::Both
        {
            painter.add(egui::Shape::LineSegment {
                points: [
                    Pos2::new(pos.x, strikethrough_y),
                    Pos2::new(pos.x + text_width, strikethrough_y),
                ],
                stroke: Stroke::new(spec.effective_size() * 0.08, color),
            });
        }
    }

    final_rect
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextBlockAlign {
    #[default]
    Left,
    Center,
    Right,
    Justified,
    JustifiedLastLineCenter,
    JustifiedLastLineRight,
    JustifiedAll,
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub spec: TypeSpec,
}

impl TextSpan {
    pub fn new(text: impl Into<String>, spec: TypeSpec) -> Self {
        Self {
            text: text.into(),
            spec,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextBlock {
    pub spans: Vec<TextSpan>,
    pub align: TextBlockAlign,
    pub line_height: f32,
    pub layout_width: Option<f32>,
}

impl TextBlock {
    pub fn new(text: impl Into<String>, spec: TypeSpec) -> Self {
        Self {
            spans: vec![TextSpan::new(text, spec)],
            align: TextBlockAlign::Left,
            line_height: 1.2,
            layout_width: None,
        }
    }

    pub fn from_spans(spans: Vec<TextSpan>) -> Self {
        Self {
            spans,
            align: TextBlockAlign::Left,
            line_height: 1.2,
            layout_width: None,
        }
    }

    pub fn align(mut self, align: TextBlockAlign) -> Self {
        self.align = align;
        self
    }
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }
    pub fn layout_width(mut self, layout_width: f32) -> Self {
        self.layout_width = Some(layout_width);
        self
    }
}

impl Default for TextBlock {
    fn default() -> Self {
        Self {
            spans: Vec::new(),
            align: TextBlockAlign::Left,
            line_height: 1.2,
            layout_width: None,
        }
    }
}

fn split_text_block_lines(block: &TextBlock) -> Vec<Vec<TextSpan>> {
    let mut lines: Vec<Vec<TextSpan>> = vec![Vec::new()];
    for span in &block.spans {
        for (idx, part) in span.text.split('\n').enumerate() {
            if idx > 0 {
                lines.push(Vec::new());
            }
            if !part.is_empty() {
                lines
                    .last_mut()
                    .expect("line exists")
                    .push(TextSpan::new(part, span.spec.clone()));
            }
        }
    }
    lines
}

fn measure_span_width(painter: &egui::Painter, span: &TextSpan) -> f32 {
    measure_text_width(painter, &span.text, &span.spec, 0.0)
}

fn calculate_word_spacing(
    align: TextBlockAlign,
    is_last_line: bool,
    space_count: usize,
    available_width: f32,
    line_width: f32,
) -> f32 {
    let should_justify = match align {
        TextBlockAlign::JustifiedAll => true,
        TextBlockAlign::Justified
        | TextBlockAlign::JustifiedLastLineCenter
        | TextBlockAlign::JustifiedLastLineRight => !is_last_line,
        TextBlockAlign::Left | TextBlockAlign::Center | TextBlockAlign::Right => false,
    };

    if should_justify && space_count > 0 && available_width > line_width {
        (available_width - line_width) / space_count as f32
    } else {
        0.0
    }
}

fn line_start_x(
    origin_x: f32,
    align: TextBlockAlign,
    is_last_line: bool,
    available_width: f32,
    line_width: f32,
    word_spacing: f32,
) -> f32 {
    match align {
        TextBlockAlign::Left | TextBlockAlign::Justified | TextBlockAlign::JustifiedAll => origin_x,
        TextBlockAlign::Center => origin_x + (available_width - line_width) * 0.5,
        TextBlockAlign::Right => origin_x + available_width - line_width,
        TextBlockAlign::JustifiedLastLineCenter if is_last_line && word_spacing == 0.0 => {
            origin_x + (available_width - line_width) * 0.5
        }
        TextBlockAlign::JustifiedLastLineRight if is_last_line && word_spacing == 0.0 => {
            origin_x + available_width - line_width
        }
        TextBlockAlign::JustifiedLastLineCenter | TextBlockAlign::JustifiedLastLineRight => {
            origin_x
        }
    }
}

pub fn render_text(
    painter: &egui::Painter,
    pos: Pos2,
    text: &str,
    spec: &TypeSpec,
    max_width: Option<f32>,
) -> Rect {
    render_text_internal(painter, pos, text, spec, max_width, 0.0)
}

pub fn render_text_block(painter: &egui::Painter, origin: Pos2, block: &TextBlock) -> Rect {
    let lines = split_text_block_lines(block);
    let mut bounds: Option<Rect> = None;
    let mut y = origin.y;
    let line_height_multiplier = block.line_height.max(0.1);
    let num_lines = lines.len();

    for (line_idx, line) in lines.into_iter().enumerate() {
        let line_width: f32 = line
            .iter()
            .map(|span| measure_span_width(painter, span))
            .sum();
        let line_height = line
            .iter()
            .map(|span| span.spec.effective_size() * line_height_multiplier)
            .fold(14.0 * line_height_multiplier, f32::max);
        let available_width = block.layout_width.unwrap_or(line_width);

        let is_last_line = line_idx == num_lines - 1;
        let mut space_count = 0;
        for span in &line {
            space_count += span.text.chars().filter(|&c| c == ' ').count();
        }

        let word_spacing = calculate_word_spacing(
            block.align,
            is_last_line,
            space_count,
            available_width,
            line_width,
        );
        let mut x = line_start_x(
            origin.x,
            block.align,
            is_last_line,
            available_width,
            line_width,
            word_spacing,
        );

        if line.is_empty() {
            let line_rect =
                Rect::from_min_size(Pos2::new(origin.x, y), egui::vec2(0.0, line_height));
            bounds = Some(bounds.map_or(line_rect, |rect| rect.union(line_rect)));
            y += line_height;
            continue;
        }

        for span in line {
            let rect = render_text_internal(
                painter,
                Pos2::new(x, y),
                &span.text,
                &span.spec,
                None,
                word_spacing,
            );
            x = rect.max.x;
            bounds = Some(bounds.map_or(rect, |existing| existing.union(rect)));
        }
        y += line_height;
    }

    bounds.unwrap_or_else(|| Rect::from_min_size(origin, egui::Vec2::ZERO))
}

pub struct TypeLabel<'a> {
    text: &'a str,
    spec: TypeSpec,
}

impl<'a> TypeLabel<'a> {
    pub fn new(text: &'a str, spec: TypeSpec) -> Self {
        Self { text, spec }
    }
}

impl Widget for TypeLabel<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let color = self
            .spec
            .color
            .unwrap_or_else(|| ui.style().visuals.text_color());
        let mut render_spec = self.spec;
        render_spec.color = Some(color);
        let width = measure_text_width(ui.painter(), self.text, &render_spec, 0.0);
        let height = render_spec.effective_size() * render_spec.line_height.max(0.1);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        render_text(ui.painter(), rect.min, self.text, &render_spec, None);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typography::{TextDecoration, TextOverflow, TextTransform};

    fn with_test_painter(mut run: impl FnMut(egui::Painter)) {
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            run(ui.painter().clone());
        });
    }

    #[test]
    fn type_spec_transform_boundaries_are_deterministic_for_ascii() {
        assert_eq!(
            transformed_text("phase six", TextTransform::Uppercase),
            "PHASE SIX"
        );
        assert_eq!(
            transformed_text("Phase Six", TextTransform::Lowercase),
            "phase six"
        );
        assert_eq!(
            transformed_text("phase six", TextTransform::Capitalize),
            "Phase Six"
        );
        assert_eq!(
            transformed_text("Phase Six", TextTransform::None),
            "Phase Six"
        );
    }

    #[test]
    fn type_spec_fast_path_boundaries_are_explicit() {
        let default = TypeSpec::new(14.0);
        assert!(should_use_fast_path(&default, 0.0, false));

        assert!(!should_use_fast_path(
            &default.clone().letter_spacing(0.5),
            0.0,
            false
        ));
        assert!(!should_use_fast_path(
            &default.clone().text_transform(TextTransform::SmallCaps),
            0.0,
            true
        ));
        assert!(!should_use_fast_path(&default, 1.0, false));
    }

    #[test]
    fn type_spec_ellipsis_and_decoration_render_bounds_are_stable() {
        let spec = TypeSpec::new(14.0)
            .color(Color32::from_rgb(32, 40, 48))
            .overflow(TextOverflow::Ellipsis)
            .decoration(TextDecoration::Underline);

        with_test_painter(|painter| {
            let full = render_text(&painter, Pos2::new(0.0, 0.0), "ASCII PANEL", &spec, None);
            let clipped = render_text(
                &painter,
                Pos2::new(0.0, 24.0),
                "ASCII PANEL",
                &spec,
                Some(32.0),
            );

            assert!(full.width() > 32.0);
            assert!(clipped.width() <= full.width());
            assert!(full.height() >= 0.0);
        });
    }

    #[test]
    fn phase7_decoration_overflow_subset_stays_ascii_default_font_only() {
        let spec = TypeSpec::new(16.0)
            .color(Color32::from_rgb(15, 23, 42))
            .letter_spacing(0.25)
            .decoration(TextDecoration::Both)
            .overflow(TextOverflow::Ellipsis);

        with_test_painter(|painter| {
            let full = render_text(
                &painter,
                Pos2::new(4.0, 0.0),
                "ASCII DECORATION OVERFLOW",
                &spec,
                None,
            );
            let decorated = render_text(
                &painter,
                Pos2::new(4.0, 8.0),
                "ASCII DECORATION OVERFLOW",
                &spec,
                Some(96.0),
            );
            assert!(decorated.width() <= full.width());
            assert!(decorated.height() >= 0.0);

            let strike = render_text(
                &painter,
                Pos2::new(4.0, 28.0),
                "CLIP ASCII",
                &spec.clone().decoration(TextDecoration::Strikethrough),
                Some(72.0),
            );
            assert!(strike.width() <= decorated.width());
        });

        let tw_contract = include_str!("../../docs/ui-framework/tw-render-contract.md");
        assert!(tw_contract.contains("typography-supported-decoration-overflow"));
        assert!(tw_contract.contains("Font weight is not exact evidence"));
    }
}
