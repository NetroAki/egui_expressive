/// Text mask where `#` accepts digits and `A` accepts ASCII alphanumeric input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextMask {
    pub pattern: String,
}

impl TextMask {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }

    pub fn sanitize_paste(&self, text: &str) -> String {
        text.chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .collect()
    }

    pub fn format(&self, raw: &str) -> String {
        let mut chars = self.sanitize_paste(raw).chars().collect::<Vec<_>>();
        chars.reverse();
        let mut output = String::new();
        for slot in self.pattern.chars() {
            match slot {
                '#' => push_matching(&mut output, &mut chars, char::is_ascii_digit),
                'A' => push_matching(&mut output, &mut chars, char::is_ascii_alphanumeric),
                literal => output.push(literal),
            }
        }
        output
            .trim_end_matches(|ch: char| !ch.is_ascii_alphanumeric())
            .to_owned()
    }
}

fn push_matching(output: &mut String, chars: &mut Vec<char>, accepts: impl Fn(&char) -> bool) {
    while let Some(ch) = chars.pop() {
        if accepts(&ch) {
            output.push(ch);
            break;
        }
    }
}

/// Numeric parsing and clamp contract for text-backed numeric fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericConstraint {
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
}

impl NumericConstraint {
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            step: None,
        }
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step.max(f64::EPSILON));
        self
    }

    pub fn clamp(&self, value: f64) -> f64 {
        value.clamp(self.min, self.max)
    }

    pub fn parse_clamped(&self, text: &str) -> Option<f64> {
        text.trim()
            .parse::<f64>()
            .ok()
            .map(|value| self.clamp(value))
    }
}

/// Selection metadata used to document deterministic text-input behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSelectionRange {
    pub start: usize,
    pub end: usize,
}

/// Application-level text direction hint for labels, placeholders, and custom text surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    LocaleDefault,
    LeftToRight,
    RightToLeft,
}

impl TextSelectionRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn normalized(self) -> Self {
        if self.start <= self.end {
            self
        } else {
            Self {
                start: self.end,
                end: self.start,
            }
        }
    }
}

/// Platform-sensitive text behavior contract documented by Forms v2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputTextContract {
    pub ime_composition_is_owned_by_egui: bool,
    pub paste_is_sanitized_before_validation: bool,
    pub rtl_bidi_is_platform_limited: bool,
    pub locale_formatting_is_application_owned: bool,
    pub text_direction: TextDirection,
}

impl Default for InputTextContract {
    fn default() -> Self {
        Self {
            ime_composition_is_owned_by_egui: true,
            paste_is_sanitized_before_validation: true,
            rtl_bidi_is_platform_limited: true,
            locale_formatting_is_application_owned: true,
            text_direction: TextDirection::LocaleDefault,
        }
    }
}

impl InputTextContract {
    pub fn ime_composition_owned_by_egui(mut self, owned_by_egui: bool) -> Self {
        self.ime_composition_is_owned_by_egui = owned_by_egui;
        self
    }

    pub fn sanitize_paste_before_validation(mut self, sanitize: bool) -> Self {
        self.paste_is_sanitized_before_validation = sanitize;
        self
    }

    pub fn rtl_bidi_platform_limited(mut self, platform_limited: bool) -> Self {
        self.rtl_bidi_is_platform_limited = platform_limited;
        self
    }

    pub fn locale_formatting_application_owned(mut self, application_owned: bool) -> Self {
        self.locale_formatting_is_application_owned = application_owned;
        self
    }

    pub fn text_direction(mut self, text_direction: TextDirection) -> Self {
        self.text_direction = text_direction;
        self
    }

    pub fn requires_platform_review(&self) -> bool {
        self.ime_composition_is_owned_by_egui || self.rtl_bidi_is_platform_limited
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_mask_formats_pasted_digits() {
        let mask = TextMask::new("###-##");

        assert_eq!(mask.format("12a345"), "123-45");
    }

    #[test]
    fn numeric_constraint_parses_and_clamps() {
        let constraint = NumericConstraint::new(0.0, 10.0);

        assert_eq!(constraint.parse_clamped("12.5"), Some(10.0));
        assert_eq!(constraint.parse_clamped("nope"), None);
    }

    #[test]
    fn selection_range_normalizes_order() {
        assert_eq!(
            TextSelectionRange::new(5, 2).normalized(),
            TextSelectionRange::new(2, 5)
        );
    }

    #[test]
    fn input_contract_records_direction_and_platform_review() {
        let contract = InputTextContract::default()
            .ime_composition_owned_by_egui(false)
            .sanitize_paste_before_validation(false)
            .rtl_bidi_platform_limited(false)
            .locale_formatting_application_owned(false)
            .text_direction(TextDirection::RightToLeft);

        assert_eq!(contract.text_direction, TextDirection::RightToLeft);
        assert!(!contract.requires_platform_review());
        assert!(!contract.paste_is_sanitized_before_validation);
        assert!(!contract.locale_formatting_is_application_owned);
    }
}
