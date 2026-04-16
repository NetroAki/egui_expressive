#![allow(dead_code)]

use egui::{Color32, Context, CornerRadius, FontId, Pos2, Rect, RichText, Stroke, Ui, Widget};

/// Text decoration style.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Strikethrough,
    Both,
}

/// Text overflow behavior.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextOverflow {
    #[default]
    Visible,
    Ellipsis,
    Clip,
}

/// Text transform style.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

/// Specification for typography styling.
#[derive(Clone, Debug)]
pub struct TypeSpec {
    pub size: f32,
    pub weight: u16,         // 100-900, standard CSS weights
    pub line_height: f32,    // multiplier, e.g. 1.4
    pub letter_spacing: f32, // extra px between chars
    pub color: Option<Color32>,
    pub font_family: Option<String>,
    pub decoration: TextDecoration,
    pub overflow: TextOverflow,
    pub text_transform: TextTransform,
}

impl TypeSpec {
    /// Creates a new TypeSpec with default values and the given size.
    pub fn new(size: f32) -> Self {
        Self {
            size,
            weight: 400,
            line_height: 1.4,
            letter_spacing: 0.0,
            color: None,
            font_family: None,
            decoration: TextDecoration::None,
            overflow: TextOverflow::Visible,
            text_transform: TextTransform::None,
        }
    }

    /// Sets the font weight (100-900).
    pub fn weight(mut self, w: u16) -> Self {
        self.weight = w;
        self
    }

    /// Sets the line height multiplier.
    pub fn line_height(mut self, lh: f32) -> Self {
        self.line_height = lh;
        self
    }

    /// Sets the letter spacing in pixels.
    pub fn letter_spacing(mut self, ls: f32) -> Self {
        self.letter_spacing = ls;
        self
    }

    /// Sets the text color.
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }

    /// Sets the font family.
    pub fn font_family(mut self, f: impl Into<String>) -> Self {
        self.font_family = Some(f.into());
        self
    }

    /// Sets the text decoration.
    pub fn decoration(mut self, d: TextDecoration) -> Self {
        self.decoration = d;
        self
    }

    /// Sets the text overflow behavior.
    pub fn overflow(mut self, o: TextOverflow) -> Self {
        self.overflow = o;
        self
    }

    /// Sets the text transform.
    pub fn text_transform(mut self, t: TextTransform) -> Self {
        self.text_transform = t;
        self
    }

    /// Converts this spec to an egui FontId.
    pub fn to_font_id(&self) -> FontId {
        let family = self
            .font_family
            .as_deref()
            .map(|f| egui::FontFamily::Name(f.into()));
        FontId::new(self.size, family.unwrap_or(egui::FontFamily::Proportional))
    }

    /// Converts this spec to RichText with the given content.
    ///
    /// Note: Weight and letter spacing are best-effort as egui doesn't
    /// support weight natively without separate font files.
    pub fn to_rich_text(&self, text: &str) -> RichText {
        let rich_text = RichText::new(text).size(self.size);

        let rich_text = match &self.font_family {
            Some(f) => rich_text.font(FontId::new(
                self.size,
                egui::FontFamily::Name(f.clone().into()),
            )),
            None => rich_text,
        };

        match self.color {
            Some(c) => rich_text.color(c),
            None => rich_text,
        }
    }
}

impl Default for TypeSpec {
    fn default() -> Self {
        TypeSpec::new(14.0)
    }
}

/// A type scale with named presets matching common design system conventions.
#[derive(Clone, Debug, Default)]
pub struct TypeScale {
    pub display: TypeSpec,  // 57px, weight 400
    pub headline: TypeSpec, // 32px, weight 400
    pub title_lg: TypeSpec, // 22px, weight 400
    pub title_md: TypeSpec, // 16px, weight 500
    pub title_sm: TypeSpec, // 14px, weight 500
    pub body_lg: TypeSpec,  // 16px, weight 400
    pub body_md: TypeSpec,  // 14px, weight 400
    pub body_sm: TypeSpec,  // 12px, weight 400
    pub label_lg: TypeSpec, // 14px, weight 500
    pub label_md: TypeSpec, // 12px, weight 500
    pub label_sm: TypeSpec, // 11px, weight 500
    pub mono: TypeSpec,     // 13px, weight 400, monospace
}

impl TypeScale {
    /// Creates a default type scale with common design system presets.
    pub fn default() -> Self {
        Self {
            display: TypeSpec::new(57.0),
            headline: TypeSpec::new(32.0),
            title_lg: TypeSpec::new(22.0),
            title_md: TypeSpec::new(16.0).weight(500),
            title_sm: TypeSpec::new(14.0).weight(500),
            body_lg: TypeSpec::new(16.0),
            body_md: TypeSpec::new(14.0),
            body_sm: TypeSpec::new(12.0),
            label_lg: TypeSpec::new(14.0).weight(500),
            label_md: TypeSpec::new(12.0).weight(500),
            label_sm: TypeSpec::new(11.0).weight(500),
            mono: TypeSpec::new(13.0).font_family("mono"),
        }
    }

    /// Stores this type scale in egui's context.
    pub fn store(&self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("egui_expressive_type_scale"), self.clone()));
    }

    /// Loads the type scale from egui's context.
    /// Returns the default type scale if none is stored.
    pub fn load(ctx: &Context) -> Self {
        ctx.data(|d| {
            d.get_temp(egui::Id::new("egui_expressive_type_scale"))
                .unwrap_or_else(TypeScale::default)
        })
    }
}

/// Renders text with letter-spacing by advancing char-by-char.
///
/// Returns the bounding rect of the rendered text.
///
/// If letter_spacing is zero, renders using a single painter.text() call.
/// Otherwise, iterates through characters and advances x position by
/// glyph_width + letter_spacing for each character.
pub fn render_text(
    painter: &egui::Painter,
    pos: Pos2,
    text: &str,
    spec: &TypeSpec,
    max_width: Option<f32>,
) -> Rect {
    if text.is_empty() {
        return Rect::from_two_pos(pos, pos);
    }

    let font_id = spec.to_font_id();
    let color = spec.color.unwrap_or(Color32::BLACK);

    // Apply text transform
    let display_text = match spec.text_transform {
        TextTransform::None => text.to_string(),
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
    };

    // Apply ellipsis if needed
    let final_text = if let Some(max_w) = max_width {
        if spec.overflow == TextOverflow::Ellipsis {
            let galley = painter.layout(display_text.clone(), font_id.clone(), color, max_w);
            if galley.rect.width() > max_w {
                // Truncate with ellipsis
                let ellipsis = "\u{2026}"; // …
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

    if spec.letter_spacing == 0.0 {
        // Fast path: no letter spacing, render as single text
        let galley = painter.layout(final_text.clone(), font_id.clone(), color, f32::INFINITY);
        let rect = galley.rect.translate(pos.to_vec2());
        painter.galley(pos, galley.clone(), color);

        // Handle decoration (underline/strikethrough)
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

    // Render character by character with letter spacing
    let mut current_x = pos.x;
    let mut max_y = pos.y;
    let mut min_y = pos.y;
    let galley = painter.layout(" ".to_string(), font_id.clone(), color, f32::INFINITY);
    let _char_width = galley.rect.width();

    for ch in final_text.chars() {
        let ch_str = ch.to_string();
        let ch_galley = painter.layout(ch_str.clone(), font_id.clone(), color, f32::INFINITY);
        let ch_width = ch_galley.rect.width();
        let ch_max_y = ch_galley.rect.max.y;
        let ch_min_y = ch_galley.rect.min.y;

        let ch_pos = Pos2::new(current_x, pos.y);
        painter.galley(ch_pos, ch_galley, color);

        current_x += ch_width + spec.letter_spacing;
        max_y = max_y.max(ch_max_y);
        min_y = min_y.min(ch_min_y);
    }

    // Calculate final bounding rect
    let final_rect = Rect::from_min_max(
        Pos2::new(pos.x, min_y),
        Pos2::new(current_x - spec.letter_spacing, max_y),
    );

    // Handle decoration (underline/strikethrough)
    if spec.decoration != TextDecoration::None {
        let underline_y = pos.y + spec.size * 0.15;
        let strikethrough_y = pos.y + spec.size * 0.5;
        let text_width = current_x - pos.x - spec.letter_spacing;

        if spec.decoration == TextDecoration::Underline || spec.decoration == TextDecoration::Both {
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

    final_rect
}

/// An egui widget that renders text using a TypeSpec.
pub struct TypeLabel<'a> {
    text: &'a str,
    spec: TypeSpec,
}

impl<'a> TypeLabel<'a> {
    /// Creates a new TypeLabel with the given text and spec.
    pub fn new(text: &'a str, spec: TypeSpec) -> Self {
        Self { text, spec }
    }
}

impl Widget for TypeLabel<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let font_id = self.spec.to_font_id();
        let color = self
            .spec
            .color
            .unwrap_or_else(|| ui.style().visuals.text_color());

        // Use fonts_mut to get mutable access to fonts view
        let galley = ui.fonts_mut(|fonts| {
            fonts.layout(self.text.to_string(), font_id.clone(), color, f32::INFINITY)
        });
        let galley_size = galley.rect.size();

        // Allocate response
        let (rect, response) = ui.allocate_exact_size(galley_size, egui::Sense::hover());

        // Get painter and render
        let painter = ui.painter();

        if self.spec.letter_spacing == 0.0 {
            painter.galley(rect.min, galley, color);
        } else {
            // Render with letter spacing
            let mut current_x = rect.min.x;
            for ch in self.text.chars() {
                let ch_str = ch.to_string();
                let ch_galley = ui
                    .fonts_mut(|fonts| fonts.layout(ch_str, font_id.clone(), color, f32::INFINITY));
                let ch_width = ch_galley.rect.width();
                let ch_pos = Pos2::new(current_x, rect.min.y);
                painter.galley(ch_pos, ch_galley, color);
                current_x += ch_width + self.spec.letter_spacing;
            }
        }

        response
    }
}
