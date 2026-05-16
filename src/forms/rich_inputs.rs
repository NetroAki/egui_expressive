use crate::forms::FormFieldValue;

/// Option metadata for autocomplete and multi-select fields.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChoiceOption {
    pub id: String,
    pub label: String,
    pub keywords: Vec<String>,
    pub disabled: bool,
}

impl ChoiceOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            keywords: Vec::new(),
            disabled: false,
        }
    }

    pub fn keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        query.is_empty()
            || self.label.to_lowercase().contains(&query)
            || self.id.to_lowercase().contains(&query)
            || self
                .keywords
                .iter()
                .any(|keyword| keyword.to_lowercase().contains(&query))
    }
}

/// Pure autocomplete state and filtering behavior.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AutocompleteState {
    pub query: String,
    pub highlighted_index: Option<usize>,
}

impl AutocompleteState {
    pub fn filtered_options<'a>(&self, options: &'a [ChoiceOption]) -> Vec<&'a ChoiceOption> {
        options
            .iter()
            .filter(|option| !option.disabled && option.matches(&self.query))
            .collect()
    }

    pub fn highlighted_option<'a>(&self, options: &'a [ChoiceOption]) -> Option<&'a ChoiceOption> {
        self.highlighted_index
            .and_then(|index| self.filtered_options(options).get(index).copied())
    }
}

/// Ordered multi-select state; selected ids preserve insertion order.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MultiSelectState {
    pub selected_ids: Vec<String>,
}

impl MultiSelectState {
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected_ids.iter().any(|selected| selected == id)
    }

    pub fn toggle(&mut self, id: impl Into<String>) {
        let id = id.into();
        if let Some(index) = self
            .selected_ids
            .iter()
            .position(|selected| selected == &id)
        {
            self.selected_ids.remove(index);
        } else {
            self.selected_ids.push(id);
        }
    }

    pub fn to_field_value(&self) -> FormFieldValue {
        FormFieldValue::List(self.selected_ids.clone())
    }
}

/// Date value with dependency-free Gregorian validity checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DateParts {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl DateParts {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub fn is_valid(self) -> bool {
        (1..=12).contains(&self.month)
            && (1..=days_in_month(self.year, self.month)).contains(&self.day)
    }

    pub fn to_field_value(self) -> Option<FormFieldValue> {
        self.is_valid().then_some(FormFieldValue::Date {
            year: self.year,
            month: self.month,
            day: self.day,
        })
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Time value with minute precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimeParts {
    pub hour: u8,
    pub minute: u8,
}

impl TimeParts {
    pub fn new(hour: u8, minute: u8) -> Self {
        Self { hour, minute }
    }

    pub fn is_valid(self) -> bool {
        self.hour < 24 && self.minute < 60
    }

    pub fn to_field_value(self) -> Option<FormFieldValue> {
        self.is_valid().then_some(FormFieldValue::Time {
            hour: self.hour,
            minute: self.minute,
        })
    }
}

/// Native file-picker intent descriptor. Apps decide if/how to fulfill it.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilePickerRequest {
    pub field_id: String,
    pub title: String,
    pub allowed_extensions: Vec<String>,
    pub must_exist: bool,
}

impl FilePickerRequest {
    pub fn new(field_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            field_id: field_id.into(),
            title: title.into(),
            allowed_extensions: Vec::new(),
            must_exist: true,
        }
    }

    pub fn allow_extension(mut self, extension: impl Into<String>) -> Self {
        self.allowed_extensions.push(extension.into());
        self
    }

    pub fn must_exist(mut self, must_exist: bool) -> Self {
        self.must_exist = must_exist;
        self
    }
}

/// RGBA color value with egui conversion helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RgbaColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColorValue {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_color32(self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(self.r, self.g, self.b, self.a)
    }

    pub fn to_field_value(self) -> FormFieldValue {
        FormFieldValue::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

impl From<egui::Color32> for RgbaColorValue {
    fn from(color: egui::Color32) -> Self {
        let [r, g, b, a] = color.to_array();
        Self { r, g, b, a }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autocomplete_filters_label_id_and_keywords() {
        let state = AutocompleteState {
            query: "osc".to_owned(),
            highlighted_index: Some(0),
        };
        let options = vec![
            ChoiceOption::new("lfo", "LFO").keyword("modulator"),
            ChoiceOption::new("osc1", "Main").keyword("voice"),
        ];

        assert_eq!(state.filtered_options(&options)[0].id, "osc1");
        assert_eq!(state.highlighted_option(&options).unwrap().id, "osc1");
    }

    #[test]
    fn multi_select_toggles_ids_in_order() {
        let mut state = MultiSelectState::default();

        state.toggle("a");
        state.toggle("b");
        state.toggle("a");

        assert_eq!(state.selected_ids, vec!["b".to_owned()]);
    }

    #[test]
    fn date_and_time_validate_without_external_dependencies() {
        assert!(DateParts::new(2024, 2, 29).is_valid());
        assert!(!DateParts::new(2023, 2, 29).is_valid());
        assert!(TimeParts::new(23, 59).is_valid());
        assert!(!TimeParts::new(24, 0).is_valid());
    }

    #[test]
    fn file_picker_request_is_descriptor_only() {
        let request = FilePickerRequest::new("preset", "Choose preset")
            .allow_extension("json")
            .must_exist(true);

        assert_eq!(request.allowed_extensions, vec!["json".to_owned()]);
    }
}
